//! Generación sintética de fotogramas de prueba sin hardware Kinect.

use std::f32::consts::PI;
use std::io::Cursor;
use std::time::Duration;
use ferrite_nucleo::IntrinsecosProfundidad;
use crate::formato::GrabadorFerrite;
use crate::{ErrorSensor, FrameSensor, FuenteFrames};

/// Generador determinista de fotogramas sintéticos de color y profundidad para pruebas.
///
/// Modela una escena 3D controlada con un fondo distante (3.0 m) y una silueta humanoide
/// centrada a ~2.0 m que oscila periódicamente simulando un paso de marcha en el sitio.
#[derive(Debug, Clone)]
pub struct GeneradorSinteticoSensor {
    intrinsecos: IntrinsecosProfundidad,
    fps: f32,
    fotograma_actual: u64,
}

impl Default for GeneradorSinteticoSensor {
    fn default() -> Self {
        Self {
            intrinsecos: IntrinsecosProfundidad::default(),
            fps: 30.0,
            fotograma_actual: 0,
        }
    }
}

impl GeneradorSinteticoSensor {
    /// Crea un nuevo generador con los parámetros intrínsecos y tasa de cuadros especificados.
    pub fn nuevo(intrinsecos: IntrinsecosProfundidad, fps: f32) -> Self {
        Self {
            intrinsecos,
            fps: if fps > 0.0 { fps } else { 30.0 },
            fotograma_actual: 0,
        }
    }

    /// Genera un fotograma sintético determinista correspondiente a la marca temporal dada.
    pub fn generar_fotograma(&self, marca_tiempo_us: u64) -> FrameSensor {
        let ancho = self.intrinsecos.ancho;
        let alto = self.intrinsecos.alto;
        let mut frame = FrameSensor::nuevo(ancho, alto);
        frame.marca_tiempo_us = marca_tiempo_us;

        let t_seg = (marca_tiempo_us as f32) / 1_000_000.0;
        let fase_marcha = 2.0 * PI * 1.0 * t_seg; // 1 Hz (un paso completo por segundo)

        // Fondo a 3000 mm (3.0 m) de color gris oscuro
        let fondo_prof_mm = 3000.0;
        let fondo_color = [35, 35, 40, 255];
        frame.profundidad.fill(fondo_prof_mm);
        frame.color_registrado.fill(fondo_color);

        let cx = self.intrinsecos.cx;
        let cy = self.intrinsecos.cy;

        // Cabeza: esfera centrada en (cx, cy - 120), radio 24 px, prof ~2000 mm
        let centro_cabeza_u = cx;
        let centro_cabeza_v = cy - 120.0;
        let radio_cabeza_cuad = 24.0 * 24.0;
        let color_piel = [225, 185, 150, 255];

        // Torso: rectángulo centrado
        let torso_u_min = (cx - 36.0) as u32;
        let torso_u_max = (cx + 36.0) as u32;
        let torso_v_min = (cy - 95.0) as u32;
        let torso_v_max = (cy + 25.0) as u32;
        let color_torso = [45, 110, 185, 255];

        // Pierna izquierda y derecha con oscilación periódica en profundidad
        let pierna_izq_prof = 2000.0 + 80.0 * fase_marcha.sin();
        let pierna_der_prof = 2000.0 - 80.0 * fase_marcha.sin();
        let color_pantalones = [30, 45, 60, 255];

        let pierna_v_min = (cy + 26.0) as u32;
        let pierna_v_max = (cy + 170.0).min((alto - 1) as f32) as u32;

        let pierna_izq_u_min = (cx - 32.0) as u32;
        let pierna_izq_u_max = (cx - 8.0) as u32;

        let pierna_der_u_min = (cx + 8.0) as u32;
        let pierna_der_u_max = (cx + 32.0) as u32;

        // Brazos a los costados
        let brazo_izq_u_min = (cx - 56.0) as u32;
        let brazo_izq_u_max = (cx - 37.0) as u32;
        let brazo_der_u_min = (cx + 37.0) as u32;
        let brazo_der_u_max = (cx + 56.0) as u32;
        let brazo_v_min = (cy - 90.0) as u32;
        let brazo_v_max = (cy + 20.0) as u32;
        let color_brazo = [40, 95, 160, 255];

        // Pies / Zapatos
        let pie_v_min = (cy + 171.0) as u32;
        let pie_v_max = (cy + 188.0).min((alto - 1) as f32) as u32;
        let color_zapatos = [20, 20, 25, 255];

        for v in 0..alto {
            for u in 0..ancho {
                let idx = (v * ancho + u) as usize;
                let u_f = u as f32;
                let v_f = v as f32;

                // Cabeza
                let dist_cabeza_cuad = (u_f - centro_cabeza_u).powi(2) + (v_f - centro_cabeza_v).powi(2);
                if dist_cabeza_cuad <= radio_cabeza_cuad {
                    let relieve = (1.0 - (dist_cabeza_cuad / radio_cabeza_cuad)).sqrt() * 50.0;
                    frame.profundidad[idx] = 2000.0 - relieve;
                    frame.color_registrado[idx] = color_piel;
                    continue;
                }

                // Torso
                if u >= torso_u_min && u <= torso_u_max && v >= torso_v_min && v <= torso_v_max {
                    frame.profundidad[idx] = 2000.0;
                    frame.color_registrado[idx] = color_torso;
                    continue;
                }

                // Brazos
                if (u >= brazo_izq_u_min && u <= brazo_izq_u_max
                    || u >= brazo_der_u_min && u <= brazo_der_u_max)
                    && v >= brazo_v_min
                    && v <= brazo_v_max
                {
                    frame.profundidad[idx] = 2050.0;
                    frame.color_registrado[idx] = color_brazo;
                    continue;
                }

                // Pierna izquierda
                if u >= pierna_izq_u_min
                    && u <= pierna_izq_u_max
                    && v >= pierna_v_min
                    && v <= pierna_v_max
                {
                    frame.profundidad[idx] = pierna_izq_prof;
                    frame.color_registrado[idx] = color_pantalones;
                    continue;
                }

                // Pierna derecha
                if u >= pierna_der_u_min
                    && u <= pierna_der_u_max
                    && v >= pierna_v_min
                    && v <= pierna_v_max
                {
                    frame.profundidad[idx] = pierna_der_prof;
                    frame.color_registrado[idx] = color_pantalones;
                    continue;
                }

                // Pies
                if (u >= pierna_izq_u_min && u <= pierna_izq_u_max + 4
                    || u >= pierna_der_u_min - 4 && u <= pierna_der_u_max)
                    && v >= pie_v_min
                    && v <= pie_v_max
                {
                    frame.profundidad[idx] = 1950.0;
                    frame.color_registrado[idx] = color_zapatos;
                }
            }
        }

        frame
    }

    /// Genera una sesión de prueba en memoria y retorna los bytes codificados en formato `.ferrite`.
    pub fn crear_grabacion_en_memoria(
        &self,
        cantidad_fotogramas: usize,
    ) -> Result<Vec<u8>, ErrorSensor> {
        let cursor = Cursor::new(Vec::new());
        let mut grabador = GrabadorFerrite::nuevo(cursor, self.intrinsecos, self.fps)?;

        let periodo_us = (1_000_000.0 / self.fps) as u64;

        for i in 0..cantidad_fotogramas {
            let marca_us = (i as u64) * periodo_us;
            let frame = self.generar_fotograma(marca_us);
            grabador.escribir_frame(&frame)?;
        }

        grabador.vaciar()?;
        Ok(grabador.desempaquetar().into_inner())
    }
}

impl FuenteFrames for GeneradorSinteticoSensor {
    fn siguiente_frame(&mut self, _espera: Duration) -> Result<Option<FrameSensor>, ErrorSensor> {
        let periodo_us = (1_000_000.0 / self.fps) as u64;
        let marca_us = self.fotograma_actual * periodo_us;
        self.fotograma_actual += 1;
        Ok(Some(self.generar_fotograma(marca_us)))
    }

    #[inline]
    fn intrinsecos(&self) -> IntrinsecosProfundidad {
        self.intrinsecos
    }
}
