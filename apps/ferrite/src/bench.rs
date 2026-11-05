//! Subcomando de medición de rendimiento (benchmark) para el pipeline de inferencia y solver.

use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use ferrite_nucleo::IntrinsecosProfundidad;
use ferrite_pose::{BackendInferencia, MotorPose};
use ferrite_sensor::GeneradorSinteticoSensor;
use ferrite_solver::SolverCinematico;
use tracing::info;

/// Opciones de configuración para el subcomando `bench`.
#[derive(Debug, Clone)]
pub struct OpcionesBench {
    /// Número de iteraciones a ejecutar.
    pub iteraciones: usize,
    /// Ruta al modelo ONNX RTMPose Halpe26.
    pub ruta_modelo: PathBuf,
    /// Backend de inferencia (`cpu`, `cuda`, `tensorrt`).
    pub backend_str: String,
}

/// Ejecuta la batería de pruebas de latencia y muestra las estadísticas p50/p90/p99.
pub fn ejecutar_bench(opciones: OpcionesBench) -> Result<()> {
    if opciones.iteraciones == 0 {
        bail!("El numero de iteraciones debe ser mayor a 0");
    }

    if !opciones.ruta_modelo.exists() {
        bail!(
            "No se encontro el modelo ONNX en '{}'.\n\
             Ejecuta './scripts/descargar-modelos.sh' para descargarlo e instalarlo.",
            opciones.ruta_modelo.display()
        );
    }

    let backend = opciones
        .backend_str
        .parse::<BackendInferencia>()
        .map_err(|e| anyhow::anyhow!(e))?;

    info!(
        iteraciones = opciones.iteraciones,
        modelo = %opciones.ruta_modelo.display(),
        backend = ?backend,
        "Iniciando medicion de rendimiento (benchmark)..."
    );

    let intrinsecos = IntrinsecosProfundidad::default();
    let generador = GeneradorSinteticoSensor::nuevo(intrinsecos, 30.0);
    let mut motor_pose = MotorPose::nuevo(&opciones.ruta_modelo, backend)
        .context("Fallo al inicializar el motor de inferencia ONNX")?;
    let mut solver = SolverCinematico::nuevo(intrinsecos);

    // Calentamiento (warm-up)
    info!("Ejecutando 5 iteraciones de calentamiento...");
    for i in 0..5 {
        let frame = generador.generar_fotograma(i * 33_333);
        let pose = motor_pose
            .estimar(&frame)
            .context("Error en calentamiento de pose")?;
        let _ = solver
            .resolver(&pose, &frame)
            .context("Error en calentamiento de solver")?;
    }

    let mut latencias_inferencia_ms = Vec::with_capacity(opciones.iteraciones);
    let mut latencias_solver_ms = Vec::with_capacity(opciones.iteraciones);
    let mut latencias_totales_ms = Vec::with_capacity(opciones.iteraciones);

    info!(
        "Midiendo {} iteraciones de inferencia y solver...",
        opciones.iteraciones
    );
    let inicio_total = Instant::now();

    for i in 0..opciones.iteraciones {
        let frame = generador.generar_fotograma((i as u64) * 33_333);

        let t0 = Instant::now();
        let pose = motor_pose
            .estimar(&frame)
            .context("Error durante la inferencia")?;
        let t1 = Instant::now();

        let _esqueleto = solver
            .resolver(&pose, &frame)
            .context("Error durante la resolucion cinematica")?;
        let t2 = Instant::now();

        let dur_inferencia = (t1.duration_since(t0).as_secs_f64()) * 1000.0;
        let dur_solver = (t2.duration_since(t1).as_secs_f64()) * 1000.0;
        let dur_total = (t2.duration_since(t0).as_secs_f64()) * 1000.0;

        latencias_inferencia_ms.push(dur_inferencia);
        latencias_solver_ms.push(dur_solver);
        latencias_totales_ms.push(dur_total);
    }

    let tiempo_total_seg = inicio_total.elapsed().as_secs_f64();
    let fps_global = (opciones.iteraciones as f64) / tiempo_total_seg;

    latencias_inferencia_ms.sort_by(|a, b| a.total_cmp(b));
    latencias_solver_ms.sort_by(|a, b| a.total_cmp(b));
    latencias_totales_ms.sort_by(|a, b| a.total_cmp(b));

    let calcular_estadisticas = |datos: &[f64]| -> (f64, f64, f64, f64, f64) {
        let n = datos.len();
        let media = datos.iter().sum::<f64>() / (n as f64);
        let min = datos[0];
        let p50 = datos[n / 2];
        let p90 = datos[(n * 90) / 100];
        let p99 = datos[(n * 99) / 100];
        (min, p50, p90, p99, media)
    };

    let (inf_min, inf_p50, inf_p90, inf_p99, inf_media) =
        calcular_estadisticas(&latencias_inferencia_ms);
    let (sol_min, sol_p50, sol_p90, sol_p99, sol_media) =
        calcular_estadisticas(&latencias_solver_ms);
    let (tot_min, tot_p50, tot_p90, tot_p99, tot_media) =
        calcular_estadisticas(&latencias_totales_ms);

    println!();
    println!("============================================================");
    println!("           RESULTADOS DE RENDIMIENTO (BENCHMARK)           ");
    println!("============================================================");
    println!(
        "Backend:           {:?} (RTMPose Halpe26)",
        motor_pose.backend()
    );
    println!("Iteraciones:       {}", opciones.iteraciones);
    println!("FPS Global:        {:.1} FPS", fps_global);
    println!("------------------------------------------------------------");
    println!("Etapa              Min (ms)  p50 (ms)  p90 (ms)  p99 (ms)  Media");
    println!("------------------------------------------------------------");
    println!(
        "Inferencia 2D      {:8.2}  {:8.2}  {:8.2}  {:8.2}  {:8.2}",
        inf_min, inf_p50, inf_p90, inf_p99, inf_media
    );
    println!(
        "Solver 3D          {:8.2}  {:8.2}  {:8.2}  {:8.2}  {:8.2}",
        sol_min, sol_p50, sol_p90, sol_p99, sol_media
    );
    println!(
        "Pipeline Total     {:8.2}  {:8.2}  {:8.2}  {:8.2}  {:8.2}",
        tot_min, tot_p50, tot_p90, tot_p99, tot_media
    );
    println!("============================================================");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ejecutar_bench_corto() {
        let ruta_modelo = PathBuf::from("../../modelos/rtmpose-m-halpe26.onnx");
        let ruta_modelo = if ruta_modelo.exists() {
            ruta_modelo
        } else {
            PathBuf::from("modelos/rtmpose-m-halpe26.onnx")
        };

        if !ruta_modelo.exists() {
            return;
        }

        let opciones = OpcionesBench {
            iteraciones: 2,
            ruta_modelo,
            backend_str: "cpu".to_string(),
        };

        let res = ejecutar_bench(opciones);
        assert!(res.is_ok());
    }
}
