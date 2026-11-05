//! Pipeline integral del solver cinemático 3D.

use std::collections::HashMap;
use ferrite_nucleo::{
    Articulacion, Esqueleto3D, IntrinsecosProfundidad, PosUnity, PoseArticulacion,
};
use ferrite_pose::simcc::Pose2D;
use ferrite_sensor::FrameSensor;
use glam::Quat;

use crate::calibracion::CalibracionEspacio;
use crate::elevacion::Elevador3D;
use crate::filtro::{FiltroOneEuroVec3, FiltroRotacion};
use crate::oclusion::GestorOclusion;
use crate::rotaciones::{
    calcular_rotacion_cadera, calcular_rotacion_pie_der, calcular_rotacion_pie_izq,
};
use crate::ErrorSolver;

/// Solver cinemático completo que transforma estimaciones 2D y profundidad en un esqueleto 3D filtrado y calibrado.
pub struct SolverCinematico {
    intrinsecos: IntrinsecosProfundidad,
    elevador: Elevador3D,
    calibracion: CalibracionEspacio,
    gestor_oclusion: GestorOclusion,
    filtros_posicion: HashMap<Articulacion, FiltroOneEuroVec3>,
    filtros_rotacion: HashMap<Articulacion, FiltroRotacion>,
}

impl SolverCinematico {
    /// Inicializa un nuevo solver con los parámetros intrínsecos de la cámara dados.
    pub fn nuevo(intrinsecos: IntrinsecosProfundidad) -> Self {
        let articulaciones = [
            Articulacion::Cabeza,
            Articulacion::Pecho,
            Articulacion::Cadera,
            Articulacion::RodillaIzq,
            Articulacion::RodillaDer,
            Articulacion::PieIzq,
            Articulacion::PieDer,
            Articulacion::CodoIzq,
            Articulacion::CodoDer,
        ];

        let mut filtros_posicion = HashMap::new();
        let mut filtros_rotacion = HashMap::new();

        for art in articulaciones {
            filtros_posicion.insert(art, FiltroOneEuroVec3::nuevo(1.0, 0.05));
            filtros_rotacion.insert(art, FiltroRotacion::nuevo(1.5, 0.05));
        }

        Self {
            intrinsecos,
            elevador: Elevador3D::default(),
            calibracion: CalibracionEspacio::default(),
            gestor_oclusion: GestorOclusion::default(),
            filtros_posicion,
            filtros_rotacion,
        }
    }

    /// Actualiza los parámetros intrínsecos de la cámara.
    pub fn con_intrinsecos(mut self, intrinsecos: IntrinsecosProfundidad) -> Self {
        self.intrinsecos = intrinsecos;
        self
    }

    /// Configura los parámetros de calibración espacial hacia VRChat.
    pub fn con_calibracion(mut self, calibracion: CalibracionEspacio) -> Self {
        self.calibracion = calibracion;
        self
    }

    /// Retorna una referencia a los parámetros de calibración actuales.
    #[inline]
    pub fn calibracion(&self) -> &CalibracionEspacio {
        &self.calibracion
    }

    /// Retorna una referencia mutable a los parámetros de calibración actuales.
    #[inline]
    pub fn calibracion_mut(&mut self) -> &mut CalibracionEspacio {
        &mut self.calibracion
    }

    /// Procesa una pose 2D junto con el mapa de profundidad para generar un [`Esqueleto3D`] suavizado y calibrado.
    pub fn resolver(
        &mut self,
        pose_2d: &Pose2D,
        frame: &FrameSensor,
    ) -> Result<Esqueleto3D, ErrorSolver> {
        let esqueleto_elevado = self.elevador.elevar(
            pose_2d,
            &frame.profundidad,
            frame.ancho,
            frame.alto,
            &self.intrinsecos,
        );

        let marca_tiempo = frame.marca_tiempo_us;
        let mut esqueleto_final = Esqueleto3D::nuevo(marca_tiempo);

        let articulaciones = [
            Articulacion::Cabeza,
            Articulacion::Pecho,
            Articulacion::Cadera,
            Articulacion::RodillaIzq,
            Articulacion::RodillaDer,
            Articulacion::PieIzq,
            Articulacion::PieDer,
            Articulacion::CodoIzq,
            Articulacion::CodoDer,
        ];

        // 1. Calcular orientaciones anatómicas
        let mut rot_cadera = calcular_rotacion_cadera(&esqueleto_elevado);
        let rot_pie_izq = calcular_rotacion_pie_izq(&esqueleto_elevado);
        let rot_pie_der = calcular_rotacion_pie_der(&esqueleto_elevado);

        // Si el sujeto está de espaldas, invertir yaw de cadera para mantener coherencia anatómica
        if GestorOclusion::detectar_espaldas(&esqueleto_elevado.articulaciones_halpe) {
            rot_cadera = rot_cadera * Quat::from_rotation_y(std::f32::consts::PI);
        }

        for art in articulaciones {
            let punto_crudo = self.elevador.extraer_articulacion(&esqueleto_elevado, art);

            let rot_cruda = match art {
                Articulacion::Cadera => rot_cadera,
                Articulacion::PieIzq => rot_pie_izq,
                Articulacion::PieDer => rot_pie_der,
                _ => Quat::IDENTITY,
            };

            // Filtrado One Euro para posición
            let pos_filtrada = if let Some(filtro_p) = self.filtros_posicion.get_mut(&art) {
                if punto_crudo.confianza > 0.05 {
                    filtro_p.filtrar(punto_crudo.posicion.0, marca_tiempo)
                } else {
                    punto_crudo.posicion.0
                }
            } else {
                punto_crudo.posicion.0
            };

            // Filtrado adaptativo para rotación
            let rot_filtrada = if let Some(filtro_r) = self.filtros_rotacion.get_mut(&art) {
                if punto_crudo.confianza > 0.05 {
                    filtro_r.filtrar(rot_cruda, marca_tiempo)
                } else {
                    rot_cruda
                }
            } else {
                rot_cruda
            };

            let pose_art = PoseArticulacion::nueva(
                PosUnity(pos_filtrada),
                rot_filtrada,
                punto_crudo.confianza,
            );
            esqueleto_final.insertar(art, pose_art);
        }

        // 2. Aplicar retención con decaimiento suave sobre articulaciones ocluidas
        self.gestor_oclusion
            .aplicar(&mut esqueleto_final, marca_tiempo);

        // 3. Transformar al espacio de juego de VRChat (elevación de suelo y recentrado)
        self.calibracion.aplicar_a_esqueleto(&mut esqueleto_final);

        Ok(esqueleto_final)
    }

    /// Reinicia los filtros temporales y la memoria de oclusiones del solver.
    pub fn reiniciar(&mut self) {
        for f in self.filtros_posicion.values_mut() {
            f.reiniciar();
        }
        for f in self.filtros_rotacion.values_mut() {
            f.reiniciar();
        }
        self.gestor_oclusion.reiniciar();
    }
}
