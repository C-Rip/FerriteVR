//! Núcleo compartido de FerriteVR.
//!
//! Este crate proporciona las estructuras y transformaciones matemáticas fundamentales
//! para el seguimiento corporal (*full body tracking*):
//! - Espacios tipados de coordenadas: [`PosKinect`] (diestro) y [`PosUnity`] (siniestro).
//! - Identificación de articulaciones y poses: [`Articulacion`], [`PoseArticulacion`] y [`Esqueleto3D`].
//! - Conversiones angulares deterministas de rotación entre cuaterniones [`glam::Quat`] y
//!   ángulos de Euler en grados con orden intrínseco **ZXY** (estándar de Unity y VRChat OSC).

#![deny(non_ascii_idents)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

use glam::{EulerRot, Quat, Vec3};

/// Posición en el espacio de la cámara Kinect (sistema diestro, metros).
///
/// ### Convención de ejes
/// - `+X`: Hacia la derecha del sensor (metros).
/// - `+Y`: Hacia arriba vertical (metros).
/// - `+Z`: Hacia el sensor / observador (metros; alejándose de la escena).
///
/// Cumple la regla de la mano derecha: `X x Y = Z`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosKinect(pub Vec3);

/// Posición en el espacio de Unity y VRChat (sistema siniestro, metros).
///
/// ### Convención de ejes
/// - `+X`: Hacia la derecha (metros).
/// - `+Y`: Hacia arriba vertical (metros).
/// - `+Z`: Hacia adelante / profundidad en el mundo virtual (metros).
///
/// Cumple la regla de la mano izquierda empleada nativamente por Unity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosUnity(pub Vec3);

impl From<PosKinect> for PosUnity {
    /// Transforma una posición desde el espacio diestro del sensor Kinect hacia el espacio
    /// siniestro de Unity invirtiendo el eje de profundidad Z.
    ///
    /// ```
    /// use ferrite_nucleo::{PosKinect, PosUnity};
    /// use glam::Vec3;
    ///
    /// let pk = PosKinect(Vec3::new(1.0, 2.0, -3.0));
    /// let pu = PosUnity::from(pk);
    /// assert_eq!(pu.0, Vec3::new(1.0, 2.0, 3.0));
    /// ```
    #[inline]
    fn from(origen: PosKinect) -> Self {
        Self(Vec3::new(origen.0.x, origen.0.y, -origen.0.z))
    }
}

impl From<PosUnity> for PosKinect {
    /// Transforma una posición desde el espacio siniestro de Unity hacia el espacio diestro
    /// de la cámara Kinect invirtiendo el eje Z.
    ///
    /// ```
    /// use ferrite_nucleo::{PosKinect, PosUnity};
    /// use glam::Vec3;
    ///
    /// let pu = PosUnity(Vec3::new(0.5, 1.2, 2.5));
    /// let pk = PosKinect::from(pu);
    /// assert_eq!(pk.0, Vec3::new(0.5, 1.2, -2.5));
    /// ```
    #[inline]
    fn from(origen: PosUnity) -> Self {
        Self(Vec3::new(origen.0.x, origen.0.y, -origen.0.z))
    }
}

/// Identificadores de las articulaciones del cuerpo humano rastreadas por FerriteVR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Articulacion {
    /// Articulación de la cabeza.
    Cabeza,
    /// Articulación del pecho / torso superior.
    Pecho,
    /// Articulación de la cadera / raíz del esqueleto.
    Cadera,
    /// Articulación de la rodilla izquierda.
    RodillaIzq,
    /// Articulación de la rodilla derecha.
    RodillaDer,
    /// Articulación del pie izquierdo.
    PieIzq,
    /// Articulación del pie derecho.
    PieDer,
    /// Articulación del codo izquierdo.
    CodoIzq,
    /// Articulación del codo derecho.
    CodoDer,
}

/// Estado espacial estimado de una articulación en un instante de tiempo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoseArticulacion {
    /// Posición en espacio métrico de Unity (mano izquierda).
    pub posicion: PosUnity,
    /// Rotación orientada mediante cuaternión unitario.
    pub rotacion: Quat,
    /// Confianza estadística de la estimación en el rango `[0.0, 1.0]`.
    pub confianza: f32,
}

impl PoseArticulacion {
    /// Crea una nueva pose de articulación con los parámetros indicados.
    #[inline]
    pub fn nueva(posicion: PosUnity, rotacion: Quat, confianza: f32) -> Self {
        Self {
            posicion,
            rotacion,
            confianza,
        }
    }
}

/// Esqueleto corporal 3D completo correspondiente a una captura temporal.
#[derive(Debug, Clone, PartialEq)]
pub struct Esqueleto3D {
    /// Marca de tiempo de la captura en microsegundos desde el inicio de la época o sesión.
    pub marca_tiempo_us: u64,
    /// Lista de articulaciones estimadas junto a su pose articular.
    pub articulaciones: Vec<(Articulacion, PoseArticulacion)>,
}

impl Esqueleto3D {
    /// Crea un esqueleto vacío con la marca de tiempo indicada.
    #[inline]
    pub fn nuevo(marca_tiempo_us: u64) -> Self {
        Self {
            marca_tiempo_us,
            articulaciones: Vec::new(),
        }
    }

    /// Agrega o actualiza la pose de una articulación en el esqueleto.
    pub fn insertar(&mut self, articulacion: Articulacion, pose: PoseArticulacion) {
        if let Some(pos) = self.articulaciones.iter().position(|(a, _)| *a == articulacion) {
            self.articulaciones[pos].1 = pose;
        } else {
            self.articulaciones.push((articulacion, pose));
        }
    }

    /// Obtiene la pose asociada a una articulación, si existe en el esqueleto.
    #[inline]
    pub fn obtener(&self, articulacion: Articulacion) -> Option<&PoseArticulacion> {
        self.articulaciones
            .iter()
            .find(|(a, _)| *a == articulacion)
            .map(|(_, p)| p)
    }
}

/// Normaliza un ángulo en grados sexagesimales al intervalo semiabierto `[-180.0, 180.0]`.
#[inline]
fn normalizar_angulo_grados(grados: f32) -> f32 {
    let angulo = (grados + 180.0).rem_euclid(360.0) - 180.0;
    if angulo == -180.0 {
        180.0
    } else {
        angulo
    }
}

/// Convierte un cuaternión de orientación en ángulos de Euler en grados sexagesimales `[x, y, z]`
/// con orden de rotación intrínseco **ZXY** (el orden estándar de Unity).
///
/// Los tres componentes resultantes representan los giros en grados:
/// - `[0]` (`x`): Inclinación (*pitch*) alrededor del eje X.
/// - `[1]` (`y`): Guiñada (*yaw*) alrededor del eje Y.
/// - `[2]` (`z`): Alabeo (*roll*) alrededor del eje Z.
///
/// Todos los ángulos se normalizan dentro del intervalo `[-180.0, 180.0]`.
///
/// ```
/// use ferrite_nucleo::cuaternion_a_euler_zxy_grados;
/// use glam::Quat;
///
/// let rotacion_identidad = Quat::IDENTITY;
/// let euler = cuaternion_a_euler_zxy_grados(rotacion_identidad);
/// assert!((euler[0]).abs() < 1e-4);
/// assert!((euler[1]).abs() < 1e-4);
/// assert!((euler[2]).abs() < 1e-4);
/// ```
pub fn cuaternion_a_euler_zxy_grados(q: Quat) -> [f32; 3] {
    let q_normalizado = q.normalize();
    let (z_rad, x_rad, y_rad) = q_normalizado.to_euler(EulerRot::ZXY);
    [
        normalizar_angulo_grados(x_rad.to_degrees()),
        normalizar_angulo_grados(y_rad.to_degrees()),
        normalizar_angulo_grados(z_rad.to_degrees()),
    ]
}

/// Convierte una terna de ángulos de Euler en grados `[x, y, z]` (orden intrínseco **ZXY**)
/// en su cuaternión de rotación unitario equivalente.
///
/// Parámetros:
/// - `euler_grados[0]` (`x`): Inclinación (*pitch*) en grados alrededor de X.
/// - `euler_grados[1]` (`y`): Guiñada (*yaw*) en grados alrededor de Y.
/// - `euler_grados[2]` (`z`): Alabeo (*roll*) en grados alrededor de Z.
///
/// ```
/// use ferrite_nucleo::{cuaternion_a_euler_zxy_grados, euler_zxy_grados_a_cuaternion};
/// use glam::Quat;
///
/// let angulos_originales = [15.0, -30.0, 45.0];
/// let q = euler_zxy_grados_a_cuaternion(angulos_originales);
/// let euler_reconstruido = cuaternion_a_euler_zxy_grados(q);
///
/// assert!((angulos_originales[0] - euler_reconstruido[0]).abs() < 1e-4);
/// assert!((angulos_originales[1] - euler_reconstruido[1]).abs() < 1e-4);
/// assert!((angulos_originales[2] - euler_reconstruido[2]).abs() < 1e-4);
/// ```
pub fn euler_zxy_grados_a_cuaternion(euler_grados: [f32; 3]) -> Quat {
    let x_rad = euler_grados[0].to_radians();
    let y_rad = euler_grados[1].to_radians();
    let z_rad = euler_grados[2].to_radians();

    Quat::from_euler(EulerRot::ZXY, z_rad, x_rad, y_rad).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_conversion_posiciones_conocidas() {
        let pk = PosKinect(Vec3::new(1.5, -0.5, -2.0));
        let pu = PosUnity::from(pk);
        assert_eq!(pu.0, Vec3::new(1.5, -0.5, 2.0));

        let pu_vuelta = PosKinect::from(pu);
        assert_eq!(pu_vuelta, pk);
    }

    #[test]
    fn test_esqueleto_insercion_y_busqueda() {
        let mut esqueleto = Esqueleto3D::nuevo(123456);
        let pose_cabeza = PoseArticulacion::nueva(
            PosUnity(Vec3::new(0.0, 1.7, 0.0)),
            Quat::IDENTITY,
            0.95,
        );
        esqueleto.insertar(Articulacion::Cabeza, pose_cabeza);

        assert_eq!(esqueleto.obtener(Articulacion::Cabeza), Some(&pose_cabeza));
        assert_eq!(esqueleto.obtener(Articulacion::PieIzq), None);
    }

    #[test]
    fn test_rotacion_identidad() {
        let q = Quat::IDENTITY;
        let euler = cuaternion_a_euler_zxy_grados(q);
        assert!((euler[0]).abs() < 1e-4);
        assert!((euler[1]).abs() < 1e-4);
        assert!((euler[2]).abs() < 1e-4);

        let q_reconstruido = euler_zxy_grados_a_cuaternion(euler);
        assert!(q.dot(q_reconstruido).abs() > 1.0 - 1e-4);
    }

    #[test]
    fn test_rotacion_ejes_individuales() {
        let casos = [
            [90.0, 0.0, 0.0],
            [-45.0, 0.0, 0.0],
            [0.0, 90.0, 0.0],
            [0.0, -60.0, 0.0],
            [0.0, 0.0, 90.0],
            [0.0, 0.0, -30.0],
            [20.0, 35.0, -40.0],
        ];

        for caso in casos {
            let q = euler_zxy_grados_a_cuaternion(caso);
            let euler = cuaternion_a_euler_zxy_grados(q);
            assert!(
                (caso[0] - euler[0]).abs() < 1e-4,
                "Fallo en eje X para {:?}: obtenido {:?}",
                caso,
                euler
            );
            assert!(
                (caso[1] - euler[1]).abs() < 1e-4,
                "Fallo en eje Y para {:?}: obtenido {:?}",
                caso,
                euler
            );
            assert!(
                (caso[2] - euler[2]).abs() < 1e-4,
                "Fallo en eje Z para {:?}: obtenido {:?}",
                caso,
                euler
            );
        }
    }

    proptest! {
        #[test]
        fn prop_pos_kinect_unity_ida_y_vuelta(
            x in -100.0f32..100.0f32,
            y in -100.0f32..100.0f32,
            z in -100.0f32..100.0f32,
        ) {
            let origen = PosKinect(Vec3::new(x, y, z));
            let destino = PosUnity::from(origen);
            let vuelta = PosKinect::from(destino);

            prop_assert!((origen.0.x - vuelta.0.x).abs() < 1e-5);
            prop_assert!((origen.0.y - vuelta.0.y).abs() < 1e-5);
            prop_assert!((origen.0.z - vuelta.0.z).abs() < 1e-5);
        }

        #[test]
        fn prop_cuaternion_euler_equivalencia_rotacion(
            x in -1.0f32..1.0f32,
            y in -1.0f32..1.0f32,
            z in -1.0f32..1.0f32,
            w in -1.0f32..1.0f32,
        ) {
            let norma_cuad = x * x + y * y + z * z + w * w;
            prop_assume!(norma_cuad > 1e-4);

            let q_original = Quat::from_xyzw(x, y, z, w).normalize();
            let euler = cuaternion_a_euler_zxy_grados(q_original);
            let q_reconstruido = euler_zxy_grados_a_cuaternion(euler);

            // Dos cuaterniones q y -q representan la misma rotación 3D
            let producto_punto = q_original.dot(q_reconstruido).abs();
            prop_assert!(
                producto_punto > 1.0 - 1e-4,
                "q_original: {:?}, q_reconstruido: {:?}, dot: {}",
                q_original,
                q_reconstruido,
                producto_punto
            );
        }

        #[test]
        fn prop_euler_a_cuaternion_ida_y_vuelta_angulos_acotados(
            // Evitamos singularidad de gimbal lock en pitch (X) cerca de +/- 90 grados
            x in -80.0f32..80.0f32,
            y in -175.0f32..175.0f32,
            z in -175.0f32..175.0f32,
        ) {
            let original = [x, y, z];
            let q = euler_zxy_grados_a_cuaternion(original);
            let reconstruido = cuaternion_a_euler_zxy_grados(q);

            prop_assert!((original[0] - reconstruido[0]).abs() < 1e-4);
            prop_assert!((original[1] - reconstruido[1]).abs() < 1e-4);
            prop_assert!((original[2] - reconstruido[2]).abs() < 1e-4);
        }
    }
}
