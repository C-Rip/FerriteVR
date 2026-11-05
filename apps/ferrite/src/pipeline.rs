//! Pipeline de procesamiento en tiempo real de FerriteVR.
//!
//! Conecta una fuente de fotogramas ([`FuenteFrames`]), el motor de inferencia de pose 2D
//! ([`MotorPose`]), el solver cinemático 3D ([`SolverCinematico`]) y el emisor OSC hacia VRChat ([`EmisorVrchat`]).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ferrite_osc::{muestras_desde_esqueleto, EmisorVrchat};
use ferrite_pose::MotorPose;
use ferrite_sensor::FuenteFrames;
use ferrite_solver::SolverCinematico;
use tracing::info;

/// Opciones de configuración para la ejecución del pipeline de seguimiento corporal.
#[derive(Debug, Clone)]
pub struct OpcionesPipeline {
    /// Frecuencia objetivo de procesamiento en hercios (Hz).
    pub hz: u32,
    /// Duración máxima en segundos (opcional; si es `None` corre indefinidamente).
    pub duracion_seg: Option<u64>,
}

/// Ejecuta el ciclo continuo de captura, estimación 2D, resolución 3D y emisión OSC.
pub fn ejecutar_pipeline(
    mut fuente: Box<dyn FuenteFrames>,
    mut motor_pose: MotorPose,
    mut solver: SolverCinematico,
    emisor: EmisorVrchat,
    opciones: OpcionesPipeline,
    cancelar: Arc<AtomicBool>,
) -> Result<()> {
    let periodo_objetivo = if opciones.hz > 0 {
        Duration::from_secs_f64(1.0 / (opciones.hz as f64))
    } else {
        Duration::from_millis(33)
    };

    let instante_inicio = Instant::now();
    let duracion_maxima = opciones.duracion_seg.map(Duration::from_secs);

    let mut contador_frames: u64 = 0;
    let mut tiempo_ultimo_reporte = Instant::now();
    let mut frames_en_intervalo: u32 = 0;
    let mut suma_tiempo_inferencia = Duration::ZERO;
    let mut suma_tiempo_solver = Duration::ZERO;
    let mut ultimo_conteo_trackers = 0;

    info!(
        hz = opciones.hz,
        destino = %emisor.destino(),
        "Iniciando pipeline de seguimiento corporal en tiempo real..."
    );

    let mut siguiente_fotograma = Instant::now();

    while !cancelar.load(Ordering::Relaxed) {
        if let Some(maxima) = duracion_maxima {
            if instante_inicio.elapsed() >= maxima {
                info!(
                    segundos = maxima.as_secs_f64(),
                    "Duracion maxima alcanzada. Finalizando pipeline..."
                );
                break;
            }
        }

        // 1. Obtención del fotograma de entrada
        let t_espera_frame = Duration::from_millis(50);
        let frame_opt = fuente
            .siguiente_frame(t_espera_frame)
            .context("Error al obtener el siguiente fotograma de la fuente de captura")?;

        let Some(frame) = frame_opt else {
            info!("Fin de la secuencia de fotogramas alcanzado.");
            break;
        };

        // 2. Inferencia de pose 2D con RTMPose Halpe26
        let t_ini_inferencia = Instant::now();
        let pose_2d = motor_pose
            .estimar(&frame)
            .context("Error durante la inferencia de pose 2D")?;
        let dur_inferencia = t_ini_inferencia.elapsed();

        // 3. Elevación 3D, filtrado One Euro y cálculo de rotaciones anatómicas
        let t_ini_solver = Instant::now();
        let esqueleto_3d = solver
            .resolver(&pose_2d, &frame)
            .context("Error durante la resolucion cinematica 3D")?;
        let dur_solver = t_ini_solver.elapsed();

        // 4. Muestreo y emisión del paquete OSC atómico a VRChat
        let muestras = muestras_desde_esqueleto(&esqueleto_3d);
        ultimo_conteo_trackers = muestras.len();
        emisor
            .enviar_frame(&muestras)
            .context("Error al transmitir paquete OSC hacia VRChat")?;

        contador_frames += 1;
        frames_en_intervalo += 1;
        suma_tiempo_inferencia += dur_inferencia;
        suma_tiempo_solver += dur_solver;

        // Telemetría periódica cada segundo
        let tiempo_transcurrido = tiempo_ultimo_reporte.elapsed();
        if tiempo_transcurrido >= Duration::from_secs(1) {
            let fps_real = (frames_en_intervalo as f64) / tiempo_transcurrido.as_secs_f64();
            let avg_inferencia_ms =
                (suma_tiempo_inferencia.as_secs_f64() * 1000.0) / (frames_en_intervalo as f64);
            let avg_solver_ms =
                (suma_tiempo_solver.as_secs_f64() * 1000.0) / (frames_en_intervalo as f64);

            info!(
                fps = format_args!("{fps_real:.1}"),
                inferencia_ms = format_args!("{avg_inferencia_ms:.2}"),
                solver_ms = format_args!("{avg_solver_ms:.2}"),
                trackers = ultimo_conteo_trackers,
                "Metricas de seguimiento"
            );

            tiempo_ultimo_reporte = Instant::now();
            frames_en_intervalo = 0;
            suma_tiempo_inferencia = Duration::ZERO;
            suma_tiempo_solver = Duration::ZERO;
        }

        // Control de temporización para no saturar el procesador
        siguiente_fotograma += periodo_objetivo;
        let ahora = Instant::now();
        if let Some(espera) = siguiente_fotograma.checked_duration_since(ahora) {
            std::thread::sleep(espera);
        } else {
            siguiente_fotograma = ahora;
        }
    }

    let tiempo_total = instante_inicio.elapsed().as_secs_f64();
    let fps_promedio = if tiempo_total > 0.0 {
        (contador_frames as f64) / tiempo_total
    } else {
        0.0
    };

    info!(
        frames = contador_frames,
        segundos = format_args!("{tiempo_total:.1}"),
        fps_promedio = format_args!("{fps_promedio:.1}"),
        trackers = ultimo_conteo_trackers,
        "Pipeline de seguimiento finalizado limpiamente"
    );

    Ok(())
}
