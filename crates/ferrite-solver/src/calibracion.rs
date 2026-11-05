//! Calibración y transformación de espacio de coordenadas entre sensor y VRChat.
//!
//! En el espacio del sensor Kinect, el origen `(0, 0, 0)` es el centro óptico de la lente de la cámara.
//! En VRChat / SteamVR, el origen de juego sitúa el suelo en `Y = 0` y al usuario centrado.
//! [`CalibracionEspacio`] efectúa la rotación de desinclinación (*un-tilt*), la elevación desde el piso
//! y el recentrado espacial.

use ferrite_nucleo::{Articulacion, Esqueleto3D, PosUnity, PoseArticulacion};
use glam::{Mat3, Quat, Vec3};

/// Parámetros de calibración para transformar posiciones del sensor al espacio de juego de VRChat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalibracionEspacio {
    /// Altura del sensor respecto al plano del suelo físico en metros (por defecto 1.10 m).
    pub altura_sensor_m: f32,
    /// Inclinación del sensor hacia abajo (pitch) en grados sexagesimales (por defecto 0.0°).
    pub inclinacion_grados: f32,
    /// Distancia media de trabajo en metros para centrar el eje Z en el origen de VRChat (por defecto 2.0 m).
    pub distancia_referencia_z: f32,
    /// Cota mínima vertical del suelo en metros para evitar que los pies atraviesen el piso (por defecto 0.05 m).
    pub cota_suelo_min_m: f32,
}

impl Default for CalibracionEspacio {
    fn default() -> Self {
        Self {
            altura_sensor_m: 1.10,
            inclinacion_grados: 0.0,
            distancia_referencia_z: 2.0,
            cota_suelo_min_m: 0.05,
        }
    }
}

impl CalibracionEspacio {
    /// Crea una nueva configuración de calibración con los parámetros especificados.
    pub fn nueva(
        altura_sensor_m: f32,
        inclinacion_grados: f32,
        distancia_referencia_z: f32,
    ) -> Self {
        Self {
            altura_sensor_m: altura_sensor_m.max(0.2),
            inclinacion_grados,
            distancia_referencia_z: distancia_referencia_z.max(0.5),
            cota_suelo_min_m: 0.05,
        }
    }

    /// Transforma un punto en espacio métrico de cámara [`PosUnity`] al espacio de juego de VRChat.
    ///
    /// 1. Desrota la inclinación vertical (*pitch*) del sensor sobre el eje X.
    /// 2. Eleva la coordenada Y sumando la altura del sensor respecto al piso.
    /// 3. Centra la coordenada Z restando la distancia de trabajo de referencia.
    pub fn transformar_posicion(&self, pos: PosUnity) -> PosUnity {
        let mut p = pos.0;

        // 1. Compensar inclinación del sensor (pitch) si es distinta de cero
        if self.inclinacion_grados.abs() > 0.01 {
            let rad = -self.inclinacion_grados.to_radians();
            let rot_pitch = Mat3::from_rotation_x(rad);
            p = rot_pitch.mul_vec3(p);
        }

        // 2. Elevar respecto al suelo y recentrar en Z
        let x_vr = p.x;
        let y_vr = p.y + self.altura_sensor_m;
        let z_vr = p.z - self.distancia_referencia_z;

        PosUnity(Vec3::new(x_vr, y_vr, z_vr))
    }

    /// Transforma una orientación [`Quat`] para compensar la inclinación física del sensor.
    pub fn transformar_rotacion(&self, rot: Quat) -> Quat {
        if self.inclinacion_grados.abs() > 0.01 {
            let rad = -self.inclinacion_grados.to_radians();
            let rot_pitch = Quat::from_rotation_x(rad);
            (rot_pitch * rot).normalize()
        } else {
            rot
        }
    }

    /// Aplica la calibración espacial a todas las articulaciones activas de un [`Esqueleto3D`].
    pub fn aplicar_a_esqueleto(&self, esqueleto: &mut Esqueleto3D) {
        for (art, pose) in esqueleto.articulaciones.iter_mut() {
            if pose.confianza <= 0.0 {
                continue;
            }

            let pos_vr = self.transformar_posicion(pose.posicion);
            let rot_vr = self.transformar_rotacion(pose.rotacion);

            // Garantizar que los pies no atraviesen el piso virtual
            let y_corregido = match art {
                Articulacion::PieIzq | Articulacion::PieDer => {
                    pos_vr.0.y.max(self.cota_suelo_min_m)
                }
                _ => pos_vr.0.y,
            };

            *pose = PoseArticulacion::nueva(
                PosUnity(Vec3::new(pos_vr.0.x, y_corregido, pos_vr.0.z)),
                rot_vr,
                pose.confianza,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibracion_elevacion_y_recentrado() {
        let cal = CalibracionEspacio::nueva(1.20, 0.0, 2.0);

        // Un pie en el suelo frente al sensor a 2 metros:
        // En cámara: X = 0.0, Y = -1.20 (a 1.2 m abajo de la lente), Z = 2.0 (a 2 m de distancia)
        let pos_cam = PosUnity(Vec3::new(0.0, -1.20, 2.0));
        let pos_vr = cal.transformar_posicion(pos_cam);

        assert!((pos_vr.0.x).abs() < 1e-4);
        assert!((pos_vr.0.y - 0.0).abs() < 1e-4); // En el suelo de VRChat (Y = 0)
        assert!((pos_vr.0.z - 0.0).abs() < 1e-4); // En el centro del espacio de juego (Z = 0)
    }

    #[test]
    fn test_calibracion_cota_minima_pies() {
        let cal = CalibracionEspacio::default();
        let mut esqueleto = Esqueleto3D::nuevo(0);

        // Insertar pie con coordenada Y negativa en espacio de cámara
        esqueleto.insertar(
            Articulacion::PieIzq,
            PoseArticulacion::nueva(PosUnity(Vec3::new(0.0, -2.0, 2.0)), Quat::IDENTITY, 0.9),
        );

        cal.aplicar_a_esqueleto(&mut esqueleto);

        let pie = esqueleto
            .obtener(Articulacion::PieIzq)
            .expect("Pie presente");
        assert!(pie.posicion.0.y >= cal.cota_suelo_min_m);
    }
}
