//! Abstracción de sensores y soporte de grabación/reproducción para FerriteVR.
//!
//! Este crate proporciona:
//! - El trait [`FuenteFrames`], que define la interfaz común de obtención de fotogramas sincronizados
//!   de color y profundidad tanto para hardware físico (Kinect v2) como para sesiones grabadas.
//! - La estructura [`FrameSensor`], optimizada para minimizar asignaciones mediante buffers reutilizables.
//! - El formato de archivo binario comprimido con zstd `.ferrite` mediante [`GrabadorFerrite`] y [`LectorFerrite`].
//! - La implementación [`FuenteReproduccion`], que permite reproducir grabaciones respetando
//!   las marcas de tiempo originales o a máxima velocidad.
//! - Generación sintética de fotogramas para pruebas mediante [`GeneradorSinteticoSensor`].

#![deny(non_ascii_idents)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod formato;
pub mod frame;
pub mod reproduccion;
pub mod sintetico;

use std::time::Duration;

pub use ferrite_nucleo::IntrinsecosProfundidad;
pub use formato::{
    EncabezadoFerrite, GrabadorFerrite, LectorFerrite, MAGIC_FERRITE, TAMANO_ENCABEZADO,
    VERSION_FORMATO_ACTUAL,
};
pub use frame::FrameSensor;
pub use reproduccion::FuenteReproduccion;
pub use sintetico::GeneradorSinteticoSensor;

/// Errores producidos por operaciones de captura de frames, compresión y archivos de grabación.
#[derive(Debug, thiserror::Error)]
pub enum ErrorSensor {
    /// Error de entrada/salida subyacente.
    #[error("Error de entrada/salida: {0}")]
    Io(#[from] std::io::Error),

    /// El archivo de grabación .ferrite contiene un formato o cabecera inválida.
    #[error("Formato .ferrite invalido: {0}")]
    FormatoInvalido(String),

    /// Error al descomprimir datos de fotograma con zstd.
    #[error("Error de descompresion zstd: {0}")]
    Descompresion(String),

    /// Error al comprimir datos de fotograma con zstd.
    #[error("Error de compresion zstd: {0}")]
    Compresion(String),

    /// Se ha alcanzado el fin de la reproducción.
    #[error("Fin de la reproduccion")]
    FinReproduccion,

    /// Error interno del sensor Kinect o de su hilo de captura.
    #[error("Error del sensor Kinect: {0}")]
    Kinect(String),
}

/// Fuente de fotogramas sincronizados (Kinect real o archivo de grabación).
pub trait FuenteFrames: Send {
    /// Obtiene el siguiente fotograma del sensor o grabación.
    ///
    /// Si no hay fotograma disponible dentro del tiempo de `espera`, retorna `Ok(None)`.
    /// Al alcanzar el final de una reproducción sin repetición, retorna `Ok(None)` o `Err(ErrorSensor::FinReproduccion)`.
    fn siguiente_frame(&mut self, espera: Duration) -> Result<Option<FrameSensor>, ErrorSensor>;

    /// Retorna los parámetros intrínsecos de la cámara de profundidad de esta fuente.
    fn intrinsecos(&self) -> IntrinsecosProfundidad;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_creacion_frame_sensor_y_dimensiones() {
        let frame = FrameSensor::nuevo(512, 424);
        assert_eq!(frame.ancho, 512);
        assert_eq!(frame.alto, 424);
        assert_eq!(frame.total_pixeles(), 512 * 424);
        assert_eq!(frame.profundidad.len(), 512 * 424);
        assert_eq!(frame.color_registrado.len(), 512 * 424);
    }

    #[test]
    fn test_ida_y_vuelta_grabacion_ferrite() {
        let intr = IntrinsecosProfundidad {
            ancho: 4,
            alto: 4,
            fx: 10.0,
            fy: 10.0,
            cx: 2.0,
            cy: 2.0,
        };

        let mut frame_original = FrameSensor::nuevo(4, 4);
        frame_original.marca_tiempo_us = 123_456;
        for i in 0..16 {
            frame_original.profundidad[i] = (i as f32) * 100.0 + 500.0;
            frame_original.color_registrado[i] = [i as u8, 100, 200, 255];
        }

        // Grabar en memoria
        let cursor = Cursor::new(Vec::new());
        let mut grabador = GrabadorFerrite::nuevo(cursor, intr, 30.0).expect("Crear grabador");
        grabador.escribir_frame(&frame_original).expect("Escribir frame");
        grabador.vaciar().expect("Vaciar");
        let bytes = grabador.desempaquetar().into_inner();

        // Leer desde memoria
        let mut lector = LectorFerrite::abrir(Cursor::new(bytes)).expect("Abrir lector");
        assert_eq!(lector.intrinsecos().ancho, 4);
        assert_eq!(lector.intrinsecos().alto, 4);

        let frame_leido = lector.leer_frame().expect("Leer frame").expect("Debe haber frame");
        assert_eq!(frame_leido.marca_tiempo_us, frame_original.marca_tiempo_us);
        assert_eq!(frame_leido.profundidad, frame_original.profundidad);
        assert_eq!(frame_leido.color_registrado, frame_original.color_registrado);

        // Fin de archivo
        let eof = lector.leer_frame().expect("Lectura EOF");
        assert!(eof.is_none());
    }

    #[test]
    fn test_reproduccion_sin_tiempo() {
        let generador = GeneradorSinteticoSensor::default();
        let bytes = generador
            .crear_grabacion_en_memoria(3)
            .expect("Crear grabacion sintetica");

        let mut reproductor =
            FuenteReproduccion::abrir(Cursor::new(bytes))
                .expect("Abrir reproduccion")
                .con_sin_tiempo(true);

        for _ in 0..3 {
            let frame = reproductor
                .siguiente_frame(Duration::from_millis(100))
                .expect("Lectura sin error");
            assert!(frame.is_some());
        }

        let final_grabacion = reproductor
            .siguiente_frame(Duration::from_millis(100))
            .expect("Fin de grabacion");
        assert!(final_grabacion.is_none());
    }

    #[test]
    fn test_reproduccion_con_repeticion() {
        let generador = GeneradorSinteticoSensor::default();
        let bytes = generador
            .crear_grabacion_en_memoria(2)
            .expect("Crear grabacion sintetica");

        let mut reproductor =
            FuenteReproduccion::abrir(Cursor::new(bytes))
                .expect("Abrir reproduccion")
                .con_sin_tiempo(true)
                .con_repeticion(true);

        // Debe leer los 2 frames originales, reiniciar en bucle y leer otros 2
        for _ in 0..4 {
            let frame = reproductor
                .siguiente_frame(Duration::from_millis(100))
                .expect("Lectura sin error");
            assert!(frame.is_some());
        }
    }

    #[test]
    fn test_deteccion_magic_invalido() {
        let mut bytes_corruptos = vec![0u8; 48];
        bytes_corruptos[0..4].copy_from_slice(b"NOPE");
        let resultado = LectorFerrite::abrir(Cursor::new(bytes_corruptos));
        assert!(resultado.is_err());
    }

    #[test]
    fn test_fuente_frames_generador_sintetico() {
        let mut generador = GeneradorSinteticoSensor::default();
        let intr = generador.intrinsecos();
        assert_eq!(intr.ancho, 512);
        assert_eq!(intr.alto, 424);

        let f1 = generador
            .siguiente_frame(Duration::from_millis(50))
            .expect("Frame 1")
            .expect("Debe retornar frame");
        let f2 = generador
            .siguiente_frame(Duration::from_millis(50))
            .expect("Frame 2")
            .expect("Debe retornar frame");

        assert_eq!(f1.marca_tiempo_us, 0);
        assert_eq!(f2.marca_tiempo_us, 33_333); // Aprox 30 FPS (~33.3 ms)
    }

    #[test]
    fn test_reutilizacion_buffers_frame_sensor() {
        let mut frame = FrameSensor::nuevo(10, 10);
        assert_eq!(frame.total_pixeles(), 100);

        frame.profundidad[0] = 1234.5;
        frame.color_registrado[0] = [1, 2, 3, 4];
        frame.reiniciar(999);

        assert_eq!(frame.marca_tiempo_us, 999);
        assert_eq!(frame.profundidad[0], 0.0);
        assert_eq!(frame.color_registrado[0], [0, 0, 0, 255]);

        frame.asegurar_dimensiones(20, 20);
        assert_eq!(frame.total_pixeles(), 400);
        assert_eq!(frame.profundidad.len(), 400);
        assert_eq!(frame.color_registrado.len(), 400);
    }
}
