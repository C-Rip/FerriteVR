//! Detección y manipulación de cajas delimitadoras (bounding boxes) sobre mapas de profundidad.

/// Caja delimitadora rectangular en coordenadas continuas de píxel `(u, v)`.
///
/// ```
/// use ferrite_pose::CajaDelimitadora;
///
/// let caja = CajaDelimitadora::nueva(10.0, 20.0, 50.0, 100.0);
/// assert_eq!(caja.ancho(), 40.0);
/// assert_eq!(caja.alto(), 80.0);
/// assert_eq!(caja.centro(), (30.0, 60.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CajaDelimitadora {
    /// Límite horizontal izquierdo (mínimo `u`).
    pub u_min: f32,
    /// Límite vertical superior (mínimo `v`).
    pub v_min: f32,
    /// Límite horizontal derecho (máximo `u`).
    pub u_max: f32,
    /// Límite vertical inferior (máximo `v`).
    pub v_max: f32,
}

impl CajaDelimitadora {
    /// Crea una nueva caja delimitadora validando que los mínimos sean menores o iguales a los máximos.
    pub fn nueva(u_min: f32, v_min: f32, u_max: f32, v_max: f32) -> Self {
        Self {
            u_min: u_min.min(u_max),
            v_min: v_min.min(v_max),
            u_max: u_min.max(u_max),
            v_max: v_min.max(v_max),
        }
    }

    /// Retorna el ancho de la caja en píxeles.
    #[inline]
    pub fn ancho(&self) -> f32 {
        (self.u_max - self.u_min).max(0.0)
    }

    /// Retorna el alto de la caja en píxeles.
    #[inline]
    pub fn alto(&self) -> f32 {
        (self.v_max - self.v_min).max(0.0)
    }

    /// Retorna el centro geométrico `(u_centro, v_centro)` de la caja.
    #[inline]
    pub fn centro(&self) -> (f32, f32) {
        (
            (self.u_min + self.u_max) * 0.5,
            (self.v_min + self.v_max) * 0.5,
        )
    }

    /// Expande la caja aplicando una fracción de margen proporcional a su tamaño y la acota a los límites dados.
    pub fn con_margen(&self, margen: f32, limite_u: f32, limite_v: f32) -> Self {
        let expansion_u = self.ancho() * margen * 0.5;
        let expansion_v = self.alto() * margen * 0.5;

        Self {
            u_min: (self.u_min - expansion_u).max(0.0),
            v_min: (self.v_min - expansion_v).max(0.0),
            u_max: (self.u_max + expansion_u).min(limite_u),
            v_max: (self.v_max + expansion_v).min(limite_v),
        }
    }

    /// Ajusta la relación de aspecto de la caja (`ancho / alto`) al valor objetivo sin recortar el contenido.
    ///
    /// Para RTMPose (256 alto × 192 ancho), la proporción objetivo es `192.0 / 256.0 = 0.75`.
    pub fn ajustar_proporcion(
        &self,
        proporcion_objetivo: f32,
        limite_u: f32,
        limite_v: f32,
    ) -> Self {
        let (cu, cv) = self.centro();
        let mut ancho = self.ancho();
        let mut alto = self.alto();

        if ancho <= 0.0 || alto <= 0.0 {
            return *self;
        }

        let proporcion_actual = ancho / alto;

        if proporcion_actual < proporcion_objetivo {
            // Demasiado estrecha: expandir ancho
            ancho = alto * proporcion_objetivo;
        } else {
            // Demasiado ancha: expandir alto
            alto = ancho / proporcion_objetivo;
        }

        let u_min = (cu - ancho * 0.5).max(0.0);
        let u_max = (cu + ancho * 0.5).min(limite_u);
        let v_min = (cv - alto * 0.5).max(0.0);
        let v_max = (cv + alto * 0.5).min(limite_v);

        Self {
            u_min,
            v_min,
            u_max,
            v_max,
        }
    }

    /// Determina si un punto en coordenadas `(u, v)` se encuentra contenido en la caja.
    #[inline]
    pub fn contiene(&self, u: f32, v: f32) -> bool {
        u >= self.u_min && u <= self.u_max && v >= self.v_min && v <= self.v_max
    }
}

/// Estima la caja delimitadora del sujeto humano principal analizando el mapa de profundidad.
///
/// Filtra los píxeles en el rango de profundidad útil `[distancia_min_mm, distancia_max_mm]`,
/// aísla el grupo en primer plano (rechazando paredes de fondo) y expande la caja con el margen indicado.
pub fn estimar_bbox_profundidad(
    profundidad: &[f32],
    ancho: u32,
    alto: u32,
    distancia_min_mm: f32,
    distancia_max_mm: f32,
    margen: f32,
) -> Option<CajaDelimitadora> {
    let total = (ancho as usize) * (alto as usize);
    if profundidad.len() != total || total == 0 {
        return None;
    }

    // 1. Recolectar valores de profundidad dentro del rango válido
    let mut profundidades_validas: Vec<f32> = profundidad
        .iter()
        .copied()
        .filter(|&p| p >= distancia_min_mm && p <= distancia_max_mm && p.is_finite())
        .collect();

    // Requerimos al menos 100 píxeles para considerar la presencia de un sujeto
    if profundidades_validas.len() < 100 {
        return None;
    }

    // 2. Encontrar la profundidad del sujeto en primer plano mediante el percentil 2 (ignora ruido espurio)
    // El sujeto humano en una habitación suele ocupar entre el 5% y 25% de los píxeles de la escena.
    profundidades_validas.sort_by(|a, b| a.total_cmp(b));
    let idx_primer_plano = (profundidades_validas.len() * 2) / 100;
    let prof_referencia = profundidades_validas[idx_primer_plano];

    // Una persona de pie típicamente ocupa una banda de profundidad de hasta ~600 mm
    let banda_min = (prof_referencia - 350.0).max(distancia_min_mm);
    let banda_max = prof_referencia + 450.0;

    let mut u_min = ancho as f32;
    let mut u_max = 0.0f32;
    let mut v_min = alto as f32;
    let mut v_max = 0.0f32;
    let mut contador = 0usize;

    for v in 0..alto {
        for u in 0..ancho {
            let idx = (v * ancho + u) as usize;
            let p = profundidad[idx];
            if p >= banda_min && p <= banda_max {
                let u_f = u as f32;
                let v_f = v as f32;
                if u_f < u_min {
                    u_min = u_f;
                }
                if u_f > u_max {
                    u_max = u_f;
                }
                if v_f < v_min {
                    v_min = v_f;
                }
                if v_f > v_max {
                    v_max = v_f;
                }
                contador += 1;
            }
        }
    }

    if contador < 80 {
        return None;
    }

    let caja_cruda = CajaDelimitadora::nueva(u_min, v_min, u_max, v_max);
    let caja_con_margen = caja_cruda.con_margen(margen, (ancho - 1) as f32, (alto - 1) as f32);
    // Relación de aspecto estándar 192 / 256 = 0.75 para RTMPose
    let proporcion_rtmpose = 192.0 / 256.0;
    Some(caja_con_margen.ajustar_proporcion(
        proporcion_rtmpose,
        (ancho - 1) as f32,
        (alto - 1) as f32,
    ))
}
