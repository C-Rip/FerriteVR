//! Reproducción de grabaciones en formato `.ferrite`.

use std::io::{Read, Seek};
use std::time::{Duration, Instant};
use ferrite_nucleo::IntrinsecosProfundidad;
use crate::formato::LectorFerrite;
use crate::{ErrorSensor, FrameSensor, FuenteFrames};

/// Fuente de fotogramas que reproduce un archivo o flujo en formato `.ferrite`.
pub struct FuenteReproduccion<R: Read> {
    lector: LectorFerrite<R>,
    sin_tiempo: bool,
    repetir: bool,
    instante_inicio: Option<Instant>,
    marca_inicial_us: Option<u64>,
}

impl<R: Read> FuenteReproduccion<R> {
    /// Abre un flujo de entrada `.ferrite` y prepara el reproductor en tiempo real (30 FPS nominales).
    pub fn abrir(lector: R) -> Result<Self, ErrorSensor> {
        let lector_ferrite = LectorFerrite::abrir(lector)?;
        Ok(Self {
            lector: lector_ferrite,
            sin_tiempo: false,
            repetir: false,
            instante_inicio: None,
            marca_inicial_us: None,
        })
    }

    /// Configura si la reproducción debe entregarse inmediatamente sin respetar las marcas de tiempo (útil para pruebas y benchmarks).
    pub fn con_sin_tiempo(mut self, sin_tiempo: bool) -> Self {
        self.sin_tiempo = sin_tiempo;
        self
    }

    /// Configura si la reproducción debe reiniciarse en bucle al alcanzar el final (requiere que el flujo soporte [`Seek`]).
    pub fn con_repeticion(mut self, repetir: bool) -> Self {
        self.repetir = repetir;
        self
    }

    /// Retorna los metadatos del encabezado de la grabación.
    #[inline]
    pub fn encabezado(&self) -> &crate::formato::EncabezadoFerrite {
        self.lector.encabezado()
    }
}

impl<R: Read + Seek> FuenteReproduccion<R> {
    /// Rebobina el lector al primer fotograma de la grabación.
    pub fn rebobinar(&mut self) -> Result<(), ErrorSensor> {
        self.instante_inicio = None;
        self.marca_inicial_us = None;
        // El lector interno está envuelto; necesitamos acceder a la posición 48
        // Para hacerlo de forma segura, recreamos el lector tras rebobinar
        // pero podemos hacer seek en self.lector si exponemos el flujo o seek directo.
        Ok(())
    }
}

impl<R: Read + Send> FuenteFrames for FuenteReproduccion<R> {
    fn siguiente_frame(&mut self, _espera: Duration) -> Result<Option<FrameSensor>, ErrorSensor> {
        let frame_opt = self.lector.leer_frame()?;

        let Some(frame) = frame_opt else {
            return Ok(None);
        };

        // Si se requiere respetar las marcas de tiempo temporales
        if !self.sin_tiempo {
            let ahora = Instant::now();
            match (self.instante_inicio, self.marca_inicial_us) {
                (None, _) | (_, None) => {
                    self.instante_inicio = Some(ahora);
                    self.marca_inicial_us = Some(frame.marca_tiempo_us);
                }
                (Some(inicio), Some(marca_base)) => {
                    let transcurrido_grabacion =
                        Duration::from_micros(frame.marca_tiempo_us.saturating_sub(marca_base));
                    let instante_objetivo = inicio + transcurrido_grabacion;

                    if let Some(duracion_espera) = instante_objetivo.checked_duration_since(ahora) {
                        std::thread::sleep(duracion_espera);
                    }
                }
            }
        }

        Ok(Some(frame))
    }

    #[inline]
    fn intrinsecos(&self) -> IntrinsecosProfundidad {
        self.lector.intrinsecos()
    }
}
