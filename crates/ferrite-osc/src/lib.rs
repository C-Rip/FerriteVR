//! Emisor OSC de telemetría de seguimiento corporal hacia VRChat.
//!
//! Este crate implementa la especificación de trackers OSC de VRChat:
//! - Mapeo de articulaciones corporales a ranuras OSC ([`RanuraTracker`]).
//! - Empaquetado atómico en [`rosc::OscBundle`] de mensajes de posición y rotación.
//! - Transmisión UDP hacia VRChat sin asignaciones innecesarias ni desincronización articular.

#![deny(non_ascii_idents)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::net::{SocketAddr, UdpSocket};

use ferrite_nucleo::{Articulacion, Esqueleto3D, cuaternion_a_euler_zxy_grados};
use rosc::{OscBundle, OscMessage, OscPacket, OscTime, OscType};
use thiserror::Error;

/// Errores producidos durante las operaciones del emisor OSC.
#[derive(Debug, Error)]
pub enum ErrorOsc {
    /// La ranura numérica indicada está fuera del intervalo permitido `1..=8`.
    #[error("la ranura numerada {0} es invalida: debe encontrarse en el rango 1..=8")]
    RanuraInvalida(u8),

    /// Error de entrada/salida en el socket de red UDP.
    #[error("error de red I/O al operar el socket UDP: {0}")]
    Red(#[from] std::io::Error),

    /// Error al serializar o codificar el paquete OSC con `rosc`.
    #[error("error al codificar el paquete OSC: {0}")]
    Codificacion(#[from] rosc::OscError),
}

/// Identificador de ranura OSC en la jerarquía de trackers de VRChat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RanuraTracker {
    /// Tracker especial para la cabeza (`/tracking/trackers/head`).
    Cabeza,
    /// Tracker numerado del cuerpo en el rango `1..=8` (`/tracking/trackers/{1..8}`).
    Numerada(u8),
}

impl RanuraTracker {
    /// Crea una nueva ranura numerada validando que pertenezca al rango `1..=8`.
    ///
    /// # Errores
    /// Devuelve [`ErrorOsc::RanuraInvalida`] si `numero` no se encuentra en `1..=8`.
    ///
    /// ```
    /// use ferrite_osc::RanuraTracker;
    ///
    /// assert!(RanuraTracker::nueva_numerada(1).is_ok());
    /// assert!(RanuraTracker::nueva_numerada(8).is_ok());
    /// assert!(RanuraTracker::nueva_numerada(9).is_err());
    /// assert!(RanuraTracker::nueva_numerada(0).is_err());
    /// ```
    pub fn nueva_numerada(numero: u8) -> Result<Self, ErrorOsc> {
        if (1..=8).contains(&numero) {
            Ok(Self::Numerada(numero))
        } else {
            Err(ErrorOsc::RanuraInvalida(numero))
        }
    }

    /// Retorna la ruta base OSC asociada a esta ranura.
    ///
    /// ```
    /// use ferrite_osc::RanuraTracker;
    ///
    /// assert_eq!(RanuraTracker::Cabeza.direccion(), "/tracking/trackers/head");
    /// assert_eq!(
    ///     RanuraTracker::nueva_numerada(1).unwrap().direccion(),
    ///     "/tracking/trackers/1"
    /// );
    /// ```
    #[inline]
    pub fn direccion(&self) -> &'static str {
        match self {
            Self::Cabeza => "/tracking/trackers/head",
            Self::Numerada(1) => "/tracking/trackers/1",
            Self::Numerada(2) => "/tracking/trackers/2",
            Self::Numerada(3) => "/tracking/trackers/3",
            Self::Numerada(4) => "/tracking/trackers/4",
            Self::Numerada(5) => "/tracking/trackers/5",
            Self::Numerada(6) => "/tracking/trackers/6",
            Self::Numerada(7) => "/tracking/trackers/7",
            Self::Numerada(8) => "/tracking/trackers/8",
            Self::Numerada(_) => "/tracking/trackers/desconocido",
        }
    }
}

/// Muestra espacial y angular de un tracker para un frame temporal.
#[derive(Debug, Clone, PartialEq)]
pub struct MuestraTracker {
    /// Ranura destino en VRChat.
    pub ranura: RanuraTracker,
    /// Coordenadas de posición `[x, y, z]` en metros (espacio Unity siniestro).
    pub posicion: [f32; 3],
    /// Orientación en grados sexagesimales `[x, y, z]` con orden intrínseco ZXY.
    pub euler_grados: [f32; 3],
}

impl MuestraTracker {
    /// Crea una nueva muestra de tracker.
    #[inline]
    pub fn nueva(ranura: RanuraTracker, posicion: [f32; 3], euler_grados: [f32; 3]) -> Self {
        Self {
            ranura,
            posicion,
            euler_grados,
        }
    }
}

/// Convierte un esqueleto tridimensional en una lista de muestras de tracker para VRChat.
///
/// Aplica la convención interna fija de asignación de ranuras de FerriteVR:
/// - Cabeza: `RanuraTracker::Cabeza`
/// - Cadera: `RanuraTracker::Numerada(1)`
/// - Pie Izquierdo: `RanuraTracker::Numerada(2)`
/// - Pie Derecho: `RanuraTracker::Numerada(3)`
/// - Pecho: `RanuraTracker::Numerada(4)`
/// - Rodilla Izquierda: `RanuraTracker::Numerada(5)`
/// - Rodilla Derecha: `RanuraTracker::Numerada(6)`
/// - Codo Izquierdo: `RanuraTracker::Numerada(7)`
/// - Codo Derecho: `RanuraTracker::Numerada(8)`
///
/// Las articulaciones con confianza estadística menor o igual a `0.0` son descartadas.
pub fn muestras_desde_esqueleto(esqueleto: &Esqueleto3D) -> Vec<MuestraTracker> {
    let mut muestras = Vec::with_capacity(esqueleto.articulaciones.len());

    for (articulacion, pose) in &esqueleto.articulaciones {
        if pose.confianza <= 0.0 {
            continue;
        }

        let ranura = match articulacion {
            Articulacion::Cabeza => RanuraTracker::Cabeza,
            Articulacion::Cadera => RanuraTracker::Numerada(1),
            Articulacion::PieIzq => RanuraTracker::Numerada(2),
            Articulacion::PieDer => RanuraTracker::Numerada(3),
            Articulacion::Pecho => RanuraTracker::Numerada(4),
            Articulacion::RodillaIzq => RanuraTracker::Numerada(5),
            Articulacion::RodillaDer => RanuraTracker::Numerada(6),
            Articulacion::CodoIzq => RanuraTracker::Numerada(7),
            Articulacion::CodoDer => RanuraTracker::Numerada(8),
        };

        let posicion = [pose.posicion.0.x, pose.posicion.0.y, pose.posicion.0.z];
        let euler_grados = cuaternion_a_euler_zxy_grados(pose.rotacion);

        muestras.push(MuestraTracker::nueva(ranura, posicion, euler_grados));
    }

    muestras
}

/// Emisor de telemetría de seguimiento corporal hacia VRChat a través de UDP.
#[derive(Debug)]
pub struct EmisorVrchat {
    socket: UdpSocket,
    destino: SocketAddr,
}

impl EmisorVrchat {
    /// Inicializa un nuevo emisor UDP vinculado a un puerto local efímero y apuntando
    /// a la dirección del receptor OSC de VRChat especificada.
    ///
    /// # Errores
    /// Devuelve [`ErrorOsc::Red`] si falla el enlazado del socket UDP local.
    pub fn nuevo(destino: SocketAddr) -> Result<Self, ErrorOsc> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        Ok(Self { socket, destino })
    }

    /// Retorna la dirección de destino configurada para los paquetes OSC.
    #[inline]
    pub fn destino(&self) -> SocketAddr {
        self.destino
    }

    /// Empaqueta todas las muestras de seguimiento del cuadro en un único [`OscBundle`]
    /// atómico y lo transmite por UDP hacia VRChat.
    ///
    /// Cada tracker genera dos mensajes OSC:
    /// - `{direccion}/position` con 3 argumentos flotantes (metros).
    /// - `{direccion}/rotation` con 3 argumentos flotantes (grados Euler en orden ZXY).
    ///
    /// # Errores
    /// Devuelve [`ErrorOsc::Codificacion`] si falla la codificación OSC, o
    /// [`ErrorOsc::Red`] si ocurre un problema durante la transmisión por la red.
    pub fn enviar_frame(&self, muestras: &[MuestraTracker]) -> Result<(), ErrorOsc> {
        if muestras.is_empty() {
            return Ok(());
        }

        let mut mensajes = Vec::with_capacity(muestras.len() * 2);

        for muestra in muestras {
            let base = muestra.ranura.direccion();

            let msg_pos = OscPacket::Message(OscMessage {
                addr: format!("{base}/position"),
                args: muestra
                    .posicion
                    .iter()
                    .copied()
                    .map(OscType::Float)
                    .collect(),
            });

            let msg_rot = OscPacket::Message(OscMessage {
                addr: format!("{base}/rotation"),
                args: muestra
                    .euler_grados
                    .iter()
                    .copied()
                    .map(OscType::Float)
                    .collect(),
            });

            mensajes.push(msg_pos);
            mensajes.push(msg_rot);
        }

        let paquete_bundle = OscPacket::Bundle(OscBundle {
            // El timetag 1 en NTP (bit 0 de la fracción) representa ejecución inmediata en OSC 1.0.
            timetag: OscTime {
                seconds: 0,
                fractional: 1,
            },
            content: mensajes,
        });

        let bytes = rosc::encoder::encode(&paquete_bundle)?;
        self.socket.send_to(&bytes, self.destino)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_nucleo::{PosUnity, PoseArticulacion};
    use glam::{Quat, Vec3};

    #[test]
    fn test_validacion_ranuras_numeradas() {
        for n in 1..=8 {
            assert!(RanuraTracker::nueva_numerada(n).is_ok());
        }
        assert!(matches!(
            RanuraTracker::nueva_numerada(0),
            Err(ErrorOsc::RanuraInvalida(0))
        ));
        assert!(matches!(
            RanuraTracker::nueva_numerada(9),
            Err(ErrorOsc::RanuraInvalida(9))
        ));
    }

    #[test]
    fn test_direcciones_ranuras() {
        assert_eq!(RanuraTracker::Cabeza.direccion(), "/tracking/trackers/head");
        assert_eq!(
            RanuraTracker::nueva_numerada(1).unwrap().direccion(),
            "/tracking/trackers/1"
        );
        assert_eq!(
            RanuraTracker::nueva_numerada(8).unwrap().direccion(),
            "/tracking/trackers/8"
        );
    }

    #[test]
    fn test_muestras_desde_esqueleto_filtra_confianza_cero() {
        let mut esqueleto = Esqueleto3D::nuevo(1000);
        esqueleto.insertar(
            Articulacion::Cabeza,
            PoseArticulacion::nueva(PosUnity(Vec3::new(0.0, 1.7, 0.0)), Quat::IDENTITY, 0.9),
        );
        esqueleto.insertar(
            Articulacion::Cadera,
            PoseArticulacion::nueva(PosUnity(Vec3::new(0.0, 1.0, 0.0)), Quat::IDENTITY, 0.0),
        );
        esqueleto.insertar(
            Articulacion::PieIzq,
            PoseArticulacion::nueva(PosUnity(Vec3::new(-0.2, 0.1, 0.0)), Quat::IDENTITY, -0.5),
        );

        let muestras = muestras_desde_esqueleto(&esqueleto);
        assert_eq!(muestras.len(), 1);
        assert_eq!(muestras[0].ranura, RanuraTracker::Cabeza);
    }

    #[test]
    fn test_emisor_y_receptor_udp_integracion() {
        // Enlazar socket receptor en puerto efímero de localhost
        let receptor = UdpSocket::bind("127.0.0.1:0").expect("falla al enlazar receptor");
        let receptor_addr = receptor.local_addr().expect("falla al obtener local_addr");

        // Crear emisor apuntando al receptor
        let emisor = EmisorVrchat::nuevo(receptor_addr).expect("falla al crear emisor");

        let muestras = vec![
            MuestraTracker::nueva(RanuraTracker::Cabeza, [0.0, 1.72, 0.05], [10.0, -5.0, 2.0]),
            MuestraTracker::nueva(
                RanuraTracker::nueva_numerada(1).unwrap(),
                [0.0, 1.01, -0.02],
                [0.0, 0.0, 0.0],
            ),
        ];

        emisor
            .enviar_frame(&muestras)
            .expect("falla al enviar frame");

        // Recibir paquete en el receptor
        let mut buffer = [0u8; 4096];
        let (bytes_leidos, _origen) = receptor
            .recv_from(&mut buffer)
            .expect("falla al recibir UDP");

        // Decodificar paquete con rosc
        let (_remanente, paquete) = rosc::decoder::decode_udp(&buffer[..bytes_leidos])
            .expect("falla al decodificar paquete OSC");

        match paquete {
            OscPacket::Bundle(bundle) => {
                // Cada muestra produce 2 mensajes: position y rotation
                assert_eq!(bundle.content.len(), 4);

                let msg0 = match &bundle.content[0] {
                    OscPacket::Message(m) => m,
                    _ => panic!("se esperaba mensaje OSC"),
                };
                assert_eq!(msg0.addr, "/tracking/trackers/head/position");
                assert_eq!(msg0.args.len(), 3);
                assert_eq!(msg0.args[0], OscType::Float(0.0));
                assert_eq!(msg0.args[1], OscType::Float(1.72));
                assert_eq!(msg0.args[2], OscType::Float(0.05));

                let msg1 = match &bundle.content[1] {
                    OscPacket::Message(m) => m,
                    _ => panic!("se esperaba mensaje OSC"),
                };
                assert_eq!(msg1.addr, "/tracking/trackers/head/rotation");
                assert_eq!(msg1.args.len(), 3);
                assert_eq!(msg1.args[0], OscType::Float(10.0));
                assert_eq!(msg1.args[1], OscType::Float(-5.0));
                assert_eq!(msg1.args[2], OscType::Float(2.0));

                let msg2 = match &bundle.content[2] {
                    OscPacket::Message(m) => m,
                    _ => panic!("se esperaba mensaje OSC"),
                };
                assert_eq!(msg2.addr, "/tracking/trackers/1/position");
                assert_eq!(msg2.args[0], OscType::Float(0.0));
                assert_eq!(msg2.args[1], OscType::Float(1.01));
                assert_eq!(msg2.args[2], OscType::Float(-0.02));

                let msg3 = match &bundle.content[3] {
                    OscPacket::Message(m) => m,
                    _ => panic!("se esperaba mensaje OSC"),
                };
                assert_eq!(msg3.addr, "/tracking/trackers/1/rotation");
                assert_eq!(msg3.args[0], OscType::Float(0.0));
                assert_eq!(msg3.args[1], OscType::Float(0.0));
                assert_eq!(msg3.args[2], OscType::Float(0.0));
            }
            _ => panic!("se esperaba un paquete tipo OscBundle"),
        }
    }
}
