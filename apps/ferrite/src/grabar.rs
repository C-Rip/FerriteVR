//! Subcomando de grabación de sesiones en formato `.ferrite`.

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use ferrite_nucleo::IntrinsecosProfundidad;
use ferrite_sensor::{FuenteFrames, GeneradorSinteticoSensor, GrabadorFerrite};
use tracing::info;

/// Ejecuta la grabación de una sesión en un archivo binario `.ferrite`.
pub fn ejecutar_grabar(
    salida: PathBuf,
    segundos: u64,
    sintetico: bool,
    hz: u32,
    cancelar: Arc<AtomicBool>,
) -> Result<()> {
    if hz == 0 {
        bail!("La frecuencia de grabacion debe ser mayor a 0 Hz");
    }

    if !sintetico {
        bail!(
            "La captura directa desde Kinect v2 fisico requiere la compilacion de libfreenect2 (Fase 3).\n\
             Por ahora puedes grabar una sesion de prueba agregando la bandera '--sintetico':\n\
             cargo run -p ferrite -- grabar --salida {} --segundos {} --sintetico",
            salida.display(),
            segundos
        );
    }

    info!(
        salida = %salida.display(),
        segundos,
        hz,
        sintetico,
        "Iniciando grabacion de sesion .ferrite..."
    );

    let intrinsecos = IntrinsecosProfundidad::default();
    let mut fuente = GeneradorSinteticoSensor::nuevo(intrinsecos, hz as f32);

    let archivo = File::create(&salida)
        .with_context(|| format!("No se pudo crear el archivo de grabacion: {}", salida.display()))?;
    let escritor_buf = BufWriter::new(archivo);
    let mut grabador = GrabadorFerrite::nuevo(escritor_buf, intrinsecos, hz as f32)
        .context("Fallo al inicializar el grabador .ferrite")?;

    let periodo = Duration::from_secs_f64(1.0 / (hz as f64));
    let frames_totales = segundos.saturating_mul(hz as u64);
    let mut frames_grabados: u64 = 0;
    let instante_inicio = Instant::now();
    let mut siguiente_frame = Instant::now();

    while !cancelar.load(Ordering::Relaxed) && frames_grabados < frames_totales {
        let frame_opt = fuente
            .siguiente_frame(Duration::from_millis(100))
            .context("Error al capturar fotograma para grabacion")?;

        let Some(frame) = frame_opt else {
            break;
        };

        grabador
            .escribir_frame(&frame)
            .context("Fallo al escribir fotograma comprimido")?;

        frames_grabados += 1;

        if frames_grabados % (hz as u64) == 0 {
            info!(
                progreso = format_args!("{frames_grabados}/{frames_totales}"),
                segundos_transcurridos =
                    format_args!("{:.1}", instante_inicio.elapsed().as_secs_f64()),
                "Grabando..."
            );
        }

        siguiente_frame += periodo;
        let ahora = Instant::now();
        if let Some(espera) = siguiente_frame.checked_duration_since(ahora) {
            std::thread::sleep(espera);
        } else {
            siguiente_frame = ahora;
        }
    }

    grabador
        .vaciar()
        .context("Fallo al vaciar buffer de grabacion")?;

    let tamano_bytes = std::fs::metadata(&salida).map(|m| m.len()).unwrap_or(0);
    let tamano_mb = (tamano_bytes as f64) / (1024.0 * 1024.0);

    info!(
        archivo = %salida.display(),
        frames = frames_grabados,
        duracion_seg = format_args!("{:.2}", instante_inicio.elapsed().as_secs_f64()),
        tamano_mb = format_args!("{tamano_mb:.2}"),
        "Grabacion completada exitosamente"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ejecutar_grabar_sintetico() {
        let ruta_temp = std::env::temp_dir().join("test_sesion_sintetica.ferrite");
        let cancelar = Arc::new(AtomicBool::new(false));

        let res = ejecutar_grabar(ruta_temp.clone(), 1, true, 30, cancelar);
        assert!(res.is_ok());
        assert!(ruta_temp.exists());
        let meta = std::fs::metadata(&ruta_temp).expect("Metadatos de archivo");
        assert!(meta.len() > 48); // Mayor que el encabezado

        let _ = std::fs::remove_file(ruta_temp);
    }
}
