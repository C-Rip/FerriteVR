//! Solver cinemático 3D, desproyección, filtrado One Euro y cálculo de rotaciones para FerriteVR.
//!
//! Este crate proporciona:
//! - Elevación tridimensional y desproyección estenopeica con muestreo de mediana $5 \times 5$ en [`elevacion`].
//! - Filtro adaptativo temporal One Euro para posiciones vectoriales y rotaciones esféricas en [`filtro`].
//! - Cálculo determinista de bases ortonormales anatómicas en [`rotaciones`].
//! - Orquestación completa del pipeline cinemático en [`solver`].

#![deny(non_ascii_idents)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod elevacion;
pub mod filtro;
pub mod rotaciones;
pub mod solver;

pub use elevacion::{Elevador3D, EsqueletoElevado, Punto3D};
pub use filtro::{FiltroOneEuro, FiltroOneEuroVec3, FiltroRotacion};
pub use rotaciones::{
    calcular_rotacion_cadera, calcular_rotacion_pie_der, calcular_rotacion_pie_izq,
    construir_rotacion_de_ejes,
};
pub use solver::SolverCinematico;

/// Errores producidos durante la resolución cinemática tridimensional.
#[derive(Debug, thiserror::Error)]
pub enum ErrorSolver {
    /// Parámetros intrínsecos de la cámara no válidos.
    #[error("Parámetros intrínsecos de cámara inválidos: {0}")]
    IntrinsecosInvalidos(String),

    /// Datos de profundidad insuficientes para reconstruir el esqueleto.
    #[error("Datos de profundidad insuficientes para elevar la pose a 3D")]
    ProfundidadInsuficiente,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_nucleo::{Articulacion, IntrinsecosProfundidad};
    use ferrite_pose::simcc::{NUM_ARTICULACIONES_HALPE, Pose2D, PuntoClave2D};
    use ferrite_pose::CajaDelimitadora;
    use ferrite_sensor::GeneradorSinteticoSensor;
    use glam::{Quat, Vec3};

    #[test]
    fn test_filtro_one_euro_reduccion_jitter_en_reposo() {
        let mut filtro = FiltroOneEuro::nuevo(1.0, 0.007, 1.0);
        let valor_base = 2.0f32;
        let dt_us = 33_333; // 30 FPS (~33.3 ms)

        let mut suma_cuad_cruda = 0.0f32;
        let mut suma_cuad_filtrada = 0.0f32;

        for i in 0..100 {
            let t_us = (i as u64) * dt_us;
            // Ruido oscilatorio de alta frecuencia (simulando jitter de sensor)
            let ruido = if i % 2 == 0 { 0.05 } else { -0.05 };
            let valor_ruidoso = valor_base + ruido;
            let filtrado = filtro.filtrar(valor_ruidoso, t_us);

            if i > 20 {
                // Evaluamos tras estabilizar el estado inicial
                suma_cuad_cruda += (valor_ruidoso - valor_base).powi(2);
                suma_cuad_filtrada += (filtrado - valor_base).powi(2);
            }
        }

        // El jitter en reposo debe reducirse significativamente (al menos 70%)
        let reduccion_jitter = 1.0 - (suma_cuad_filtrada / suma_cuad_cruda);
        assert!(
            reduccion_jitter >= 0.70,
            "Reducción de jitter insuficiente: {:.2}%",
            reduccion_jitter * 100.0
        );
    }

    #[test]
    fn test_filtro_one_euro_baja_latencia_escalon() {
        let mut filtro = FiltroOneEuro::nuevo(1.0, 0.05, 1.0);
        let dt_us = 33_333;

        // Estabilizar en 0.0
        for i in 0..10 {
            filtro.filtrar(0.0, (i as u64) * dt_us);
        }

        // Salto tipo escalón a 10.0 (movimiento rápido)
        let t_salto = 10 * dt_us;
        let f1 = filtro.filtrar(10.0, t_salto);
        let f2 = filtro.filtrar(10.0, t_salto + dt_us);
        let f3 = filtro.filtrar(10.0, t_salto + 2 * dt_us);

        // Debido al parámetro beta ante velocidad, el filtro se adapta rápidamente alcanzando >70% en 1 fotograma adicional
        assert!(f1 > 4.0, "f1 debe responder de inmediato: f1={}", f1);
        assert!(f2 > 7.0, "f2 debe superar el 70% del escalón: f2={}", f2);
        assert!(f3 > 8.5, "f3 debe superar el 85% del escalón: f3={}", f3);
    }

    #[test]
    fn test_elevador_3d_ventana_mediana_y_desproyeccion() {
        let mut elevador = Elevador3D::nuevo(2, 500.0, 4500.0);
        let mut profundidad = vec![3000.0f32; 512 * 424];

        // Insertamos un valor atípico de ruido en el centro
        profundidad[212 * 512 + 256] = 5000.0; // Fuera de rango o pared

        let z_med = elevador
            .estimar_profundidad_mediana(256.0, 212.0, &profundidad, 512, 424)
            .expect("Debe retornar profundidad mediana");

        // La mediana en ventana 5x5 debe ignorar el valor atípico y retornar 3000.0
        assert_eq!(z_med, 3000.0);
    }

    #[test]
    fn test_rotaciones_cuaterniones_normalizados_y_casos_degenerados() {
        // Caso normal: X hacia la derecha, Y hacia arriba
        let q = construir_rotacion_de_ejes(Vec3::X, Vec3::Y);
        assert!(q.is_normalized());
        assert!((q.length() - 1.0).abs() < 1e-5);

        // Caso degenerado: vectores colineales
        let q_colineal = construir_rotacion_de_ejes(Vec3::X, Vec3::X);
        assert!(!q_colineal.is_nan());
        assert_eq!(q_colineal, Quat::IDENTITY);

        // Caso degenerado: vector nulo
        let q_cero = construir_rotacion_de_ejes(Vec3::ZERO, Vec3::Y);
        assert!(!q_cero.is_nan());
        assert_eq!(q_cero, Quat::IDENTITY);
    }

    #[test]
    fn test_solver_cinematico_pipeline_completo() {
        let generador = GeneradorSinteticoSensor::default();
        let frame = generador.generar_fotograma(0);

        let mut solver = SolverCinematico::nuevo(IntrinsecosProfundidad::default());

        // Crear una pose 2D sintética básica
        let mut pose_2d = Pose2D {
            puntos: [PuntoClave2D::default(); NUM_ARTICULACIONES_HALPE],
            bbox: CajaDelimitadora::nueva(100.0, 50.0, 400.0, 400.0),
        };

        // Asignar puntos clave con confianza en el centro del sujeto sintético
        for k in 0..NUM_ARTICULACIONES_HALPE {
            pose_2d.puntos[k] = PuntoClave2D {
                u: 256.0,
                v: 212.0,
                confianza: 0.9,
            };
        }

        let esqueleto_3d = solver
            .resolver(&pose_2d, &frame)
            .expect("Resolución 3D exitosa");

        assert!(esqueleto_3d.articulaciones.len() > 0);

        // Comprobar que todas las articulaciones generadas tienen valores finitos
        for (art, pose) in &esqueleto_3d.articulaciones {
            assert!(pose.posicion.0.is_finite(), "Posición no finita en {:?}", art);
            assert!(pose.rotacion.is_normalized(), "Rotación no normalizada en {:?}", art);
            assert!(!pose.rotacion.is_nan(), "Rotación NaN en {:?}", art);
            assert!(pose.confianza >= 0.0 && pose.confianza <= 1.0);
        }

        // Comprobar presencia de articulaciones clave para VRChat
        assert!(esqueleto_3d.obtener(Articulacion::Cadera).is_some());
        assert!(esqueleto_3d.obtener(Articulacion::Cabeza).is_some());
        assert!(esqueleto_3d.obtener(Articulacion::PieIzq).is_some());
        assert!(esqueleto_3d.obtener(Articulacion::PieDer).is_some());
    }
}
