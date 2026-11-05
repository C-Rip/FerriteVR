//! Estimación de pose 2D e inferencia con RTMPose para FerriteVR.
//!
//! Este crate proporciona:
//! - Detección de cajas delimitadoras (*bounding boxes*) de la persona mediante segmentación por profundidad en [`bbox`].
//! - Preprocesamiento afin, escalado e interpolación bilineal normalizada a tensores NCHW (256×192) en [`preprocesado`].
//! - Reproyección de coordenadas estimadas hacia el espacio original de captura.

#![deny(non_ascii_idents)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod bbox;
pub mod preprocesado;

pub use bbox::{estimar_bbox_profundidad, CajaDelimitadora};
pub use preprocesado::{PreprocesadorPose, ALTO_MODELO, ANCHO_MODELO, ELEMENTOS_NCHW};

/// Errores producidos durante el preprocesamiento y la estimación de pose.
#[derive(Debug, thiserror::Error)]
pub enum ErrorPose {
    /// Las dimensiones del fotograma suministrado son inválidas o nulas.
    #[error("Dimensiones de fotograma invalidas: {0}x{1}")]
    DimensionesInvalidas(u32, u32),

    /// No se detectó ninguna persona o sujeto dentro del rango de profundidad configurado.
    #[error("No se detecto ningun sujeto en el rango de profundidad")]
    SujetoNoDetectado,

    /// Error durante la ejecución del grafo de inferencia ONNX.
    #[error("Error de inferencia ONNX: {0}")]
    Inferencia(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_sensor::GeneradorSinteticoSensor;

    #[test]
    fn test_caja_delimitadora_geometria_y_margen() {
        let caja = CajaDelimitadora::nueva(10.0, 20.0, 50.0, 100.0);
        assert_eq!(caja.ancho(), 40.0);
        assert_eq!(caja.alto(), 80.0);
        assert_eq!(caja.centro(), (30.0, 60.0));
        assert!(caja.contiene(30.0, 60.0));
        assert!(!caja.contiene(5.0, 60.0));

        let expandida = caja.con_margen(0.20, 200.0, 200.0);
        assert!(expandida.ancho() > caja.ancho());
        assert!(expandida.alto() > caja.alto());
        assert_eq!(expandida.centro(), caja.centro());
    }

    #[test]
    fn test_estimacion_bbox_sobre_sujeto_sintetico() {
        let generador = GeneradorSinteticoSensor::default();
        let frame = generador.generar_fotograma(0);

        let preprocesador = PreprocesadorPose::default();
        let bbox = preprocesador
            .estimar_bbox(&frame)
            .expect("Debe detectar al sujeto sintetico");

        // El generador sintetico crea al sujeto centrado en (256, 212)
        assert!(bbox.contiene(256.0, 212.0));
        // Debe abarcar tanto cabeza como piernas
        assert!(bbox.v_min < 150.0);
        assert!(bbox.v_max > 300.0);
        // La proporcion debe ser aproximadamente 0.75 (192 / 256)
        let proporcion = bbox.ancho() / bbox.alto();
        assert!((proporcion - 0.75).abs() < 0.05);
    }

    #[test]
    fn test_preprocesador_nchw_dimensiones_y_valores_finitos() {
        let generador = GeneradorSinteticoSensor::default();
        let frame = generador.generar_fotograma(0);

        let mut preprocesador = PreprocesadorPose::default();
        let bbox = preprocesador.estimar_bbox(&frame);
        let tensor = preprocesador
            .preprocesar(&frame, bbox)
            .expect("Preprocesamiento exitoso");

        assert_eq!(tensor.len(), ELEMENTOS_NCHW);
        for &val in tensor {
            assert!(val.is_finite());
            // Valores normalizados típicamente caen entre -3.0 y 3.0
            assert!(val > -4.0 && val < 4.0);
        }
    }

    #[test]
    fn test_reproyeccion_centro_bbox() {
        let preprocesador = PreprocesadorPose::default();
        let bbox = CajaDelimitadora::nueva(100.0, 50.0, 300.0, 400.0);

        // El centro del tensor (96, 128) debe reproyectarse exactamente al centro de la bbox
        let (u_orig, v_orig) = preprocesador.reproyectar_a_frame(
            (ANCHO_MODELO as f32) * 0.5,
            (ALTO_MODELO as f32) * 0.5,
            &bbox,
        );
        let (u_esp, v_esp) = bbox.centro();
        assert!((u_orig - u_esp).abs() < 1e-4);
        assert!((v_orig - v_esp).abs() < 1e-4);
    }
}
