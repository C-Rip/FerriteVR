//! Generador determinista de esqueletos 3D sintéticos para pruebas de telemetría.

use std::f32::consts::PI;

use ferrite_nucleo::{
    euler_zxy_grados_a_cuaternion, Articulacion, Esqueleto3D, PosUnity, PoseArticulacion,
};
use glam::{Quat, Vec3};

/// Frecuencia base de la caminata en ciclos completos por segundo (Hz).
const FRECUENCIA_CAMINATA_HZ: f32 = 1.0;

/// Genera un [`Esqueleto3D`] sintético determinista correspondiente a una caminata en el sitio.
///
/// Parámetros:
/// - `tiempo_seg`: Tiempo transcurrido en segundos (usado para calcular la fase de la marcha).
///
/// Características cinemáticas del esqueleto:
/// - **Cabeza:** situada a ~1.70 m con oscilación vertical suave.
/// - **Pecho:** situado a ~1.35 m con ligero balanceo compensatorio.
/// - **Cadera:** situada a ~1.00 m con balanceo lateral armónico.
/// - **Pies y Rodillas:** alternan en contrafase ($\pi$ radianes). La altura vertical de los pies
///   oscila periódicamente sin descender en ningún instante por debajo de `Y = 0.05` m.
/// - **Codos:** situados a los costados con braceo natural opuesto a las piernas.
pub fn generar_esqueleto_caminata(tiempo_seg: f64) -> Esqueleto3D {
    let t = tiempo_seg as f32;
    let omega = 2.0 * PI * FRECUENCIA_CAMINATA_HZ;
    let fase = omega * t;

    // Marca de tiempo en microsegundos
    let marca_tiempo_us = (tiempo_seg * 1_000_000.0) as u64;
    let mut esqueleto = Esqueleto3D::nuevo(marca_tiempo_us);

    // 1. Cadera (raíz): balanceo lateral a la frecuencia de paso, oscilación vertical doble
    let sway_x = 0.03 * fase.sin();
    let cadera_y = 1.00 + 0.02 * (2.0 * fase).cos();
    let cadera_yaw = 4.0 * fase.sin();
    let rot_cadera = euler_zxy_grados_a_cuaternion([0.0, cadera_yaw, 0.0]);
    esqueleto.insertar(
        Articulacion::Cadera,
        PoseArticulacion::nueva(PosUnity(Vec3::new(sway_x, cadera_y, 0.0)), rot_cadera, 1.0),
    );

    // 2. Cabeza: a ~1.70 m con seguimiento suave
    let cabeza_x = 0.015 * fase.sin();
    let cabeza_y = 1.70 + 0.015 * (2.0 * fase).cos();
    esqueleto.insertar(
        Articulacion::Cabeza,
        PoseArticulacion::nueva(
            PosUnity(Vec3::new(cabeza_x, cabeza_y, 0.02)),
            Quat::IDENTITY,
            1.0,
        ),
    );

    // 3. Pecho: a ~1.35 m
    let pecho_x = 0.02 * fase.sin();
    let pecho_y = 1.35 + 0.018 * (2.0 * fase).cos();
    let rot_pecho = euler_zxy_grados_a_cuaternion([0.0, -2.0 * fase.sin(), 0.0]);
    esqueleto.insertar(
        Articulacion::Pecho,
        PoseArticulacion::nueva(PosUnity(Vec3::new(pecho_x, pecho_y, 0.01)), rot_pecho, 1.0),
    );

    // 4. Pies: alternancia armónica en el plano sagital y elevación en fase de oscilación
    // Pie izquierdo (fase directa)
    let elevacion_izq = fase.sin().max(0.0);
    let pie_izq_y = 0.05 + 0.08 * elevacion_izq;
    let pie_izq_z = 0.15 * fase.cos();
    let rot_pie_izq = euler_zxy_grados_a_cuaternion([-10.0 * elevacion_izq, 0.0, 0.0]);
    esqueleto.insertar(
        Articulacion::PieIzq,
        PoseArticulacion::nueva(
            PosUnity(Vec3::new(-0.12, pie_izq_y, pie_izq_z)),
            rot_pie_izq,
            1.0,
        ),
    );

    // Pie derecho (desfasado en PI radianes)
    let fase_der = fase + PI;
    let elevacion_der = fase_der.sin().max(0.0);
    let pie_der_y = 0.05 + 0.08 * elevacion_der;
    let pie_der_z = 0.15 * fase_der.cos();
    let rot_pie_der = euler_zxy_grados_a_cuaternion([-10.0 * elevacion_der, 0.0, 0.0]);
    esqueleto.insertar(
        Articulacion::PieDer,
        PoseArticulacion::nueva(
            PosUnity(Vec3::new(0.12, pie_der_y, pie_der_z)),
            rot_pie_der,
            1.0,
        ),
    );

    // 5. Rodillas: posición anatómica coordinada con la pierna
    let rodilla_izq_y = 0.52 + 0.04 * elevacion_izq;
    let rodilla_izq_z = 0.08 + 0.10 * fase.cos();
    esqueleto.insertar(
        Articulacion::RodillaIzq,
        PoseArticulacion::nueva(
            PosUnity(Vec3::new(-0.11, rodilla_izq_y, rodilla_izq_z)),
            Quat::IDENTITY,
            1.0,
        ),
    );

    let rodilla_der_y = 0.52 + 0.04 * elevacion_der;
    let rodilla_der_z = 0.08 + 0.10 * fase_der.cos();
    esqueleto.insertar(
        Articulacion::RodillaDer,
        PoseArticulacion::nueva(
            PosUnity(Vec3::new(0.11, rodilla_der_y, rodilla_der_z)),
            Quat::IDENTITY,
            1.0,
        ),
    );

    // 6. Codos: laterales con braceo opuesto a la cadera/piernas
    let codo_izq_y = 1.15 + 0.02 * fase.cos();
    let codo_izq_z = -0.05 - 0.08 * fase.cos();
    esqueleto.insertar(
        Articulacion::CodoIzq,
        PoseArticulacion::nueva(
            PosUnity(Vec3::new(-0.28, codo_izq_y, codo_izq_z)),
            Quat::IDENTITY,
            1.0,
        ),
    );

    let codo_der_y = 1.15 - 0.02 * fase.cos();
    let codo_der_z = -0.05 + 0.08 * fase.cos();
    esqueleto.insertar(
        Articulacion::CodoDer,
        PoseArticulacion::nueva(
            PosUnity(Vec3::new(0.28, codo_der_y, codo_der_z)),
            Quat::IDENTITY,
            1.0,
        ),
    );

    esqueleto
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generador_es_determinista() {
        let t = 42.12345;
        let esq1 = generar_esqueleto_caminata(t);
        let esq2 = generar_esqueleto_caminata(t);

        assert_eq!(esq1.marca_tiempo_us, esq2.marca_tiempo_us);
        assert_eq!(esq1.articulaciones.len(), esq2.articulaciones.len());

        for (art, pose1) in &esq1.articulaciones {
            let pose2 = esq2.obtener(*art).expect("articulacion faltante en esq2");
            assert_eq!(pose1.posicion, pose2.posicion);
            assert_eq!(pose1.rotacion, pose2.rotacion);
            assert_eq!(pose1.confianza, pose2.confianza);
        }
    }

    #[test]
    fn test_pies_nunca_bajan_de_cero() {
        for i in 0..1000 {
            let t = i as f64 * 0.01;
            let esqueleto = generar_esqueleto_caminata(t);

            let pie_izq = esqueleto
                .obtener(Articulacion::PieIzq)
                .expect("falta pie izquierdo");
            let pie_der = esqueleto
                .obtener(Articulacion::PieDer)
                .expect("falta pie derecho");

            assert!(
                pie_izq.posicion.0.y >= 0.05,
                "Pie izquierdo bajo de 0.05 m en t={}: y={}",
                t,
                pie_izq.posicion.0.y
            );
            assert!(
                pie_der.posicion.0.y >= 0.05,
                "Pie derecho bajo de 0.05 m en t={}: y={}",
                t,
                pie_der.posicion.0.y
            );
        }
    }

    #[test]
    fn test_coordenadas_finitas_y_validas() {
        for i in 0..200 {
            let t = i as f64 * 0.05;
            let esqueleto = generar_esqueleto_caminata(t);

            for (art, pose) in &esqueleto.articulaciones {
                let pos = pose.posicion.0;
                assert!(
                    pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite(),
                    "Posicion no finita en {:?} para t={}",
                    art,
                    t
                );
                assert!(
                    pose.rotacion.is_finite(),
                    "Rotacion no finita en {:?} para t={}",
                    art,
                    t
                );
                assert!(
                    pose.confianza >= 0.0 && pose.confianza <= 1.0,
                    "Confianza fuera de rango en {:?} para t={}",
                    art,
                    t
                );
            }
        }
    }
}
