//! Codificación y decodificación del formato de grabación `.ferrite`.
//!
//! Un archivo `.ferrite` se compone de un encabezado de 48 bytes seguido de
//! una secuencia consecutiva de fotogramas comprimidos individualmente con `zstd`.

use crate::{ErrorSensor, FrameSensor};
use ferrite_nucleo::IntrinsecosProfundidad;
use std::io::{Read, Seek, SeekFrom, Write};

/// Identificador mágico de cuatro bytes que encabeza todo archivo `.ferrite`.
pub const MAGIC_FERRITE: &[u8; 4] = b"FERR";

/// Versión actual del formato `.ferrite`.
pub const VERSION_FORMATO_ACTUAL: u16 = 1;

/// Tamaño exacto del encabezado binario en bytes.
pub const TAMANO_ENCABEZADO: usize = 48;

/// Metadatos contenidos en el encabezado de un archivo `.ferrite`.
#[derive(Debug, Clone, PartialEq)]
pub struct EncabezadoFerrite {
    /// Versión del formato de archivo (actualmente 1).
    pub version: u16,
    /// Banderas de configuración (reservadas).
    pub banderas: u16,
    /// Parámetros intrínsecos de la cámara de profundidad.
    pub intrinsecos: IntrinsecosProfundidad,
    /// Tasa nominal de fotogramas por segundo del sensor (ej. 30.0 FPS).
    pub fps_nominal: f32,
}

impl EncabezadoFerrite {
    /// Serializa el encabezado en un bloque de 48 bytes en orden little-endian.
    pub fn a_bytes(&self) -> [u8; TAMANO_ENCABEZADO] {
        let mut buffer = [0u8; TAMANO_ENCABEZADO];
        buffer[0..4].copy_from_slice(MAGIC_FERRITE);
        buffer[4..6].copy_from_slice(&self.version.to_le_bytes());
        buffer[6..8].copy_from_slice(&self.banderas.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.intrinsecos.ancho.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.intrinsecos.alto.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.intrinsecos.fx.to_le_bytes());
        buffer[20..24].copy_from_slice(&self.intrinsecos.fy.to_le_bytes());
        buffer[24..28].copy_from_slice(&self.intrinsecos.cx.to_le_bytes());
        buffer[28..32].copy_from_slice(&self.intrinsecos.cy.to_le_bytes());
        buffer[32..36].copy_from_slice(&self.fps_nominal.to_le_bytes());
        // Bytes 36..48 quedan en ceros para extensiones futuras
        buffer
    }

    /// Deserializa y valida un encabezado desde un slice de 48 bytes.
    pub fn desde_bytes(bytes: &[u8]) -> Result<Self, ErrorSensor> {
        if bytes.len() < TAMANO_ENCABEZADO {
            return Err(ErrorSensor::FormatoInvalido(
                "Longitud insuficiente para encabezado .ferrite".into(),
            ));
        }

        if &bytes[0..4] != MAGIC_FERRITE {
            return Err(ErrorSensor::FormatoInvalido(format!(
                "Identificador magico incorrecto: esperado {:?}, recibido {:?}",
                MAGIC_FERRITE,
                &bytes[0..4]
            )));
        }

        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != VERSION_FORMATO_ACTUAL {
            return Err(ErrorSensor::FormatoInvalido(format!(
                "Version de formato .ferrite no soportada: {version} (esperada {VERSION_FORMATO_ACTUAL})"
            )));
        }

        let banderas = u16::from_le_bytes([bytes[6], bytes[7]]);
        let ancho = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let alto = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        if ancho == 0 || alto == 0 {
            return Err(ErrorSensor::FormatoInvalido(
                "Dimensiones de imagen invalidas en encabezado".into(),
            ));
        }

        let fx = f32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let fy = f32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        let cx = f32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let cy = f32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
        let fps_nominal = f32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);

        Ok(Self {
            version,
            banderas,
            intrinsecos: IntrinsecosProfundidad {
                ancho,
                alto,
                fx,
                fy,
                cx,
                cy,
            },
            fps_nominal,
        })
    }
}

/// Grabador de sesiones en formato `.ferrite`.
pub struct GrabadorFerrite<W: Write> {
    escritor: W,
    encabezado: EncabezadoFerrite,
    buffer_crudo: Vec<u8>,
    nivel_compresion: i32,
}

impl<W: Write> GrabadorFerrite<W> {
    /// Crea un nuevo grabador, escribe el encabezado inicial en el flujo de salida y preasigna buffers.
    pub fn nuevo(
        mut escritor: W,
        intrinsecos: IntrinsecosProfundidad,
        fps_nominal: f32,
    ) -> Result<Self, ErrorSensor> {
        let encabezado = EncabezadoFerrite {
            version: VERSION_FORMATO_ACTUAL,
            banderas: 0,
            intrinsecos,
            fps_nominal,
        };

        escritor.write_all(&encabezado.a_bytes())?;

        let bytes_por_frame = (intrinsecos.ancho as usize) * (intrinsecos.alto as usize) * 8;
        Ok(Self {
            escritor,
            encabezado,
            buffer_crudo: Vec::with_capacity(bytes_por_frame),
            nivel_compresion: 3, // Nivel óptimo rápido de zstd para tiempo real
        })
    }

    /// Ajusta el nivel de compresión de `zstd` (1 para máxima velocidad, hasta 22 para máxima compresión).
    pub fn con_nivel_compresion(mut self, nivel: i32) -> Self {
        self.nivel_compresion = nivel;
        self
    }

    /// Comprime y escribe un fotograma en el archivo de salida.
    pub fn escribir_frame(&mut self, frame: &FrameSensor) -> Result<(), ErrorSensor> {
        let total_pixeles = (self.encabezado.intrinsecos.ancho as usize)
            * (self.encabezado.intrinsecos.alto as usize);

        if frame.profundidad.len() != total_pixeles || frame.color_registrado.len() != total_pixeles
        {
            return Err(ErrorSensor::FormatoInvalido(format!(
                "El fotograma no coincide con las dimensiones configuradas: esperado {total_pixeles} pixeles"
            )));
        }

        // Empaquetar datos crudos: [profundidad: 4 bytes por pixel] seguido de [color: 4 bytes RGBA por pixel]
        let tamano_total = total_pixeles * 8;
        self.buffer_crudo.clear();
        self.buffer_crudo.reserve(tamano_total);

        for &prof in &frame.profundidad {
            self.buffer_crudo.extend_from_slice(&prof.to_le_bytes());
        }

        for &rgba in &frame.color_registrado {
            self.buffer_crudo.extend_from_slice(&rgba);
        }

        // Comprimir con zstd
        let datos_comprimidos = zstd::encode_all(&self.buffer_crudo[..], self.nivel_compresion)
            .map_err(|e| ErrorSensor::Compresion(e.to_string()))?;

        // Escribir marca de tiempo (8 bytes) y longitud comprimida (4 bytes)
        self.escritor
            .write_all(&frame.marca_tiempo_us.to_le_bytes())?;
        self.escritor
            .write_all(&(datos_comprimidos.len() as u32).to_le_bytes())?;
        self.escritor.write_all(&datos_comprimidos)?;

        Ok(())
    }

    /// Asegura que todos los datos pendientes se sincronicen en el medio de salida.
    pub fn vaciar(&mut self) -> Result<(), ErrorSensor> {
        self.escritor.flush()?;
        Ok(())
    }

    /// Consume el grabador y retorna el escritor subyacente.
    pub fn desempaquetar(self) -> W {
        self.escritor
    }
}

/// Lector y decodificador de archivos en formato `.ferrite`.
pub struct LectorFerrite<R: Read> {
    lector: R,
    encabezado: EncabezadoFerrite,
    buffer_comprimido: Vec<u8>,
    buffer_descomprimido: Vec<u8>,
}

impl<R: Read> LectorFerrite<R> {
    /// Abre un flujo de entrada, lee y valida el encabezado de 48 bytes.
    pub fn abrir(mut lector: R) -> Result<Self, ErrorSensor> {
        let mut bytes_encabezado = [0u8; TAMANO_ENCABEZADO];
        lector.read_exact(&mut bytes_encabezado)?;

        let encabezado = EncabezadoFerrite::desde_bytes(&bytes_encabezado)?;
        let total_pixeles =
            (encabezado.intrinsecos.ancho as usize) * (encabezado.intrinsecos.alto as usize);
        let bytes_descomprimidos = total_pixeles * 8;

        Ok(Self {
            lector,
            encabezado,
            buffer_comprimido: Vec::new(),
            buffer_descomprimido: Vec::with_capacity(bytes_descomprimidos),
        })
    }

    /// Retorna los metadatos del encabezado leído.
    #[inline]
    pub fn encabezado(&self) -> &EncabezadoFerrite {
        &self.encabezado
    }

    /// Retorna los parámetros intrínsecos de la cámara configurados en la grabación.
    #[inline]
    pub fn intrinsecos(&self) -> IntrinsecosProfundidad {
        self.encabezado.intrinsecos
    }

    /// Lee el siguiente fotograma reutilizando la memoria del objeto `FrameSensor` destino.
    ///
    /// Retorna `Ok(true)` si se leyó un fotograma exitosamente, o `Ok(false)` si se alcanzó el fin del archivo.
    pub fn leer_frame_en(&mut self, destino: &mut FrameSensor) -> Result<bool, ErrorSensor> {
        let mut bytes_marca = [0u8; 8];
        match self.lector.read_exact(&mut bytes_marca) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(false),
            Err(e) => return Err(ErrorSensor::Io(e)),
        }

        let marca_tiempo_us = u64::from_le_bytes(bytes_marca);

        let mut bytes_len = [0u8; 4];
        self.lector.read_exact(&mut bytes_len)?;
        let len_comprimido = u32::from_le_bytes(bytes_len) as usize;

        self.buffer_comprimido.resize(len_comprimido, 0);
        self.lector.read_exact(&mut self.buffer_comprimido)?;

        self.buffer_descomprimido.clear();
        zstd::stream::copy_decode(&self.buffer_comprimido[..], &mut self.buffer_descomprimido)
            .map_err(|e| ErrorSensor::Descompresion(e.to_string()))?;

        let ancho = self.encabezado.intrinsecos.ancho;
        let alto = self.encabezado.intrinsecos.alto;
        let total_pixeles = (ancho as usize) * (alto as usize);
        let esperado = total_pixeles * 8;

        if self.buffer_descomprimido.len() != esperado {
            return Err(ErrorSensor::FormatoInvalido(format!(
                "Tamano descomprimido incongruente: esperado {esperado} bytes, recibido {}",
                self.buffer_descomprimido.len()
            )));
        }

        destino.asegurar_dimensiones(ancho, alto);
        destino.marca_tiempo_us = marca_tiempo_us;

        // Extraer profundidad (primeros total_pixeles * 4 bytes)
        for i in 0..total_pixeles {
            let offset = i * 4;
            let bytes_f32 = [
                self.buffer_descomprimido[offset],
                self.buffer_descomprimido[offset + 1],
                self.buffer_descomprimido[offset + 2],
                self.buffer_descomprimido[offset + 3],
            ];
            destino.profundidad[i] = f32::from_le_bytes(bytes_f32);
        }

        // Extraer color RGBA (siguientes total_pixeles * 4 bytes)
        let offset_color = total_pixeles * 4;
        for i in 0..total_pixeles {
            let offset = offset_color + (i * 4);
            destino.color_registrado[i] = [
                self.buffer_descomprimido[offset],
                self.buffer_descomprimido[offset + 1],
                self.buffer_descomprimido[offset + 2],
                self.buffer_descomprimido[offset + 3],
            ];
        }

        Ok(true)
    }

    /// Lee y retorna el siguiente fotograma asignando una nueva instancia de [`FrameSensor`].
    pub fn leer_frame(&mut self) -> Result<Option<FrameSensor>, ErrorSensor> {
        let mut frame = FrameSensor::nuevo(
            self.encabezado.intrinsecos.ancho,
            self.encabezado.intrinsecos.alto,
        );
        if self.leer_frame_en(&mut frame)? {
            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }
}

impl<R: Read + Seek> LectorFerrite<R> {
    /// Rebobina el lector al primer fotograma de la grabación (justo después del encabezado).
    pub fn rebobinar(&mut self) -> Result<(), ErrorSensor> {
        self.lector
            .seek(SeekFrom::Start(TAMANO_ENCABEZADO as u64))
            .map_err(ErrorSensor::Io)?;
        Ok(())
    }
}
