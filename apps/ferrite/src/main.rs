//! Binario CLI principal de FerriteVR.
//!
//! Proporciona las herramientas de línea de comandos para captura de sensor,
//! inferencia de pose, emisión de telemetría OSC hacia VRChat y simulación sintética.

#![deny(non_ascii_idents)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

mod sintetico;

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use ferrite_osc::{muestras_desde_esqueleto, EmisorVrchat};
use tracing::{info, warn};

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
    /// Genera y emite un esqueleto sintético de caminata para pruebas en VRChat sin sensor físico.
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
    Run,

    /// Graba los flujos de profundidad y color en un archivo binario `.ferrite`.
    Grabar,

    /// Reproduce una sesión grabada previamente para depuración y pruebas sin sensor.
    Reproducir,

    /// Mide la latencia de inferencia y procesado entre los distintos backends de cómputo.
    Bench,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.comando {
        Comandos::Fake {
            destino,
            hz,
            duracion_seg,
        } => ejecutar_fake(destino, hz, duracion_seg),
        Comandos::Run => {
            bail!("El subcomando 'run' aun no esta implementado (se completara en la Fase 6)")
        }
        Comandos::Grabar => {
            bail!("El subcomando 'grabar' aun no esta implementado (se completara en el WP-3.3)")
        }
        Comandos::Reproducir => {
            bail!("El subcomando 'reproducir' aun no esta implementado (se completara en el WP-3.3)")
        }
        Comandos::Bench => {
            bail!("El subcomando 'bench' aun no esta implementado (se completara en el WP-4.4)")
        }
    }
}

/// Ejecuta el bucle de emisión sintética sin deriva temporal.
fn ejecutar_fake(destino: SocketAddr, hz: u32, duracion_seg: Option<u64>) -> Result<()> {
    if hz == 0 {
        bail!("La frecuencia de emision debe ser mayor a 0 Hz");
    }

    info!(
        "Iniciando emisor simulado (modo fake) hacia {destino} a {hz} Hz..."
    );

    let emisor = EmisorVrchat::nuevo(destino)
        .with_context(|| format!("No se pudo inicializar el socket UDP hacia {destino}"))?;

    let ejecutando = Arc::new(AtomicBool::new(true));
    let r = ejecutando.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error al registrar el manejador de interrupcion Ctrl+C")?;

    let periodo = Duration::from_secs_f64(1.0 / hz as f64);
    let inicio = Instant::now();
    let duracion_max = duracion_seg.map(Duration::from_secs);
    let mut frame_indice: u64 = 0;

    info!("Transmitiendo telemetria OSC. Presione Ctrl+C para detener.");

    while ejecutando.load(Ordering::Relaxed) {
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
