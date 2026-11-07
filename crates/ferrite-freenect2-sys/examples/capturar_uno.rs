//! Ejemplo de captura de un fotograma real de color y profundidad desde Kinect v2.
//!
//! Requiere tener conectado el sensor Kinect v2 y compilar con `--features kinect`.
//! Guarda `captura_color.png` y `captura_profundidad.png` en el directorio actual.

#[cfg(feature = "kinect")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ferrite_freenect2_sys::{
        configurar_registro, DispositivoKinect, ModoPipeline, NivelRegistro,
    };
    use std::time::Instant;

    println!("=== Captura de fotograma Kinect v2 (FerriteVR) ===");

    configurar_registro(NivelRegistro::Advertencia);

    println!("Abriendo conexion con el sensor Kinect v2...");
    let mut kinect = DispositivoKinect::abrir(ModoPipeline::PorDefecto)?;

    println!("Iniciando flujos de datos...");
    kinect.iniciar()?;

    let serie = kinect.numero_serie().unwrap_or_else(|_| "desconocido".to_string());
    let firmware = kinect.version_firmware().unwrap_or_else(|_| "desconocido".to_string());
    let intr = kinect.intrinsecos()?;

    println!("Sensor conectado e iniciado:");
    println!("  - Numero de serie:    {serie}");
    println!("  - Version de firmware:{firmware}");
    println!(
        "  - Intrinsecos IR:     fx={:.2}, fy={:.2}, cx={:.2}, cy={:.2}",
        intr.fx, intr.fy, intr.cx, intr.cy
    );

    let ancho = 512;
    let alto = 424;
    let total_pixeles = ancho * alto;
    let mut buffer_profundidad = vec![0.0f32; total_pixeles];
    let mut buffer_color = vec![0u8; total_pixeles * 4];

    println!("Esperando estabilizacion y primer fotograma valido...");
    let inicio = Instant::now();
    let mut capturado = false;

    // Descartar primeros fotogramas si la exposicion o el sensor estan en calentamiento
    for intento in 1..=30 {
        if kinect.esperar_y_registrar(1000, &mut buffer_profundidad, &mut buffer_color)? {
            // Verificar si hay al menos algunos pixeles de profundidad validos (> 0.0)
            let pixeles_validos = buffer_profundidad.iter().filter(|&&d| d > 0.0).count();
            if pixeles_validos > 1000 {
                println!(
                    "Fotograma capturado con exito en intento {intento} (pixeles validos: {pixeles_validos})"
                );
                capturado = true;
                break;
            }
        }
    }

    let duracion = inicio.elapsed();
    println!("Tiempo transcurrido: {:.2?}", duracion);

    println!("Deteniendo sensor...");
    kinect.detener()?;

    if !capturado {
        eprintln!("Advertencia: no se obtuvo un fotograma con profundidad valida a tiempo");
        return Ok(());
    }

    // Guardar imagen de color (512x424 RGBA)
    let ruta_color = "captura_color.png";
    image::save_buffer(
        ruta_color,
        &buffer_color,
        ancho as u32,
        alto as u32,
        image::ExtendedColorType::Rgba8,
    )?;
    println!("Guardada imagen de color registrada en: {ruta_color}");

    // Convertir profundidad a escala de grises visual (Luma8)
    // Mapear rango util (500 mm a 4500 mm) a [0, 255]
    let mut visual_profundidad = vec![0u8; total_pixeles];
    for (i, &dist_mm) in buffer_profundidad.iter().enumerate() {
        if dist_mm >= 500.0 && dist_mm <= 4500.0 {
            // Mas cercano = mas brillante
            let norm = 1.0 - ((dist_mm - 500.0) / 4000.0);
            visual_profundidad[i] = (norm.clamp(0.0, 1.0) * 255.0) as u8;
        } else {
            visual_profundidad[i] = 0;
        }
    }

    let ruta_profundidad = "captura_profundidad.png";
    image::save_buffer(
        ruta_profundidad,
        &visual_profundidad,
        ancho as u32,
        alto as u32,
        image::ExtendedColorType::L8,
    )?;
    println!("Guardada imagen de mapa de profundidad en: {ruta_profundidad}");

    println!("=== Captura completada exitosamente ===");
    Ok(())
}

#[cfg(not(feature = "kinect"))]
fn main() {
    println!("Este ejemplo requiere la caracteristica 'kinect'. Ejecutar con: cargo run --example capturar_uno --features kinect");
}
