//! Elevación geométrica de puntos clave 2D a posiciones 3D en espacio de Unity.

use ferrite_nucleo::{Articulacion, IntrinsecosProfundidad, PosUnity};
use ferrite_pose::simcc::{NUM_ARTICULACIONES_HALPE, Pose2D};
use glam::Vec3;

/// Posición tridimensional elevada con confianza asociada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Punto3D {
    /// Posición en coordenadas métricas de Unity (mano izquierda, +Y arriba).
    pub posicion: PosUnity,
    /// Confianza estadística combinada (detección 2D + validez de profundidad).
    pub confianza: f32,
}

impl Default for Punto3D {
    fn default() -> Self {
        Self {
            posicion: PosUnity(Vec3::ZERO),
            confianza: 0.0,
        }
    }
}

/// Conjunto de articulaciones corporales elevadas a tres dimensiones antes de calcular orientaciones.
#[derive(Debug, Clone)]
pub struct EsqueletoElevado {
    /// Poses 3D directas de las 26 articulaciones de Halpe.
    pub articulaciones_halpe: [Punto3D; NUM_ARTICULACIONES_HALPE],
}

impl Default for EsqueletoElevado {
    fn default() -> Self {
        Self {
            articulaciones_halpe: [Punto3D::default(); NUM_ARTICULACIONES_HALPE],
        }
    }
}

/// Elevador geométrico que muestrea la profundidad local y desproyecta coordenadas 2D a 3D.
pub struct Elevador3D {
    radio_ventana: i32,
    distancia_min_mm: f32,
    distancia_max_mm: f32,
    buffer_muestras: Vec<f32>,
}

impl Default for Elevador3D {
    fn default() -> Self {
        Self::nuevo(2, 500.0, 4500.0) // radio 2 equivale a ventana 5x5
    }
}

impl Elevador3D {
    /// Crea un nuevo elevador con el radio de vecindad y límites de profundidad indicados.
    pub fn nuevo(radio_ventana: i32, distancia_min_mm: f32, distancia_max_mm: f32) -> Self {
        let lado = (radio_ventana * 2 + 1) as usize;
        Self {
            radio_ventana: radio_ventana.max(1),
            distancia_min_mm,
            distancia_max_mm,
            buffer_muestras: Vec::with_capacity(lado * lado),
        }
    }

    /// Muestrea la profundidad local en una ventana `(2R+1) × (2R+1)` alrededor de `(u, v)`
    /// y retorna la mediana de las profundidades válidas.
    pub fn estimar_profundidad_mediana(
        &mut self,
        u_centro: f32,
        v_centro: f32,
        profundidad: &[f32],
        ancho: u32,
        alto: u32,
    ) -> Option<f32> {
        let u_int = u_centro.round() as i32;
        let v_int = v_centro.round() as i32;

        self.buffer_muestras.clear();

        let ancho_i = ancho as i32;
        let alto_i = alto as i32;

        for dv in -self.radio_ventana..=self.radio_ventana {
            let v = v_int + dv;
            if v < 0 || v >= alto_i {
                continue;
            }
            let offset_v = (v as usize) * (ancho as usize);

            for du in -self.radio_ventana..=self.radio_ventana {
                let u = u_int + du;
                if u < 0 || u >= ancho_i {
                    continue;
                }
                let idx = offset_v + (u as usize);
                let p = profundidad[idx];
                if p >= self.distancia_min_mm && p <= self.distancia_max_mm && p.is_finite() {
                    self.buffer_muestras.push(p);
                }
            }
        }

        if self.buffer_muestras.len() < 3 {
            return None;
        }

        self.buffer_muestras.sort_by(|a, b| a.total_cmp(b));
        let mitad = self.buffer_muestras.len() / 2;
        Some(self.buffer_muestras[mitad])
    }

    /// Eleva las 26 articulaciones de `Pose2D` a coordenadas 3D en espacio métrico de Unity.
    pub fn elevar(
        &mut self,
        pose_2d: &Pose2D,
        profundidad: &[f32],
        ancho: u32,
        alto: u32,
        intrinsecos: &IntrinsecosProfundidad,
    ) -> EsqueletoElevado {
        let mut esqueleto = EsqueletoElevado::default();

        for (k, punto_2d) in pose_2d.puntos.iter().enumerate() {
            if punto_2d.confianza < 0.15 {
                continue;
            }

            if let Some(prof_mm) =
                self.estimar_profundidad_mediana(punto_2d.u, punto_2d.v, profundidad, ancho, alto)
            {
                let pos_kinect = intrinsecos.desproyectar(punto_2d.u, punto_2d.v, prof_mm);
                let pos_unity = PosUnity::from(pos_kinect);

                esqueleto.articulaciones_halpe[k] = Punto3D {
                    posicion: pos_unity,
                    confianza: punto_2d.confianza,
                };
            }
        }

        esqueleto
    }

    /// Extrae la posición estimada para una articulación específica del esqueleto de FerriteVR.
    pub fn extraer_articulacion(
        &self,
        esqueleto: &EsqueletoElevado,
        articulacion: Articulacion,
    ) -> Punto3D {
        let h = &esqueleto.articulaciones_halpe;

        match articulacion {
            Articulacion::Cabeza => h[17],
            Articulacion::Pecho => {
                // Punto medio entre hombros (5, 6) y cuello (18)
                let c_izq = h[5];
                let c_der = h[6];
                let cuello = h[18];

                if c_izq.confianza > 0.1 && c_der.confianza > 0.1 {
                    let pos_media = (c_izq.posicion.0 + c_der.posicion.0) * 0.5;
                    let pos_final = if cuello.confianza > 0.1 {
                        (pos_media + cuello.posicion.0) * 0.5
                    } else {
                        pos_media
                    };
                    Punto3D {
                        posicion: PosUnity(pos_final),
                        confianza: (c_izq.confianza + c_der.confianza) * 0.5,
                    }
                } else if cuello.confianza > 0.1 {
                    cuello
                } else {
                    Punto3D::default()
                }
            }
            Articulacion::Cadera => {
                // Cadera central (19) o punto medio de caderas izquierda (11) y derecha (12)
                let pelvis = h[19];
                if pelvis.confianza > 0.25 {
                    pelvis
                } else {
                    let cad_izq = h[11];
                    let cad_der = h[12];
                    if cad_izq.confianza > 0.1 && cad_der.confianza > 0.1 {
                        Punto3D {
                            posicion: PosUnity((cad_izq.posicion.0 + cad_der.posicion.0) * 0.5),
                            confianza: (cad_izq.confianza + cad_der.confianza) * 0.5,
                        }
                    } else {
                        pelvis
                    }
                }
            }
            Articulacion::RodillaIzq => h[13],
            Articulacion::RodillaDer => h[14],
            Articulacion::PieIzq => {
                // Tobillo (15) o talón (22)
                let tobillo = h[15];
                let talon = h[22];
                if talon.confianza > 0.25 {
                    talon
                } else {
                    tobillo
                }
            }
            Articulacion::PieDer => {
                let tobillo = h[16];
                let talon = h[25];
                if talon.confianza > 0.25 {
                    talon
                } else {
                    tobillo
                }
            }
            Articulacion::CodoIzq => h[7],
            Articulacion::CodoDer => h[8],
        }
    }
}
