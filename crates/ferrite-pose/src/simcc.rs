//! Decodificación de salidas SimCC (Simple Coordinate Classification) a puntos clave 2D.

use crate::bbox::CajaDelimitadora;
use crate::preprocesado::{ALTO_MODELO, ANCHO_MODELO};

/// Cantidad total de articulaciones anotadas en el estándar Halpe26.
pub const NUM_ARTICULACIONES_HALPE: usize = 26;

/// Factor de división subpíxel empleado por RTMPose en SimCC (`simcc_split_ratio = 2.0`).
pub const SIMCC_SPLIT_RATIO: f32 = 2.0;

/// Cantidad de bins en la dimensión horizontal para SimCC (`192 * 2 = 384`).
pub const BINS_SIMCC_X: usize = (ANCHO_MODELO as usize) * 2;

/// Cantidad de bins en la dimensión vertical para SimCC (`256 * 2 = 512`).
pub const BINS_SIMCC_Y: usize = (ALTO_MODELO as usize) * 2;

/// Punto clave en coordenadas de píxel 2D con confianza asociada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PuntoClave2D {
    /// Coordenada horizontal en píxeles.
    pub u: f32,
    /// Coordenada vertical en píxeles.
    pub v: f32,
    /// Confianza estadística de la detección en el rango `[0.0, 1.0]`.
    pub confianza: f32,
}

impl Default for PuntoClave2D {
    fn default() -> Self {
        Self {
            u: 0.0,
            v: 0.0,
            confianza: 0.0,
        }
    }
}

/// Estimación de pose 2D completa con los 26 puntos clave de Halpe26.
#[derive(Debug, Clone, PartialEq)]
pub struct Pose2D {
    /// Puntos clave indexados según la convención Halpe26 (0..26).
    pub puntos: [PuntoClave2D; NUM_ARTICULACIONES_HALPE],
    /// Caja delimitadora utilizada para el recorte e inferencia.
    pub bbox: CajaDelimitadora,
}

/// Decodifica los tensores 1D de clasificación `simcc_x` y `simcc_y` a puntos clave 2D
/// y los reproyecta hacia el espacio del fotograma original utilizando la `bbox` dada.
pub fn decodificar_simcc(
    simcc_x: &[f32],
    simcc_y: &[f32],
    bbox: &CajaDelimitadora,
) -> [PuntoClave2D; NUM_ARTICULACIONES_HALPE] {
    let mut puntos = [PuntoClave2D::default(); NUM_ARTICULACIONES_HALPE];

    let bbox_ancho = bbox.ancho();
    let bbox_alto = bbox.alto();
    let ancho_f = ANCHO_MODELO as f32;
    let alto_f = ALTO_MODELO as f32;

    for (k, punto) in puntos.iter_mut().enumerate() {
        let offset_x = k * BINS_SIMCC_X;
        let slice_x = &simcc_x[offset_x..offset_x + BINS_SIMCC_X];

        let offset_y = k * BINS_SIMCC_Y;
        let slice_y = &simcc_y[offset_y..offset_y + BINS_SIMCC_Y];

        // 1. Argmax y valor máximo en X
        let mut idx_max_x = 0usize;
        let mut val_max_x = f32::NEG_INFINITY;
        for (i, &v) in slice_x.iter().enumerate() {
            if v > val_max_x {
                val_max_x = v;
                idx_max_x = i;
            }
        }

        // 2. Argmax y valor máximo en Y
        let mut idx_max_y = 0usize;
        let mut val_max_y = f32::NEG_INFINITY;
        for (j, &v) in slice_y.iter().enumerate() {
            if v > val_max_y {
                val_max_y = v;
                idx_max_y = j;
            }
        }

        // 3. Coordenadas continuas en el espacio del modelo
        let u_mod = (idx_max_x as f32) / SIMCC_SPLIT_RATIO;
        let v_mod = (idx_max_y as f32) / SIMCC_SPLIT_RATIO;

        // 4. Reproyección al espacio original de la cámara
        let u_orig = bbox.u_min + (u_mod / ancho_f) * bbox_ancho;
        let v_orig = bbox.v_min + (v_mod / alto_f) * bbox_alto;

        // 5. Confianza combinada (acotada entre 0.0 y 1.0)
        let conf_cruda = val_max_x.min(val_max_y);
        let confianza = conf_cruda.clamp(0.0, 1.0);

        *punto = PuntoClave2D {
            u: u_orig,
            v: v_orig,
            confianza,
        };
    }

    puntos
}

/// Genera una estimación de pose 2D Halpe26 sintética determinista para pruebas del solver y calibración.
pub fn generar_pose_2d_sintetica(cx: f32, cy: f32, fase_marcha: f32) -> Pose2D {
    let mut puntos = [PuntoClave2D::default(); NUM_ARTICULACIONES_HALPE];

    let elev_izq = (fase_marcha.sin()).max(0.0);
    let elev_der = ((-fase_marcha).sin()).max(0.0);
    let sway_x = 10.0 * fase_marcha.sin();

    // 0: Nariz
    puntos[0] = PuntoClave2D {
        u: cx + 0.5 * sway_x,
        v: cy - 117.0,
        confianza: 0.95,
    };
    // 1, 2: Ojos
    puntos[1] = PuntoClave2D {
        u: cx - 8.0 + 0.5 * sway_x,
        v: cy - 124.0,
        confianza: 0.95,
    };
    puntos[2] = PuntoClave2D {
        u: cx + 8.0 + 0.5 * sway_x,
        v: cy - 124.0,
        confianza: 0.95,
    };
    // 3, 4: Orejas
    puntos[3] = PuntoClave2D {
        u: cx - 22.0 + 0.5 * sway_x,
        v: cy - 122.0,
        confianza: 0.90,
    };
    puntos[4] = PuntoClave2D {
        u: cx + 22.0 + 0.5 * sway_x,
        v: cy - 122.0,
        confianza: 0.90,
    };
    // 5, 6: Hombros
    puntos[5] = PuntoClave2D {
        u: cx - 36.0 + 0.7 * sway_x,
        v: cy - 90.0,
        confianza: 0.95,
    };
    puntos[6] = PuntoClave2D {
        u: cx + 36.0 + 0.7 * sway_x,
        v: cy - 90.0,
        confianza: 0.95,
    };
    // 7, 8: Codos
    puntos[7] = PuntoClave2D {
        u: cx - 48.0,
        v: cy - 35.0 - 15.0 * elev_der,
        confianza: 0.90,
    };
    puntos[8] = PuntoClave2D {
        u: cx + 48.0,
        v: cy - 35.0 - 15.0 * elev_izq,
        confianza: 0.90,
    };
    // 9, 10: Muñecas
    puntos[9] = PuntoClave2D {
        u: cx - 50.0,
        v: cy + 15.0 - 20.0 * elev_der,
        confianza: 0.85,
    };
    puntos[10] = PuntoClave2D {
        u: cx + 50.0,
        v: cy + 15.0 - 20.0 * elev_izq,
        confianza: 0.85,
    };
    // 11, 12: Caderas
    puntos[11] = PuntoClave2D {
        u: cx - 20.0 + sway_x,
        v: cy + 25.0,
        confianza: 0.95,
    };
    puntos[12] = PuntoClave2D {
        u: cx + 20.0 + sway_x,
        v: cy + 25.0,
        confianza: 0.95,
    };
    // 13, 14: Rodillas
    puntos[13] = PuntoClave2D {
        u: cx - 20.0 + 0.5 * sway_x,
        v: cy + 95.0 - 18.0 * elev_izq,
        confianza: 0.95,
    };
    puntos[14] = PuntoClave2D {
        u: cx + 20.0 + 0.5 * sway_x,
        v: cy + 95.0 - 18.0 * elev_der,
        confianza: 0.95,
    };
    // 15, 16: Tobillos
    puntos[15] = PuntoClave2D {
        u: cx - 20.0,
        v: cy + 165.0 - 30.0 * elev_izq,
        confianza: 0.95,
    };
    puntos[16] = PuntoClave2D {
        u: cx + 20.0,
        v: cy + 165.0 - 30.0 * elev_der,
        confianza: 0.95,
    };
    // 17: Cabeza
    puntos[17] = PuntoClave2D {
        u: cx + 0.5 * sway_x,
        v: cy - 130.0,
        confianza: 0.95,
    };
    // 18: Cuello
    puntos[18] = PuntoClave2D {
        u: cx + 0.6 * sway_x,
        v: cy - 95.0,
        confianza: 0.95,
    };
    // 19: Pelvis
    puntos[19] = PuntoClave2D {
        u: cx + sway_x,
        v: cy + 25.0,
        confianza: 0.95,
    };
    // 20, 21, 22: Pie izquierdo (dedo gordo, dedo pequeño, talón)
    puntos[20] = PuntoClave2D {
        u: cx - 16.0,
        v: cy + 175.0 - 30.0 * elev_izq,
        confianza: 0.90,
    };
    puntos[21] = PuntoClave2D {
        u: cx - 24.0,
        v: cy + 175.0 - 30.0 * elev_izq,
        confianza: 0.90,
    };
    puntos[22] = PuntoClave2D {
        u: cx - 20.0,
        v: cy + 165.0 - 30.0 * elev_izq,
        confianza: 0.90,
    };
    // 23, 24, 25: Pie derecho (dedo gordo, dedo pequeño, talón)
    puntos[23] = PuntoClave2D {
        u: cx + 16.0,
        v: cy + 175.0 - 30.0 * elev_der,
        confianza: 0.90,
    };
    puntos[24] = PuntoClave2D {
        u: cx + 24.0,
        v: cy + 175.0 - 30.0 * elev_der,
        confianza: 0.90,
    };
    puntos[25] = PuntoClave2D {
        u: cx + 20.0,
        v: cy + 165.0 - 30.0 * elev_der,
        confianza: 0.90,
    };

    Pose2D {
        puntos,
        bbox: CajaDelimitadora::nueva(cx - 70.0, cy - 145.0, cx + 70.0, cy + 185.0),
    }
}
