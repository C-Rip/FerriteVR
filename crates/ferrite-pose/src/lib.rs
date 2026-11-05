//! Estimación de pose 2D e inferencia con RTMPose para FerriteVR.
//!
//! Este crate proporciona:
//! - Detección de cajas delimitadoras (*bounding boxes*) de la persona mediante segmentación por profundidad en [`bbox`].
//! - Preprocesamiento afin, escalado e interpolación bilineal normalizada a tensores NCHW (256×192) en [`preprocesado`].
//! - Decodificación de salidas SimCC a coordenadas de píxel y confianza estadística en [`simcc`].
//! - Motor de inferencia ONNX de alta velocidad multihilo en [`motor`].

#![deny(non_ascii_idents)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod bbox;
pub mod motor;
pub mod preprocesado;
pub mod simcc;

pub use bbox::{estimar_bbox_profundidad, CajaDelimitadora};
pub use motor::{BackendInferencia, MotorPose};
pub use preprocesado::{PreprocesadorPose, ALTO_MODELO, ANCHO_MODELO, ELEMENTOS_NCHW};
pub use simcc::{
    decodificar_simcc, generar_pose_2d_sintetica, Pose2D, PuntoClave2D, BINS_SIMCC_X, BINS_SIMCC_Y,
    NUM_ARTICULACIONES_HALPE, SIMCC_SPLIT_RATIO,
};

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
    use std::path::Path;

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
            assert!(val > -4.0 && val < 4.0);
        }
    }

    #[test]
    fn test_reproyeccion_centro_bbox() {
        let preprocesador = PreprocesadorPose::default();
        let bbox = CajaDelimitadora::nueva(100.0, 50.0, 300.0, 400.0);

        let (u_orig, v_orig) = preprocesador.reproyectar_a_frame(
            (ANCHO_MODELO as f32) * 0.5,
            (ALTO_MODELO as f32) * 0.5,
            &bbox,
        );
        let (u_esp, v_esp) = bbox.centro();
        assert!((u_orig - u_esp).abs() < 1e-4);
        assert!((v_orig - v_esp).abs() < 1e-4);
    }

    #[test]
    fn test_decodificacion_simcc_valores_sinteticos() {
        let mut simcc_x = vec![0.0f32; NUM_ARTICULACIONES_HALPE * BINS_SIMCC_X];
        let mut simcc_y = vec![0.0f32; NUM_ARTICULACIONES_HALPE * BINS_SIMCC_Y];

        // Articulación 0 con pico en bin X = 192 (medio de 384) e Y = 256 (medio de 512)
        simcc_x[192] = 0.95;
        simcc_y[256] = 0.90;

        let bbox = CajaDelimitadora::nueva(0.0, 0.0, 192.0, 256.0);
        let puntos = decodificar_simcc(&simcc_x, &simcc_y, &bbox);

        // Bin 192 / 2.0 = 96.0; Bin 256 / 2.0 = 128.0
        assert!((puntos[0].u - 96.0).abs() < 1e-4);
        assert!((puntos[0].v - 128.0).abs() < 1e-4);
        assert!((puntos[0].confianza - 0.90).abs() < 1e-4);
    }

    #[test]
    fn test_motor_pose_inferencia_completa_con_modelo() {
        let ruta_modelo = Path::new("../../modelos/rtmpose-m-halpe26.onnx");
        if !ruta_modelo.exists() {
            println!("Omitiendo test de motor: modelo ONNX no encontrado en la ruta esperada");
            return;
        }

        let mut motor =
            MotorPose::nuevo(ruta_modelo, BackendInferencia::Cpu).expect("Cargar modelo RTMPose");
        assert_eq!(motor.backend(), BackendInferencia::Cpu);

        let generador = GeneradorSinteticoSensor::default();
        let frame = generador.generar_fotograma(0);

        let pose = motor.estimar(&frame).expect("Inferencia exitosa");
        assert_eq!(pose.puntos.len(), NUM_ARTICULACIONES_HALPE);

        for punto in &pose.puntos {
            assert!(punto.u.is_finite());
            assert!(punto.v.is_finite());
            assert!(punto.confianza >= 0.0 && punto.confianza <= 1.0);
        }

        // Cabeza (articulación 17) debe estar arriba de los tobillos (15 y 16)
        let cabeza_v = pose.puntos[17].v;
        let tobillo_izq_v = pose.puntos[15].v;
        let tobillo_der_v = pose.puntos[16].v;

        assert!(cabeza_v < tobillo_izq_v);
        assert!(cabeza_v < tobillo_der_v);
    }
}
