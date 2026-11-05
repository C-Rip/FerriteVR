//! Definición del fotograma sincronizado de color y profundidad.

/// Fotograma capturado o reproducido que combina profundidad y color registrado.
///
/// La imagen de color está alineada píxel a píxel con el mapa de profundidad.
/// En el sensor Kinect v2, la resolución estándar es de 512 × 424 píxeles a 30 FPS.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameSensor {
    /// Marca de tiempo en microsegundos desde el inicio de la sesión o época del sistema.
    pub marca_tiempo_us: u64,
    /// Mapa de profundidad en milímetros (típicamente 512 × 424 valores `f32`).
    /// Un valor menor o igual a `0.0`, o `NaN`, indica que el píxel carece de medición válida.
    pub profundidad: Vec<f32>,
    /// Imagen de color RGBA alineada geométricamente con la profundidad (512 × 424 píxeles).
    pub color_registrado: Vec<[u8; 4]>,
    /// Ancho del fotograma en píxeles.
    pub ancho: u32,
    /// Alto del fotograma en píxeles.
    pub alto: u32,
}

impl FrameSensor {
    /// Crea un nuevo fotograma con capacidad preasignada para las dimensiones indicadas.
    ///
    /// Todos los píxeles de profundidad se inicializan en `0.0` y los de color en negro opaco `[0, 0, 0, 255]`.
    ///
    /// ```
    /// use ferrite_sensor::FrameSensor;
    ///
    /// let frame = FrameSensor::nuevo(512, 424);
    /// assert_eq!(frame.ancho, 512);
    /// assert_eq!(frame.alto, 424);
    /// assert_eq!(frame.profundidad.len(), 512 * 424);
    /// assert_eq!(frame.color_registrado.len(), 512 * 424);
    /// ```
    pub fn nuevo(ancho: u32, alto: u32) -> Self {
        let total = (ancho as usize) * (alto as usize);
        Self {
            marca_tiempo_us: 0,
            profundidad: vec![0.0; total],
            color_registrado: vec![[0, 0, 0, 255]; total],
            ancho,
            alto,
        }
    }

    /// Retorna la cantidad total de píxeles esperados (`ancho * alto`).
    #[inline]
    pub fn total_pixeles(&self) -> usize {
        (self.ancho as usize) * (self.alto as usize)
    }

    /// Reinicia los valores del fotograma asignando una nueva marca temporal sin reasignar memoria.
    #[inline]
    pub fn reiniciar(&mut self, marca_tiempo_us: u64) {
        self.marca_tiempo_us = marca_tiempo_us;
        self.profundidad.fill(0.0);
        self.color_registrado.fill([0, 0, 0, 255]);
    }

    /// Asegura que los vectores internos tengan la capacidad exacta para `ancho * alto`.
    ///
    /// Si las dimensiones ya coinciden, no realiza ninguna asignación de memoria.
    pub fn asegurar_dimensiones(&mut self, ancho: u32, alto: u32) {
        let total = (ancho as usize) * (alto as usize);
        self.ancho = ancho;
        self.alto = alto;
        if self.profundidad.len() != total {
            self.profundidad.resize(total, 0.0);
        }
        if self.color_registrado.len() != total {
            self.color_registrado.resize(total, [0, 0, 0, 255]);
        }
    }
}
