//! Cálculo de rotaciones anatómicas y bases ortonormales para cuaterniones de Unity.

use crate::elevacion::EsqueletoElevado;
use glam::{Mat3, Quat, Vec3};

/// Construye una rotación [`Quat`] a partir de vectores de dirección aproximados para el eje lateral y el eje superior.
///
/// Utiliza ortonormalización tipo Gram-Schmidt. Si los vectores son colineales o nulos,
/// retorna `Quat::IDENTITY` de forma determinista para evitar la propagación de valores `NaN`.
pub fn construir_rotacion_de_ejes(eje_x_aprox: Vec3, eje_y_aprox: Vec3) -> Quat {
    let len_x_cuad = eje_x_aprox.length_squared();
    let len_y_cuad = eje_y_aprox.length_squared();

    if len_x_cuad < 1e-6 || len_y_cuad < 1e-6 {
        return Quat::IDENTITY;
    }

    let y = eje_y_aprox.normalize();
    // En Unity (+Y arriba, +X derecha), el eje frontal hacia adelante es Z = X x Y
    let z_crudo = eje_x_aprox.cross(y);
    let len_z_cuad = z_crudo.length_squared();

    if len_z_cuad < 1e-6 {
        return Quat::IDENTITY;
    }

    let z = z_crudo.normalize();
    let x = y.cross(z).normalize();

    let mat = Mat3::from_cols(x, y, z);
    Quat::from_mat3(&mat).normalize()
}

/// Calcula la rotación orientada de la cadera (pelvis) a partir del esqueleto elevado.
pub fn calcular_rotacion_cadera(esqueleto: &EsqueletoElevado) -> Quat {
    let h = &esqueleto.articulaciones_halpe;
    let cad_izq = h[11];
    let cad_der = h[12];
    let pelvis = h[19];
    let cuello = h[18];

    if cad_izq.confianza < 0.1 || cad_der.confianza < 0.1 {
        return Quat::IDENTITY;
    }

    // Vector horizontal de cadera izquierda hacia cadera derecha
    let eje_x = cad_der.posicion.0 - cad_izq.posicion.0;

    // Vector vertical de pelvis hacia cuello
    let eje_y = if cuello.confianza > 0.1 && pelvis.confianza > 0.1 {
        cuello.posicion.0 - pelvis.posicion.0
    } else {
        Vec3::Y
    };

    construir_rotacion_de_ejes(eje_x, eje_y)
}

/// Calcula la rotación orientada del pie izquierdo a partir del talón, tobillo y dedos.
pub fn calcular_rotacion_pie_izq(esqueleto: &EsqueletoElevado) -> Quat {
    let h = &esqueleto.articulaciones_halpe;
    let rodilla = h[13];
    let tobillo = h[15];
    let dedo_gordo = h[20];
    let talon = h[22];

    let eje_y = if rodilla.confianza > 0.1 && tobillo.confianza > 0.1 {
        rodilla.posicion.0 - tobillo.posicion.0
    } else {
        Vec3::Y
    };

    let eje_z = if dedo_gordo.confianza > 0.1 && talon.confianza > 0.1 {
        dedo_gordo.posicion.0 - talon.posicion.0
    } else {
        Vec3::Z
    };

    // Para el pie: Z es adelante, Y es arriba -> X = Y x Z
    let eje_x = eje_y.cross(eje_z);
    construir_rotacion_de_ejes(eje_x, eje_y)
}

/// Calcula la rotación orientada del pie derecho a partir del talón, tobillo y dedos.
pub fn calcular_rotacion_pie_der(esqueleto: &EsqueletoElevado) -> Quat {
    let h = &esqueleto.articulaciones_halpe;
    let rodilla = h[14];
    let tobillo = h[16];
    let dedo_gordo = h[23];
    let talon = h[25];

    let eje_y = if rodilla.confianza > 0.1 && tobillo.confianza > 0.1 {
        rodilla.posicion.0 - tobillo.posicion.0
    } else {
        Vec3::Y
    };

    let eje_z = if dedo_gordo.confianza > 0.1 && talon.confianza > 0.1 {
        dedo_gordo.posicion.0 - talon.posicion.0
    } else {
        Vec3::Z
    };

    let eje_x = eje_y.cross(eje_z);
    construir_rotacion_de_ejes(eje_x, eje_y)
}
