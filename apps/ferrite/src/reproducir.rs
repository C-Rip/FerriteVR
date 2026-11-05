//! Subcomando para reproducir sesiones grabadas en formato `.ferrite` o fotogramas sintéticos.

use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use ferrite_nucleo::IntrinsecosProfundidad;
use ferrite_osc::EmisorVrchat;
use ferrite_pose::{BackendInferencia, MotorPose};
use ferrite_sensor::{FuenteFrames, FuenteReproduccion, GeneradorSinteticoSensor};
use ferrite_solver::SolverCinematico;
use tracing::info;

use crate::pipeline::{ejecutar_pipeline, OpcionesPipeline};

/// Opciones de configuración para la reproducción del pipeline.
#[derive(Debug, Clone)]
pub struct OpcionesReproducir {
    /// Archivo `.ferrite` opcional a reproducir.
    pub archivo: Option<PathBuf>,
    /// Usar generador sintético si es verdadero.
    pub sintetico: bool,
    /// Ruta al modelo ONNX RTMPose Halpe26.
    pub ruta_modelo: PathBuf,
    /// Backend de inferencia (`cpu`, `cuda`, `tensorrt`).
    pub backend_str: String,
    /// Dirección del receptor OSC de VRChat.
    pub destino: SocketAddr,
    /// Frecuencia en hercios (Hz).
    pub hz: u32,
    /// Duración máxima en segundos (opcional).
    pub duracion_seg: Option<u64>,
    /// Bucle continuo sobre archivo de sesión.
    pub bucle: bool,
}

/// Ejecuta la reproducción y el pipeline de inferencia y solver hacia VRChat.
pub fn ejecutar_reproducir(opciones: OpcionesReproducir, cancelar: Arc<AtomicBool>) -> Result<()> {
    if opciones.hz == 0 {
        bail!("La frecuencia debe ser mayor a 0 Hz");
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

    let (fuente, intrinsecos): (Box<dyn FuenteFrames>, IntrinsecosProfundidad) =
        match (&opciones.archivo, opciones.sintetico) {
            (Some(ruta), _) => {
                let archivo = File::open(ruta).with_context(|| {
                    format!(
                        "No se pudo abrir el archivo de grabacion: {}",
                        ruta.display()
                    )
                })?;
                let lector = BufReader::new(archivo);
                let reproductor = FuenteReproduccion::abrir(lector)
                    .context("Error al procesar el encabezado del archivo .ferrite")?
                    .con_repeticion(opciones.bucle);

                let intr = reproductor.intrinsecos();
                info!(
                    archivo = %ruta.display(),
                    bucle = opciones.bucle,
                    ancho = intr.ancho,
                    alto = intr.alto,
                    "Reproduciendo archivo de sesion .ferrite"
                );
                (Box::new(reproductor), intr)
            }
            (None, true) => {
                let intr = IntrinsecosProfundidad::default();
                let generador = GeneradorSinteticoSensor::nuevo(intr, opciones.hz as f32);
                info!(
                    hz = opciones.hz,
                    "Reproduciendo fotogramas desde generador sintetico humanoide"
                );
                (Box::new(generador), intr)
            }
            (None, false) => {
                bail!(
                    "Debes especificar la ruta de un archivo '.ferrite' para reproducir o usar la bandera '--sintetico'.\n\
                     Ejemplo:\n\
                     cargo run -p ferrite -- reproducir sesion.ferrite\n\
                     cargo run -p ferrite -- reproducir --sintetico"
                );
            }
        };

    info!(
        modelo = %opciones.ruta_modelo.display(),
        backend = ?backend,
        "Cargando motor de inferencia RTMPose..."
    );
    let motor_pose = MotorPose::nuevo(&opciones.ruta_modelo, backend)
        .context("Fallo al inicializar el motor de inferencia ONNX")?;

    let solver = SolverCinematico::nuevo(intrinsecos);

    let emisor = EmisorVrchat::nuevo(opciones.destino).with_context(|| {
        format!(
            "No se pudo inicializar el emisor OSC hacia {}",
            opciones.destino
        )
    })?;

    let pipeline_opts = OpcionesPipeline {
        hz: opciones.hz,
        duracion_seg: opciones.duracion_seg,
        sintetico: opciones.sintetico,
    };

    ejecutar_pipeline(fuente, motor_pose, solver, emisor, pipeline_opts, cancelar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ejecutar_reproducir_sintetico_corto() {
        let ruta_modelo = PathBuf::from("../../modelos/rtmpose-m-halpe26.onnx");
        let ruta_modelo = if ruta_modelo.exists() {
            ruta_modelo
        } else {
            PathBuf::from("modelos/rtmpose-m-halpe26.onnx")
        };

        if !ruta_modelo.exists() {
            // Si no se ha descargado el modelo en este entorno, omitir
            return;
        }

        let opciones = OpcionesReproducir {
            archivo: None,
            sintetico: true,
            ruta_modelo,
            backend_str: "cpu".to_string(),
            destino: "127.0.0.1:19876".parse().unwrap(),
            hz: 30,
            duracion_seg: Some(1),
            bucle: false,
        };

        let cancelar = Arc::new(AtomicBool::new(false));
        let res = ejecutar_reproducir(opciones, cancelar);
        assert!(res.is_ok());
    }
}
