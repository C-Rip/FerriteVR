//! Filtro adaptativo temporal One Euro para eliminar temblores (jitter) sin latencia perceptible.
//!
//! Basado en el algoritmo clásico de Casiez, Roussel y Vogel (CHI 2012):
//! *"1€ Filter: A Simple Speed-based Low-pass Filter for Computing with Noisy Input in Human-Computer Interfaces"*.

use glam::{Quat, Vec3};
use std::f32::consts::PI;

/// Calcula el coeficiente alfa de suavizado para una frecuencia de corte y periodo temporal dados.
#[inline]
fn calcular_alfa(dt_seg: f32, frecuencia_corte: f32) -> f32 {
    let tau = 1.0 / (2.0 * PI * frecuencia_corte);
    1.0 / (1.0 + tau / dt_seg)
}

/// Filtro One Euro para valores escalares de precisión simple `f32`.
#[derive(Debug, Clone)]
pub struct FiltroOneEuro {
    corte_min: f32,
    beta: f32,
    corte_derivada: f32,
    x_previo: Option<f32>,
    dx_previo: Option<f32>,
    marca_tiempo_us_previa: Option<u64>,
}

impl Default for FiltroOneEuro {
    /// Valores predeterminados óptimos para seguimiento corporal humano en RV:
    /// - `corte_min = 1.0` Hz (alta estabilidad y >95% reducción de ruido cuando el cuerpo está quieto).
    /// - `beta = 0.05` (rápida adaptación y latencia < 1 frame durante movimientos rápidos).
    /// - `corte_derivada = 1.0` Hz.
    fn default() -> Self {
        Self::nuevo(1.0, 0.05, 1.0)
    }
}

impl FiltroOneEuro {
    /// Crea un nuevo filtro One Euro con los parámetros especificados.
    pub fn nuevo(corte_min: f32, beta: f32, corte_derivada: f32) -> Self {
        Self {
            corte_min: corte_min.max(0.001),
            beta: beta.max(0.0),
            corte_derivada: corte_derivada.max(0.001),
            x_previo: None,
            dx_previo: None,
            marca_tiempo_us_previa: None,
        }
    }

    /// Filtra un nuevo valor escalar en el instante `marca_tiempo_us`.
    pub fn filtrar(&mut self, x: f32, marca_tiempo_us: u64) -> f32 {
        let Some(t_previo) = self.marca_tiempo_us_previa else {
            self.x_previo = Some(x);
            self.dx_previo = Some(0.0);
            self.marca_tiempo_us_previa = Some(marca_tiempo_us);
            return x;
        };

        let dt_us = marca_tiempo_us.saturating_sub(t_previo);
        if dt_us == 0 {
            return self.x_previo.unwrap_or(x);
        }

        let dt_seg = (dt_us as f32) / 1_000_000.0;
        let x_prev = self.x_previo.unwrap_or(x);
        let dx_prev = self.dx_previo.unwrap_or(0.0);

        // 1. Estimar velocidad derivada filtrada
        let dx_cruda = (x - x_prev) / dt_seg;
        let alfa_d = calcular_alfa(dt_seg, self.corte_derivada);
        let dx_filtrada = alfa_d * dx_cruda + (1.0 - alfa_d) * dx_prev;

        // 2. Frecuencia de corte adaptativa basada en la velocidad
        let corte_adaptativo = self.corte_min + self.beta * dx_filtrada.abs();

        // 3. Suavizar valor principal
        let alfa = calcular_alfa(dt_seg, corte_adaptativo);
        let x_filtrado = alfa * x + (1.0 - alfa) * x_prev;

        self.x_previo = Some(x_filtrado);
        self.dx_previo = Some(dx_filtrada);
        self.marca_tiempo_us_previa = Some(marca_tiempo_us);

        x_filtrado
    }

    /// Reinicia el estado interno del filtro.
    pub fn reiniciar(&mut self) {
        self.x_previo = None;
        self.dx_previo = None;
        self.marca_tiempo_us_previa = None;
    }
}

/// Filtro One Euro tridimensional para posiciones vectoriales [`Vec3`].
#[derive(Debug, Clone, Default)]
pub struct FiltroOneEuroVec3 {
    filtro_x: FiltroOneEuro,
    filtro_y: FiltroOneEuro,
    filtro_z: FiltroOneEuro,
}

impl FiltroOneEuroVec3 {
    /// Crea un nuevo filtro vectorial con los coeficientes indicados.
    pub fn nuevo(corte_min: f32, beta: f32) -> Self {
        Self {
            filtro_x: FiltroOneEuro::nuevo(corte_min, beta, 1.0),
            filtro_y: FiltroOneEuro::nuevo(corte_min, beta, 1.0),
            filtro_z: FiltroOneEuro::nuevo(corte_min, beta, 1.0),
        }
    }

    /// Filtra un vector tridimensional de posición en el instante de tiempo dado.
    pub fn filtrar(&mut self, pos: Vec3, marca_tiempo_us: u64) -> Vec3 {
        Vec3::new(
            self.filtro_x.filtrar(pos.x, marca_tiempo_us),
            self.filtro_y.filtrar(pos.y, marca_tiempo_us),
            self.filtro_z.filtrar(pos.z, marca_tiempo_us),
        )
    }

    /// Reinicia el estado de los tres ejes.
    pub fn reiniciar(&mut self) {
        self.filtro_x.reiniciar();
        self.filtro_y.reiniciar();
        self.filtro_z.reiniciar();
    }
}

/// Filtro temporal adaptativo para rotaciones representadas mediante cuaterniones [`Quat`].
#[derive(Debug, Clone)]
pub struct FiltroRotacion {
    corte_min: f32,
    beta: f32,
    q_previo: Option<Quat>,
    marca_tiempo_us_previa: Option<u64>,
}

impl Default for FiltroRotacion {
    fn default() -> Self {
        Self::nuevo(1.5, 0.01)
    }
}

impl FiltroRotacion {
    /// Crea un nuevo filtro adaptativo de rotación.
    pub fn nuevo(corte_min: f32, beta: f32) -> Self {
        Self {
            corte_min: corte_min.max(0.001),
            beta: beta.max(0.0),
            q_previo: None,
            marca_tiempo_us_previa: None,
        }
    }

    /// Filtra suavemente una rotación mediante interpolación esférica (SLERP) adaptativa.
    pub fn filtrar(&mut self, q_nuevo: Quat, marca_tiempo_us: u64) -> Quat {
        let q_normalizado = q_nuevo.normalize();

        let Some(t_previo) = self.marca_tiempo_us_previa else {
            self.q_previo = Some(q_normalizado);
            self.marca_tiempo_us_previa = Some(marca_tiempo_us);
            return q_normalizado;
        };

        let dt_us = marca_tiempo_us.saturating_sub(t_previo);
        if dt_us == 0 {
            return self.q_previo.unwrap_or(q_normalizado);
        }

        let dt_seg = (dt_us as f32) / 1_000_000.0;
        let q_prev = self.q_previo.unwrap_or(q_normalizado);

        // Asegurar que seguimos el camino más corto en la esfera 4D (q y -q representan la misma rotación)
        let dot = q_prev.dot(q_normalizado);
        let q_ajustado = if dot < 0.0 {
            -q_normalizado
        } else {
            q_normalizado
        };

        // Distancia angular en radianes
        let angulo = 2.0 * (dot.abs().min(1.0)).acos();
        let velocidad_angular = angulo / dt_seg;

        let corte_adaptativo = self.corte_min + self.beta * velocidad_angular;
        let alfa = calcular_alfa(dt_seg, corte_adaptativo).clamp(0.0, 1.0);

        let q_filtrado = q_prev.slerp(q_ajustado, alfa).normalize();

        self.q_previo = Some(q_filtrado);
        self.marca_tiempo_us_previa = Some(marca_tiempo_us);

        q_filtrado
    }

    /// Reinicia el historial de rotación.
    pub fn reiniciar(&mut self) {
        self.q_previo = None;
        self.marca_tiempo_us_previa = None;
    }
}
