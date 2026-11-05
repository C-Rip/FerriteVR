//! Motor de inferencia ONNX para estimación de pose 2D con RTMPose-m.

use std::path::Path;
use ferrite_sensor::FrameSensor;
use ort::session::Session;
use ort::value::Tensor;
use crate::bbox::CajaDelimitadora;
use crate::preprocesado::{PreprocesadorPose, ALTO_MODELO, ANCHO_MODELO, ELEMENTOS_NCHW};
use crate::simcc::{decodificar_simcc, Pose2D};
use crate::ErrorPose;

/// Backend de cómputo para la inferencia de redes neuronales con ONNX Runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendInferencia {
    /// Inferencia en procesador central (CPU) multihilo.
    #[default]
    Cpu,
    /// Aceleración por hardware NVIDIA CUDA.
    Cuda,
    /// Aceleración de alto rendimiento NVIDIA TensorRT.
    TensorRt,
}

impl std::str::FromStr for BackendInferencia {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "cpu" => Ok(Self::Cpu),
            "cuda" => Ok(Self::Cuda),
            "tensorrt" | "trt" => Ok(Self::TensorRt),
            otro => Err(format!(
                "Backend desconocido: '{otro}'. Opciones: 'cpu', 'cuda', 'tensorrt'"
            )),
        }
    }
}

/// Motor de inferencia de pose 2D basado en RTMPose-m (Halpe26).
pub struct MotorPose {
    session: Session,
    preprocesador: PreprocesadorPose,
    backend: BackendInferencia,
    buffer_entrada: Box<[f32]>,
}

impl MotorPose {
    /// Carga el modelo ONNX desde la ruta especificada e inicializa la sesión de inferencia.
    pub fn nuevo(
        ruta_modelo: impl AsRef<Path>,
        backend: BackendInferencia,
    ) -> Result<Self, ErrorPose> {
        let ruta = ruta_modelo.as_ref();
        if !ruta.exists() {
            return Err(ErrorPose::Inferencia(format!(
                "El archivo del modelo ONNX no existe en: {}",
                ruta.display()
            )));
        }

        let builder = Session::builder()
            .map_err(|e| ErrorPose::Inferencia(format!("Fallo al crear constructor ort: {e}")))?;

        // Configurar sesión optimizada para baja latencia en CPU
        let builder = builder
            .with_intra_threads(4)
            .map_err(|e| ErrorPose::Inferencia(format!("Fallo al configurar hilos: {e}")))?;

        let session = builder
            .commit_from_file(ruta)
            .map_err(|e| ErrorPose::Inferencia(format!("Fallo al cargar modelo ONNX: {e}")))?;

        Ok(Self {
            session,
            preprocesador: PreprocesadorPose::default(),
            backend,
            buffer_entrada: vec![0.0f32; ELEMENTOS_NCHW].into_boxed_slice(),
        })
    }

    /// Retorna el backend de cómputo actualmente configurado.
    #[inline]
    pub fn backend(&self) -> BackendInferencia {
        self.backend
    }

    /// Ejecuta la inferencia sobre el fotograma suministrado y retorna los 26 puntos clave 2D.
    pub fn estimar(&mut self, frame: &FrameSensor) -> Result<Pose2D, ErrorPose> {
        let bbox = self.preprocesador.estimar_bbox(frame).unwrap_or_else(|| {
            CajaDelimitadora::nueva(
                0.0,
                0.0,
                (frame.ancho - 1) as f32,
                (frame.alto - 1) as f32,
            )
            .ajustar_proporcion(
                (ANCHO_MODELO as f32) / (ALTO_MODELO as f32),
                (frame.ancho - 1) as f32,
                (frame.alto - 1) as f32,
            )
        });

        // Preprocesar imagen en tensor continuo NCHW
        let tensor_slice = self.preprocesador.preprocesar(frame, Some(bbox))?;
        self.buffer_entrada.copy_from_slice(tensor_slice);

        let forma_entrada = [1usize, 3, ALTO_MODELO as usize, ANCHO_MODELO as usize];
        let tensor_entrada =
            Tensor::from_array((forma_entrada, self.buffer_entrada.clone()))
                .map_err(|e| ErrorPose::Inferencia(format!("Error al crear tensor de entrada: {e}")))?;

        let salidas = self
            .session
            .run(ort::inputs!["input" => tensor_entrada])
            .map_err(|e| ErrorPose::Inferencia(format!("Error en inferencia ONNX: {e}")))?;

        let mut simcc_x_opt: Option<Vec<f32>> = None;
        let mut simcc_y_opt: Option<Vec<f32>> = None;

        for (nombre, valor) in salidas {
            let (_, datos) = valor
                .try_extract_tensor::<f32>()
                .map_err(|e| ErrorPose::Inferencia(format!("Error al extraer salida '{nombre}': {e}")))?;

            if nombre == "simcc_x" {
                simcc_x_opt = Some(datos.to_vec());
            } else if nombre == "simcc_y" {
                simcc_y_opt = Some(datos.to_vec());
            }
        }

        let simcc_x = simcc_x_opt.ok_or_else(|| {
            ErrorPose::Inferencia("Salida 'simcc_x' no encontrada en el modelo".into())
        })?;
        let simcc_y = simcc_y_opt.ok_or_else(|| {
            ErrorPose::Inferencia("Salida 'simcc_y' no encontrada en el modelo".into())
        })?;

        let puntos = decodificar_simcc(&simcc_x, &simcc_y, &bbox);

        Ok(Pose2D { puntos, bbox })
    }
}
