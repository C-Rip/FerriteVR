//! Preprocesamiento de imágenes, recorte afin y normalización NCHW para RTMPose.

use ferrite_sensor::FrameSensor;
use crate::bbox::{estimar_bbox_profundidad, CajaDelimitadora};
use crate::ErrorPose;

/// Ancho de entrada estándar esperado por RTMPose-m en píxeles.
pub const ANCHO_MODELO: u32 = 192;

/// Alto de entrada estándar esperado por RTMPose-m en píxeles.
pub const ALTO_MODELO: u32 = 256;

/// Cantidad total de elementos escalares por fotograma en formato NCHW (`3 * 256 * 192 = 147 456`).
pub const ELEMENTOS_NCHW: usize = 3 * (ALTO_MODELO as usize) * (ANCHO_MODELO as usize);

// Medias y desviaciones estándar oficiales de ImageNet / RTMPose (RGB)
const MEDIA_R: f32 = 123.675;
const MEDIA_G: f32 = 116.280;
const MEDIA_B: f32 = 103.530;

const DESV_R: f32 = 58.395;
const DESV_G: f32 = 57.120;
const DESV_B: f32 = 57.375;

/// Preprocesador de imágenes que recorta, escala y normaliza fotogramas de entrada hacia el tensor NCHW.
///
/// Dispone de buffers internos preasignados para garantizar cero asignaciones de memoria dinámicas
/// durante el procesamiento en tiempo real.
///
/// ```
/// use ferrite_pose::{PreprocesadorPose, ELEMENTOS_NCHW};
///
/// let preprocesador = PreprocesadorPose::default();
/// assert_eq!(ELEMENTOS_NCHW, 147_456);
/// ```
pub struct PreprocesadorPose {
    distancia_min_mm: f32,
    distancia_max_mm: f32,
    margen_bbox: f32,
    buffer_nchw: Vec<f32>,
}

impl Default for PreprocesadorPose {
    fn default() -> Self {
        Self::nuevo(500.0, 4500.0, 0.20)
    }
}

impl PreprocesadorPose {
    /// Crea un nuevo preprocesador con los umbrales de rango de profundidad y margen de recorte especificados.
    pub fn nuevo(distancia_min_mm: f32, distancia_max_mm: f32, margen_bbox: f32) -> Self {
        Self {
            distancia_min_mm,
            distancia_max_mm,
            margen_bbox,
            buffer_nchw: vec![0.0; ELEMENTOS_NCHW],
        }
    }

    /// Estima la caja delimitadora del sujeto sobre el fotograma suministrado.
    pub fn estimar_bbox(&self, frame: &FrameSensor) -> Option<CajaDelimitadora> {
        estimar_bbox_profundidad(
            &frame.profundidad,
            frame.ancho,
            frame.alto,
            self.distancia_min_mm,
            self.distancia_max_mm,
            self.margen_bbox,
        )
    }

    /// Recorta la región indicada por la `bbox` (o la imagen completa si no se proporciona)
    /// mediante interpolación bilineal, normaliza los canales RGB y rellena el buffer NCHW continuo.
    ///
    /// Retorna una referencia de solo lectura al slice de `147 456` flotantes listo para inferencia ONNX.
    pub fn preprocesar(
        &mut self,
        frame: &FrameSensor,
        bbox_opt: Option<CajaDelimitadora>,
    ) -> Result<&[f32], ErrorPose> {
        if frame.ancho == 0 || frame.alto == 0 {
            return Err(ErrorPose::DimensionesInvalidas(frame.ancho, frame.alto));
        }

        let bbox = bbox_opt.unwrap_or_else(|| {
            CajaDelimitadora::nueva(
                0.0,
                0.0,
                (frame.ancho - 1) as f32,
                (frame.alto - 1) as f32,
            )
            .ajustar_proporcion(
                (ANCHO_MODELO as f32) / (ALTO_MODELO as f32),
                (frame.ancho - 1) as f32,
                (frame.alto - 1) as f32,
            )
        });

        let ancho_src = frame.ancho as f32;
        let alto_src = frame.alto as f32;
        let stride_canal = (ALTO_MODELO as usize) * (ANCHO_MODELO as usize);

        let bbox_ancho = bbox.ancho();
        let bbox_alto = bbox.alto();

        for y_dst in 0..ALTO_MODELO {
            let beta = ((y_dst as f32) + 0.5) / (ALTO_MODELO as f32);
            let v_src = (bbox.v_min + beta * bbox_alto - 0.5).clamp(0.0, alto_src - 1.0);

            let v0 = v_src.floor() as usize;
            let v1 = (v0 + 1).min((frame.alto - 1) as usize);
            let wv = v_src - (v0 as f32);

            let offset_fila_dst = (y_dst as usize) * (ANCHO_MODELO as usize);

            for x_dst in 0..ANCHO_MODELO {
                let alpha = ((x_dst as f32) + 0.5) / (ANCHO_MODELO as f32);
                let u_src = (bbox.u_min + alpha * bbox_ancho - 0.5).clamp(0.0, ancho_src - 1.0);

                let u0 = u_src.floor() as usize;
                let u1 = (u0 + 1).min((frame.ancho - 1) as usize);
                let wu = u_src - (u0 as f32);

                // Cuatro esquinas para interpolación bilineal
                let p00 = frame.color_registrado[v0 * (frame.ancho as usize) + u0];
                let p10 = frame.color_registrado[v0 * (frame.ancho as usize) + u1];
                let p01 = frame.color_registrado[v1 * (frame.ancho as usize) + u0];
                let p11 = frame.color_registrado[v1 * (frame.ancho as usize) + u1];

                // Interpolación canal R (0)
                let r0 = (p00[0] as f32) * (1.0 - wu) + (p10[0] as f32) * wu;
                let r1 = (p01[0] as f32) * (1.0 - wu) + (p11[0] as f32) * wu;
                let r_interp = r0 * (1.0 - wv) + r1 * wv;

                // Interpolación canal G (1)
                let g0 = (p00[1] as f32) * (1.0 - wu) + (p10[1] as f32) * wu;
                let g1 = (p01[1] as f32) * (1.0 - wu) + (p11[1] as f32) * wu;
                let g_interp = g0 * (1.0 - wv) + g1 * wv;

                // Interpolación canal B (2)
                let b0 = (p00[2] as f32) * (1.0 - wu) + (p10[2] as f32) * wu;
                let b1 = (p01[2] as f32) * (1.0 - wu) + (p11[2] as f32) * wu;
                let b_interp = b0 * (1.0 - wv) + b1 * wv;

                // Normalización oficial RTMPose
                let r_norm = (r_interp - MEDIA_R) / DESV_R;
                let g_norm = (g_interp - MEDIA_G) / DESV_G;
                let b_norm = (b_interp - MEDIA_B) / DESV_B;

                let idx_dst = offset_fila_dst + (x_dst as usize);
                self.buffer_nchw[idx_dst] = r_norm;
                self.buffer_nchw[stride_canal + idx_dst] = g_norm;
                self.buffer_nchw[2 * stride_canal + idx_dst] = b_norm;
            }
        }

        Ok(&self.buffer_nchw[..])
    }

    /// Transforma unas coordenadas `(u_modelo, v_modelo)` en el espacio 256×192 del modelo
    /// de vuelta a las coordenadas `(u, v)` del fotograma de entrada original según la `bbox` utilizada.
    #[inline]
    pub fn reproyectar_a_frame(
        &self,
        u_modelo: f32,
        v_modelo: f32,
        bbox: &CajaDelimitadora,
    ) -> (f32, f32) {
        let u = bbox.u_min + (u_modelo / (ANCHO_MODELO as f32)) * bbox.ancho();
        let v = bbox.v_min + (v_modelo / (ALTO_MODELO as f32)) * bbox.alto();
        (u, v)
    }
}
