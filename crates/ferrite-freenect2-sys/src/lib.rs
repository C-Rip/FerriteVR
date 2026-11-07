//! Enlace FFI seguro con `libfreenect2` para comunicación con Kinect v2 en FerriteVR.
//!
//! Este crate proporciona acceso directo al hardware del sensor Kinect v2 (Xbox One)
//! mediante un shim en C++ que gestiona la captura sincronizada de color y profundidad,
//! así como el algoritmo de alineación geométrica (*Registration*).
//!
//! Todo el acceso a hardware está protegido bajo la característica `kinect`. Sin esta
//! característica activada, el crate expone estructuras inertes para permitir la compilación
//! en entornos de integración continua o máquinas sin bibliotecas de Kinect instaladas.

#![allow(unsafe_code)]

#[cfg(feature = "kinect")]
use std::pin::Pin;
#[cfg(feature = "kinect")]
use thiserror::Error;

/// Errores producidos durante la comunicación con el sensor Kinect v2 o libfreenect2.
#[cfg(feature = "kinect")]
#[derive(Debug, Error)]
pub enum ErrorKinect {
    /// Error propagado desde el shim C++ o libfreenect2.
    #[error("Error de libfreenect2 / C++: {0}")]
    Ffi(#[from] cxx::Exception),
    /// Parámetro de búfer o configuración inválido.
    #[error("Parámetro inválido: {0}")]
    ParametroInvalido(&'static str),
}

/// Parámetros intrínsecos de la cámara infrarroja / profundidad del Kinect v2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParametrosIntrinsecos {
    /// Longitud focal horizontal en píxeles.
    pub fx: f32,
    /// Longitud focal vertical en píxeles.
    pub fy: f32,
    /// Punto principal horizontal en píxeles.
    pub cx: f32,
    /// Punto principal vertical en píxeles.
    pub cy: f32,
}

/// Modo de aceleración para el procesamiento y decodificación de paquetes de profundidad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModoPipeline {
    /// Selección automática según bibliotecas detectadas (OpenGL -> OpenCL -> CPU).
    #[default]
    PorDefecto = 0,
    /// Decodificación pura en procesador (CPU).
    Cpu = 1,
    /// Decodificación acelerada por hardware mediante sombreadores OpenGL.
    OpenGl = 2,
    /// Decodificación acelerada por OpenCL.
    OpenCl = 3,
}

/// Nivel de verbosidad del registro de libfreenect2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NivelRegistro {
    /// Desactiva todos los mensajes de libfreenect2.
    Silencioso = 0,
    /// Únicamente mensajes de error crítico.
    Error = 1,
    /// Advertencias y errores (por defecto).
    #[default]
    Advertencia = 2,
    /// Mensajes informativos generales.
    Info = 3,
    /// Mensajes detallados de depuración.
    Depuracion = 4,
}

#[cfg(feature = "kinect")]
#[cxx::bridge(namespace = "ferrite::freenect2")]
mod ffi {
    unsafe extern "C++" {
        include!("shim.h");

        type DispositivoKinect;

        fn crear_dispositivo(modo_pipeline: i32) -> Result<UniquePtr<DispositivoKinect>>;
        fn iniciar(self: Pin<&mut DispositivoKinect>) -> Result<()>;
        fn detener(self: Pin<&mut DispositivoKinect>) -> Result<()>;
        fn esperar_y_registrar(
            self: Pin<&mut DispositivoKinect>,
            tiempo_espera_ms: i32,
            buffer_profundidad: &mut [f32],
            buffer_color_rgba: &mut [u8],
        ) -> Result<bool>;
        fn obtener_intrinsecos(
            self: &DispositivoKinect,
            fx: &mut f32,
            fy: &mut f32,
            cx: &mut f32,
            cy: &mut f32,
        ) -> Result<()>;
        fn obtener_numero_serie(self: &DispositivoKinect) -> String;
        fn obtener_version_firmware(self: &DispositivoKinect) -> String;

        fn configurar_nivel_registro(nivel: i32);
    }
}

/// Configura el nivel de detalle de registro en consola para libfreenect2.
#[cfg(feature = "kinect")]
pub fn configurar_registro(nivel: NivelRegistro) {
    ffi::configurar_nivel_registro(nivel as i32);
}

/// Dispositivo Kinect v2 conectado y gestionado de forma segura con semántica RAII.
#[cfg(feature = "kinect")]
pub struct DispositivoKinect {
    crudo: cxx::UniquePtr<ffi::DispositivoKinect>,
}

// SEGURIDAD: El objeto `DispositivoKinect` posee exclusivamente sus estructuras internas en C++
// y la interfaz Rust garantiza acceso mutable exclusivo (`&mut`) para métodos de captura y control.
#[cfg(feature = "kinect")]
unsafe impl Send for DispositivoKinect {}

#[cfg(feature = "kinect")]
impl DispositivoKinect {
    /// Abre el primer sensor Kinect v2 disponible utilizando el pipeline especificado.
    pub fn abrir(modo: ModoPipeline) -> Result<Self, ErrorKinect> {
        let crudo = ffi::crear_dispositivo(modo as i32)?;
        Ok(Self { crudo })
    }

    /// Inicia los flujos de recepción USB de color y profundidad.
    pub fn iniciar(&mut self) -> Result<(), ErrorKinect> {
        let mut pin = self.obtener_pin()?;
        pin.as_mut().iniciar()?;
        Ok(())
    }

    /// Detiene los flujos de recepción USB del sensor.
    pub fn detener(&mut self) -> Result<(), ErrorKinect> {
        let mut pin = self.obtener_pin()?;
        pin.as_mut().detener()?;
        Ok(())
    }

    /// Espera hasta que un fotograma sincronizado esté disponible y realiza la alineación geométrica.
    ///
    /// Copia la profundidad corregida en milímetros en `buffer_profundidad` (mínimo 217,088 elementos `f32`)
    /// y la imagen de color registrada en formato RGBA en `buffer_color_rgba` (mínimo 868,352 bytes).
    ///
    /// Retorna `true` si se recibió y registró un fotograma, o `false` si expiró el tiempo de espera.
    pub fn esperar_y_registrar(
        &mut self,
        tiempo_espera_ms: i32,
        buffer_profundidad: &mut [f32],
        buffer_color_rgba: &mut [u8],
    ) -> Result<bool, ErrorKinect> {
        let mut pin = self.obtener_pin()?;
        let recibido = pin.as_mut().esperar_y_registrar(
            tiempo_espera_ms,
            buffer_profundidad,
            buffer_color_rgba,
        )?;
        Ok(recibido)
    }

    /// Obtiene los parámetros intrínsecos de la cámara infrarroja del dispositivo.
    pub fn intrinsecos(&self) -> Result<ParametrosIntrinsecos, ErrorKinect> {
        let ref_crudo = self.obtener_ref()?;
        let mut fx = 0.0f32;
        let mut fy = 0.0f32;
        let mut cx = 0.0f32;
        let mut cy = 0.0f32;
        ref_crudo.obtener_intrinsecos(&mut fx, &mut fy, &mut cx, &mut cy)?;
        Ok(ParametrosIntrinsecos { fx, fy, cx, cy })
    }

    /// Retorna el número de serie de fábrica del sensor Kinect v2.
    pub fn numero_serie(&self) -> Result<String, ErrorKinect> {
        let ref_crudo = self.obtener_ref()?;
        Ok(ref_crudo.obtener_numero_serie())
    }

    /// Retorna la versión del firmware instalado en el sensor.
    pub fn version_firmware(&self) -> Result<String, ErrorKinect> {
        let ref_crudo = self.obtener_ref()?;
        Ok(ref_crudo.obtener_version_firmware())
    }

    fn obtener_pin(&mut self) -> Result<Pin<&mut ffi::DispositivoKinect>, ErrorKinect> {
        self.crudo
            .as_mut()
            .ok_or(ErrorKinect::ParametroInvalido("Puntero a dispositivo Kinect nulo"))
    }

    fn obtener_ref(&self) -> Result<&ffi::DispositivoKinect, ErrorKinect> {
        self.crudo
            .as_ref()
            .ok_or(ErrorKinect::ParametroInvalido("Puntero a dispositivo Kinect nulo"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modo_pipeline_valores_y_defecto() {
        assert_eq!(ModoPipeline::default(), ModoPipeline::PorDefecto);
        assert_eq!(ModoPipeline::PorDefecto as i32, 0);
        assert_eq!(ModoPipeline::Cpu as i32, 1);
        assert_eq!(ModoPipeline::OpenGl as i32, 2);
        assert_eq!(ModoPipeline::OpenCl as i32, 3);
    }

    #[test]
    fn test_nivel_registro_valores_y_defecto() {
        assert_eq!(NivelRegistro::default(), NivelRegistro::Advertencia);
        assert_eq!(NivelRegistro::Silencioso as i32, 0);
        assert_eq!(NivelRegistro::Error as i32, 1);
        assert_eq!(NivelRegistro::Advertencia as i32, 2);
        assert_eq!(NivelRegistro::Info as i32, 3);
        assert_eq!(NivelRegistro::Depuracion as i32, 4);
    }

    #[test]
    fn test_parametros_intrinsecos_creacion_y_copia() {
        let p = ParametrosIntrinsecos {
            fx: 367.0,
            fy: 367.0,
            cx: 256.0,
            cy: 212.0,
        };
        let copia = p;
        assert_eq!(p, copia);
        assert_eq!(copia.fx, 367.0);
    }

    #[cfg(feature = "kinect")]
    #[test]
    fn test_error_kinect_formato_display() {
        let err = ErrorKinect::ParametroInvalido("buffer muy pequeno");
        assert_eq!(format!("{err}"), "Parámetro inválido: buffer muy pequeno");
    }
}
