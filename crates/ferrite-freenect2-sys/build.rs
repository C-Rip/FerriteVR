//! Script de compilación para `ferrite-freenect2-sys`.
//!
//! Enlaza con `libfreenect2` y compila el puente C++ mediante `cxx_build`
//! cuando la característica `kinect` está activada.

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=include/shim.h");
    println!("cargo:rerun-if-changed=src/shim.cc");
    println!("cargo:rerun-if-changed=src/lib.rs");

    if env::var("CARGO_FEATURE_KINECT").is_err() {
        return;
    }

    // Asegurar que pkg-config busque en /usr/local/lib/pkgconfig
    let ruta_pkg = env::var("PKG_CONFIG_PATH").unwrap_or_default();
    let rutas_adicionales = "/usr/local/lib/pkgconfig:/usr/local/lib64/pkgconfig";
    let ruta_unificada = if ruta_pkg.is_empty() {
        rutas_adicionales.to_string()
    } else {
        format!("{ruta_pkg}:{rutas_adicionales}")
    };
    // SEGURIDAD: Configuracion de variable de entorno local para resolucion de rutas pkg-config en build.rs
    unsafe {
        env::set_var("PKG_CONFIG_PATH", ruta_unificada);
    }

    let mut builder = cxx_build::bridge("src/lib.rs");
    builder.file("src/shim.cc");
    builder.include("include");
    builder.std("c++14");

    // Intentar resolver rutas via pkg-config
    let mut incluye_usr_local = false;
    if let Ok(biblioteca) = pkg_config::Config::new()
        .atleast_version("0.2.0")
        .probe("freenect2")
    {
        for dir in biblioteca.include_paths {
            builder.include(dir);
        }
        for dir in &biblioteca.link_paths {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
        }
    } else {
        // Fallback directo a rutas estandar de instalacion local
        builder.include("/usr/local/include");
        println!("cargo:rustc-link-search=native=/usr/local/lib");
        println!("cargo:rustc-link-lib=freenect2");
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/local/lib");
        incluye_usr_local = true;
    }

    if !incluye_usr_local {
        // Asegurar que /usr/local/include este disponible como respaldo para headers
        let usr_local_inc = PathBuf::from("/usr/local/include");
        if usr_local_inc.exists() {
            builder.include(usr_local_inc);
        }
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/local/lib");
    }

    builder.compile("ferrite-freenect2-sys");
}
