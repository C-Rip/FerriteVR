//! Binario CLI principal de FerriteVR.
//!
//! Proporciona las herramientas de línea de comandos para captura de sensor,
//! inferencia de pose, emisión de telemetría OSC hacia VRChat y simulación sintética.

#![deny(non_ascii_idents)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

mod bench;
mod grabar;
mod pipeline;
mod reproducir;
mod sintetico;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use ferrite_osc::{EmisorVrchat, muestras_desde_esqueleto};
use tracing::{info, warn};

use bench::{OpcionesBench, ejecutar_bench};
use grabar::ejecutar_grabar;
use reproducir::{OpcionesReproducir, ejecutar_reproducir};

/// Interfaz de línea de comandos de FerriteVR.
#[derive(Debug, Parser)]
#[command(
    name = "ferrite",
    version,
    about = "Seguimiento corporal completo con Kinect v2 para VRChat en Linux",
    long_about = "FerriteVR procesa en tiempo real color y profundidad con RTMPose para proporcionar seguimiento de cuerpo completo (FBT) a VRChat mediante el protocolo OSC."
)]
struct Cli {
    #[command(subcommand)]
    comando: Comandos,
}

/// Subcomandos disponibles en la CLI de FerriteVR.
#[derive(Debug, Subcommand)]
enum Comandos {
    /// Genera y emite un esqueleto sintético de caminata para pruebas en VRChat sin sensor físico ni inferencia.
    Fake {
        /// Dirección de red y puerto UDP del receptor OSC de VRChat.
        #[arg(short, long, default_value = "127.0.0.1:9000")]
        destino: SocketAddr,

        /// Frecuencia de emisión de paquetes OSC en hercios (Hz).
        #[arg(long, default_value_t = 30)]
        hz: u32,

        /// Duración máxima de la simulación en segundos (opcional; por defecto corre indefinidamente).
        #[arg(long)]
        duracion_seg: Option<u64>,
    },

    /// Ejecuta el pipeline completo de seguimiento corporal en tiempo real.
    Run {
        /// Ejecuta en modo sintético simulado si no hay hardware Kinect conectado.
        #[arg(long, default_value_t = false)]
        sintetico: bool,

        /// Ruta al modelo ONNX RTMPose Halpe26.
        #[arg(long, default_value = "modelos/rtmpose-m-halpe26.onnx")]
        modelo: PathBuf,

        /// Backend de inferencia (`cpu`, `cuda`, `tensorrt`).
        #[arg(long, default_value = "cpu")]
        backend: String,

        /// Dirección de red y puerto UDP del receptor OSC de VRChat.
        #[arg(short, long, default_value = "127.0.0.1:9000")]
        destino: SocketAddr,

        /// Frecuencia objetivo de muestreo en hercios (Hz).
        #[arg(long, default_value_t = 30)]
        hz: u32,

        /// Duración máxima en segundos (opcional).
        #[arg(long)]
        duracion_seg: Option<u64>,
    },

    /// Graba los flujos de profundidad y color en un archivo binario `.ferrite`.
    Grabar {
        /// Ruta de destino del archivo `.ferrite`.
        #[arg(short, long, default_value = "sesion.ferrite")]
        salida: PathBuf,

        /// Duración de la grabación en segundos.
        #[arg(short, long, default_value_t = 10)]
        segundos: u64,

        /// Graba fotogramas sintéticos de prueba sin sensor Kinect v2 físico.
        #[arg(long, default_value_t = false)]
        sintetico: bool,

        /// Frecuencia de muestreo en hercios (Hz).
        #[arg(long, default_value_t = 30)]
        hz: u32,
    },

    /// Reproduce una sesión grabada previamente o fotogramas sintéticos ejecutando inferencia y solver hacia VRChat.
    Reproducir {
        /// Ruta al archivo de sesión `.ferrite` a reproducir.
        #[arg(value_name = "ARCHIVO")]
        archivo: Option<PathBuf>,

        /// Ejecuta fotogramas sintéticos si no se especifica un archivo grabado.
        #[arg(long, default_value_t = false)]
        sintetico: bool,

        /// Ruta al modelo ONNX RTMPose Halpe26.
        #[arg(long, default_value = "modelos/rtmpose-m-halpe26.onnx")]
        modelo: PathBuf,

        /// Backend de inferencia (`cpu`, `cuda`, `tensorrt`).
        #[arg(long, default_value = "cpu")]
        backend: String,

        /// Dirección de red y puerto UDP del receptor OSC de VRChat.
        #[arg(short, long, default_value = "127.0.0.1:9000")]
        destino: SocketAddr,

        /// Frecuencia objetivo de reproducción en hercios (Hz).
        #[arg(long, default_value_t = 30)]
        hz: u32,

        /// Duración máxima en segundos (opcional).
        #[arg(long)]
        duracion_seg: Option<u64>,

        /// Reproduce en bucle infinito sobre el archivo de sesión grabado.
        #[arg(short, long, default_value_t = false)]
        bucle: bool,
    },

    /// Mide la latencia de inferencia y procesado del solver en el sistema.
    Bench {
        /// Cantidad de iteraciones a promediar.
        #[arg(short, long, default_value_t = 100)]
        iteraciones: usize,

        /// Ruta al modelo ONNX RTMPose Halpe26.
        #[arg(long, default_value = "modelos/rtmpose-m-halpe26.onnx")]
        modelo: PathBuf,

        /// Backend de inferencia (`cpu`, `cuda`, `tensorrt`).
        #[arg(long, default_value = "cpu")]
        backend: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("ferrite=info,warn")),
        )
        .init();

    let cli = Cli::parse();

    let cancelar = Arc::new(AtomicBool::new(false));
    let r = cancelar.clone();
    ctrlc::set_handler(move || {
        r.store(true, Ordering::SeqCst);
    })
    .context("Error al registrar el manejador de interrupcion Ctrl+C")?;

    match cli.comando {
        Comandos::Fake {
            destino,
            hz,
            duracion_seg,
        } => ejecutar_fake(destino, hz, duracion_seg, cancelar),

        Comandos::Run {
            sintetico,
            modelo,
            backend,
            destino,
            hz,
            duracion_seg,
        } => {
            if !sintetico {
                bail!(
                    "El controlador para hardware Kinect v2 fisico se conectara en la Fase 3 tras compilar libfreenect2.\n\
                     Para ejecutar el pipeline completo de seguimiento corporal (vision + solver + OSC) ahora mismo,\n\
                     agrega la bandera '--sintetico':\n\
                     cargo run -p ferrite -- run --sintetico\n\
                     o reproduce una sesion con:\n\
                     cargo run -p ferrite -- reproducir sesion.ferrite"
                );
            }

            let opciones = OpcionesReproducir {
                archivo: None,
                sintetico: true,
                ruta_modelo: modelo,
                backend_str: backend,
                destino,
                hz,
                duracion_seg,
                bucle: false,
            };
            ejecutar_reproducir(opciones, cancelar)
        }

        Comandos::Grabar {
            salida,
            segundos,
            sintetico,
            hz,
        } => ejecutar_grabar(salida, segundos, sintetico, hz, cancelar),

        Comandos::Reproducir {
            archivo,
            sintetico,
            modelo,
            backend,
            destino,
            hz,
            duracion_seg,
            bucle,
        } => {
            let opciones = OpcionesReproducir {
                archivo,
                sintetico,
                ruta_modelo: modelo,
                backend_str: backend,
                destino,
                hz,
                duracion_seg,
                bucle,
            };
            ejecutar_reproducir(opciones, cancelar)
        }

        Comandos::Bench {
            iteraciones,
            modelo,
            backend,
        } => {
            let opciones = OpcionesBench {
                iteraciones,
                ruta_modelo: modelo,
                backend_str: backend,
            };
            ejecutar_bench(opciones)
        }
    }
}

/// Ejecuta el bucle de emisión sintética sin deriva temporal.
fn ejecutar_fake(
    destino: SocketAddr,
    hz: u32,
    duracion_seg: Option<u64>,
    cancelar: Arc<AtomicBool>,
) -> Result<()> {
    if hz == 0 {
        bail!("La frecuencia de emision debe ser mayor a 0 Hz");
    }

    info!("Iniciando emisor simulado (modo fake) hacia {destino} a {hz} Hz...");

    let emisor = EmisorVrchat::nuevo(destino)
        .with_context(|| format!("No se pudo inicializar el socket UDP hacia {destino}"))?;

    let periodo = Duration::from_secs_f64(1.0 / hz as f64);
    let inicio = Instant::now();
    let duracion_max = duracion_seg.map(Duration::from_secs);
    let mut frame_indice: u64 = 0;

    info!("Transmitiendo telemetria OSC. Presione Ctrl+C para detener.");

    while !cancelar.load(Ordering::Relaxed) {
        if let Some(limite) = duracion_max {
            if inicio.elapsed() >= limite {
                info!("Duracion programada alcanzada ({limite:?}). Finalizando emision.");
                break;
            }
        }

        let tiempo_actual_seg = inicio.elapsed().as_secs_f64();
        let esqueleto = sintetico::generar_esqueleto_caminata(tiempo_actual_seg);
        let muestras = muestras_desde_esqueleto(&esqueleto);

        if let Err(error) = emisor.enviar_frame(&muestras) {
            warn!("Fallo al emitir cuadro OSC {frame_indice}: {error}");
        }

        frame_indice += 1;

        if frame_indice % (hz as u64 * 5) == 0 {
            info!(
                "Cuadros transmitidos: {frame_indice} (tiempo transcurrido: {:.1} s)",
                inicio.elapsed().as_secs_f64()
            );
        }

        // Ticker sin deriva temporal
        let proximo_instante = inicio + periodo.mul_f64(frame_indice as f64);
        let ahora = Instant::now();
        if proximo_instante > ahora {
            std::thread::sleep(proximo_instante - ahora);
        }
    }

    info!(
        "Emision detenida. Total de cuadros enviados: {frame_indice} en {:.2} s.",
        inicio.elapsed().as_secs_f64()
    );

    Ok(())
}
