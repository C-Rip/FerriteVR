//! Mitigación de oclusiones y persistencia cinemática para FerriteVR.
//!
//! Cuando una articulación queda ocluida brevemente (por cruce de extremidades, giro o pérdida momentánea
//! de profundidad), este módulo mantiene la última pose válida estimada y aplica un decaimiento exponencial
//! sobre su confianza. Si la oclusión persiste más de 400 ms, la articulación decae suavemente a cero.

use crate::elevacion::Punto3D;
use ferrite_nucleo::{Articulacion, Esqueleto3D, PoseArticulacion};
use std::collections::HashMap;

/// Gestor de oclusiones con memoria temporal y decaimiento exponencial de confianza.
pub struct GestorOclusion {
    /// Constante de tiempo tau para decaimiento exponencial en microsegundos (por defecto 250_000 µs = 250 ms).
    tau_us: f32,
    /// Umbral mínimo de confianza bajo el cual se considera la articulación totalmente perdida.
    umbral_minimo: f32,
    /// Últimas poses válidas observadas por articulación junto con su marca de tiempo en microsegundos.
    historial: HashMap<Articulacion, (PoseArticulacion, u64)>,
}

impl Default for GestorOclusion {
    fn default() -> Self {
        Self {
            tau_us: 250_000.0,
            umbral_minimo: 0.05,
            historial: HashMap::new(),
        }
    }
}

impl GestorOclusion {
    /// Crea un nuevo gestor de oclusión con el tiempo de persistencia en milisegundos especificado.
    pub fn nuevo(persistencia_ms: f32) -> Self {
        Self {
            tau_us: (persistencia_ms * 1000.0).max(50_000.0),
            umbral_minimo: 0.05,
            historial: HashMap::new(),
        }
    }

    /// Actualiza el esqueleto aplicando retención con decaimiento suave sobre articulaciones ocluidas.
    pub fn aplicar(&mut self, esqueleto: &mut Esqueleto3D, marca_tiempo_us: u64) {
        let articulaciones_criticas = [
            Articulacion::Cabeza,
            Articulacion::Pecho,
            Articulacion::Cadera,
            Articulacion::PieIzq,
            Articulacion::PieDer,
            Articulacion::RodillaIzq,
            Articulacion::RodillaDer,
            Articulacion::CodoIzq,
            Articulacion::CodoDer,
        ];

        for art in articulaciones_criticas {
            let pose_actual_opt = esqueleto.obtener(art).copied();

            match pose_actual_opt {
                Some(pose_actual) if pose_actual.confianza >= 0.15 => {
                    // Detección válida y confiable: actualizar memoria
                    self.historial.insert(art, (pose_actual, marca_tiempo_us));
                }
                _ => {
                    // Articulación ocluida o con confianza insuficiente: verificar si podemos retenerla
                    if let Some((ultima_pose, t_ultima)) = self.historial.get(&art) {
                        let dt_us = marca_tiempo_us.saturating_sub(*t_ultima);
                        let factor_decaimiento = (-((dt_us as f32) / self.tau_us)).exp();
                        let confianza_retenida = ultima_pose.confianza * factor_decaimiento;

                        if confianza_retenida > self.umbral_minimo {
                            let mut pose_retenida = *ultima_pose;
                            pose_retenida.confianza = confianza_retenida;
                            esqueleto.insertar(art, pose_retenida);
                        } else {
                            // Se superó el tiempo máximo de persistencia: eliminar del esqueleto
                            esqueleto.articulaciones.retain(|(a, _)| *a != art);
                        }
                    }
                }
            }
        }
    }

    /// Determina si el sujeto se encuentra orientado de espaldas al sensor según los puntos faciales y hombros.
    pub fn detectar_espaldas(puntos_halpe: &[Punto3D; 26]) -> bool {
        // En Halpe26:
        // 0: Nariz, 1: OjoIzq, 2: OjoDer, 3: OrejaIzq, 4: OrejaDer, 5: HombroIzq, 6: HombroDer
        let nariz_conf = puntos_halpe[0].confianza;
        let ojo_izq_conf = puntos_halpe[1].confianza;
        let ojo_der_conf = puntos_halpe[2].confianza;
        let hombro_izq_conf = puntos_halpe[5].confianza;
        let hombro_der_conf = puntos_halpe[6].confianza;

        let cara_visible = nariz_conf > 0.3 || (ojo_izq_conf > 0.3 && ojo_der_conf > 0.3);
        let hombros_visibles = hombro_izq_conf > 0.3 && hombro_der_conf > 0.3;

        // Si los hombros se detectan claramente pero la cara no es visible, está de espaldas
        !cara_visible && hombros_visibles
    }

    /// Reinicia el historial de oclusiones almacenado.
    pub fn reiniciar(&mut self) {
        self.historial.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_nucleo::PosUnity;
    use glam::{Quat, Vec3};

    #[test]
    fn test_oclusion_retencion_y_decaimiento() {
        let mut gestor = GestorOclusion::nuevo(200.0); // 200 ms tau
        let mut esqueleto = Esqueleto3D::nuevo(0);

        // Frame inicial: articulación visible con confianza 0.90
        esqueleto.insertar(
            Articulacion::PieIzq,
            PoseArticulacion::nueva(PosUnity(Vec3::new(0.0, 0.1, 0.0)), Quat::IDENTITY, 0.90),
        );
        gestor.aplicar(&mut esqueleto, 0);
        assert_eq!(
            esqueleto.obtener(Articulacion::PieIzq).unwrap().confianza,
            0.90
        );

        // Frame tras 100 ms: articulación desaparece del sensor
        let mut esqueleto_ocluido = Esqueleto3D::nuevo(100_000);
        gestor.aplicar(&mut esqueleto_ocluido, 100_000);

        // Debe mantenerse gracias a la retención pero con confianza atenuada
        let retenida = esqueleto_ocluido.obtener(Articulacion::PieIzq);
        assert!(retenida.is_some());
        let conf_100ms = retenida.unwrap().confianza;
        assert!(conf_100ms < 0.90 && conf_100ms > 0.30);

        // Frame tras 2000 ms (2 segundos de oclusión continua): debe desaparecer completamente
        let mut esqueleto_perdido = Esqueleto3D::nuevo(2_000_000);
        gestor.aplicar(&mut esqueleto_perdido, 2_000_000);
        assert!(esqueleto_perdido.obtener(Articulacion::PieIzq).is_none());
    }
}
