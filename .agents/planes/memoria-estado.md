# Memoria de estado y traspaso para agentes de IA (FerriteVR)

> **Destinatario:** Cualquier modelo de IA o agente asistente (Claude, Gemini, Grok, GLM, DeepSeek, etc.) y desarrollador humano que retome este proyecto en el futuro.
> **Fecha de corte de esta sesión:** 2026-11-01
> **Rama activa:** `dev` (¡NUNCA hacer commit directo a `main`!)
> **Último commit de funcionalidad:** `12211a2 feat(cli): agrega modo fake con esqueleto sintetico`

---

## 1. Contexto esencial del proyecto

**FerriteVR** es un sistema nativo para Linux escrito en **Rust** que convierte un sensor **Kinect v2 (Xbox One)** en seguimiento de cuerpo completo (*Full Body Tracking* o FBT) para **VRChat**, emitiendo paquetes de trackers vía **OSC** sobre UDP al puerto local `9000`.

- **Hardware del usuario:** Intel Core Ultra 5 225F, NVIDIA GeForce RTX 3060 12 GB, 16 GB RAM, red Ethernet Gigabit (1000 Mbps).
- **Visor VR:** Meta Quest 3S conectado vía **Steam Link** inalámbrico (5 GHz) al PC ejecutando SteamVR en **Nobara Linux 44** (KDE Wayland).
- **Sensor Kinect v2:** El usuario ya cuenta con el adaptador de corriente USB 3.0 para PC; el sensor físico llega en ~1 semana.
- **Licencia:** Doble licencia permisiva `MIT OR Apache-2.0`.
- **Idioma:** **Todo en español**: documentación, rustdoc, comentarios en código, mensajes de commit e **identificadores en código (ASCII sin tildes ni `ñ`)**.

---

## 2. Estado actual de implementación (¿Qué está 100% hecho y verificado?)

A la fecha, se completaron con éxito las siguientes fases:

### F0: Gobernanza y Calidad
1. **Reglas del agente:** `AGENTS.md` en la raíz y `crates/ferrite-osc/AGENTS.md`.
2. **Skills modulares:** En `.agents/skills/`:
   - `rust-guidelines`: Directrices pragmáticas de Rust de Microsoft (`pragmatic-rust-guidelines.txt` vendorizado).
   - `ferrite-verificar`: Manual del script de validación.
   - `ferrite-docs`: Matriz Diátaxis, Keep a Changelog y MADR.
   - `vrchat-osc`: Especificación técnica del protocolo OSC para trackers.
3. **Documentación estructurada (`docs/`):**
   - 9 Registros de Decisiones de Arquitectura (ADRs `0000` a `0008` en `docs/adr/`).
   - `docs/explicacion/herramientas-agente.md`
   - `docs/referencia/vrchat-osc-trackers.md`
   - `docs/guias/instalar-cuda-tensorrt.md`
   - `docs/tutoriales/primera-prueba-con-vrchat.md`
4. **Tooling de calidad:**
   - `rustfmt.toml`, `clippy.toml`, `deny.toml` (licencias y seguridad), `_typos.toml` (ortografía en español).
   - `scripts/verificar.sh` (script canónico que ejecuta fmt, clippy, tests, docs, deny y typos).
   - `.githooks/pre-commit` enlazado con `git config core.hooksPath .githooks`.
   - `.github/workflows/ci.yml` para GitHub Actions.

### F1 (Investigación): Aceleración CUDA / TensorRT
- Guía completa generada en `docs/guias/instalar-cuda-tensorrt.md` explicando cómo instalar bibliotecas modulares de CUDA 13.2 y cuDNN 9.20 en Nobara 44 sin romper el driver 595.

### F2: Workspace Cargo, Núcleo, OSC y CLI
- **`Cargo.toml` raíz:** Workspace configurado con resolver 3, perfiles de release optimizados y lints estrictos (`unsafe_code = "deny"`, `missing_docs = "warn"`, `non_ascii_idents = "deny"`, `unwrap_used = "warn"`).
- **`crates/ferrite-nucleo`:**
  - Espacios de coordenadas tipados con newtypes: `PosKinect` (mano derecha) y `PosUnity` (mano izquierda, $Y$ arriba, metros).
  - Tipos `Articulacion`, `PoseArticulacion`, `Esqueleto3D`.
  - Conversión bidireccional pura entre cuaterniones (`glam::Quat`) y ángulos de Euler en convención ZXY en grados sexagesimales de Unity/VRChat.
  - 11 pruebas (7 unitarias incluyendo `proptest` de ida y vuelta + 4 doc-tests).
- **`crates/ferrite-osc`:**
  - Tipos `RanuraTracker` (validada para `1..=8` y `Cabeza`), `MuestraTracker` y `EmisorVrchat`.
  - Empaquetado atómico de cada fotograma en un único `OscBundle` de UDP.
  - 6 pruebas (4 unitarias incluyendo prueba de integración con socket UDP real y decodificación con `rosc::decoder` + 2 doc-tests).
- **`apps/ferrite` (CLI):**
  - Subcomando funcional `ferrite fake` (`--destino 127.0.0.1:9000`, `--hz 30`, `--duracion-seg N`).
  - Generador sintético de caminata determinista en el sitio (pies con $Y \ge 0.05$ m, coordenadas finitas).
  - Manejo de Ctrl+C limpio (`ctrlc`) y emisión sin deriva de reloj.
  - 3 pruebas unitarias probando el generador sintético.

### F3: Sensores y Grabación (Parte A sin hardware)
- **`crates/ferrite-sensor` (WP-3.2):**
  - Trait `FuenteFrames` unificando hardware físico y archivos de sesión.
  - Estructura `FrameSensor` con preasignación y método `asegurar_dimensiones` para garantizar cero asignaciones en caliente.
  - Formato de archivo binario `.ferrite` (encabezado versionado de 48 bytes + fotogramas comprimidos individualmente con `zstd`).
  - `GrabadorFerrite` y `LectorFerrite` optimizados para compresión por fotograma.
  - `FuenteReproduccion` respetando marcas de tiempo en microsegundos o a máxima velocidad (`sin_tiempo`).
  - `GeneradorSinteticoSensor` para pruebas sin hardware simulando una figura humana en profundidad oscilando a 1 Hz.
  - 7 pruebas (6 unitarias + 1 doc-test).
  - Especificación técnica completa en `docs/referencia/formato-ferrite.md`.

### F4: Estimación de Pose 2D (RTMPose Halpe26)
- **Modelos y recursos (WP-4.1):**
  - Script canónico `scripts/descargar-modelos.sh` con descarga automatizada del paquete oficial de OpenMMLab, extracción de `end2end.onnx` y validación estricta de hash SHA-256 (`26f3a19e61304a600dfb82d1001d41d24343b89fc70a33ffc84657e0b0bf2ecf`). Idempotente.
  - Especificación técnica completa en `docs/referencia/modelos.md` documentando entrada normalizada NCHW (256x192), representación SimCC (vectores 1D horizontales y verticales a medio píxel) y tabla de correspondencia de las 26 articulaciones hacia los trackers OSC de VRChat.
- **Inferencia y decodificación SimCC (WP-4.3 - `ferrite-pose`):**
  - Motor neuronal `MotorPose` integrando `ort` 2.0 (ONNX Runtime en Rust) optimizado en CPU multihilo con `intra_threads = 4`.
  - Módulo `simcc`: Decodificación SimCC mediante `argmax` en vectores 1D independientes (`simcc_x` 384 bins, `simcc_y` 512 bins) con división subpíxel (`split_ratio = 2.0`).
  - Extracción de confianza estadística acotada y reproyección inmediata al marco de referencia 2D de la cámara ($512 \times 424$).
  - Prueba de integración end-to-end con el modelo real ONNX y un fotograma generado por `GeneradorSinteticoSensor`, validando los 26 puntos clave de Halpe26.
  - 8 pruebas en total (6 unitarias + 2 doc-tests).

### F5: Solver Cinemático 3D (`ferrite-solver`)
- **Elevación 3D, Filtrado y Rotaciones (WP-5.1 a WP-5.3):**
  - Muestreo espacial por ventana mediana $5 \times 5$ (`Elevador3D`) que elimina ruido de bordes (*flying pixels*) y desproyecta píxeles 2D a posiciones métricas [`PosUnity`](../../crates/ferrite-nucleo/src/lib.rs).
  - Filtro adaptativo temporal `FiltroOneEuro` y `FiltroOneEuroVec3` ($f_{c,\min} = 1.0$ Hz, $\beta = 0.05$) con reducción de jitter en reposo $> 99\%$ y respuesta ultrarrápida ($< 1$ fotograma) ante movimientos bruscos.
  - Filtro adaptativo para rotaciones `FiltroRotacion` con interpolación esférica (SLERP) ponderada por velocidad angular.
  - Construcción de bases ortonormales anatómicas tipo Gram-Schmidt para cadera y pies sin división por cero ni `NaN`.
- **Mitigación de Oclusiones y Persistencia (WP-5.4):**
  - `GestorOclusion` con memoria de articulaciones y decaimiento exponencial temporal ($\tau = 250$ ms) para evitar pérdida abrupta de trackers.
  - Detección de orientación de espaldas (visibilidad facial vs. hombros) que preserva la coherencia anatómica en giros a 180°.
- **Calibración Espacial y Alineación de Suelo (WP-5.5):**
  - `CalibracionEspacio` compensando inclinación de cámara (*un-tilt* de pitch), elevando la coordenada Y respecto al piso físico ($H = 1.10$ m) y recentrando el eje Z en el origen de VRChat ($Z = 0$).
  - Garantía de cota mínima de suelo para pies ($Y \ge 0.05$ m).
  - Orquestador `SolverCinematico` integrando elevación, oclusiones, filtros y calibración para alimentar `Esqueleto3D`.
  - 8 pruebas unitarias exhaustivas validando filtrado, calibración, oclusiones y coherencia cinemática.

### F6: Integración del Pipeline en CLI (`apps/ferrite`)
- **Conexión completa de punta a punta (Sensor -> Pose 2D -> Solver 3D -> OSC VRChat):**
  - Orquestador `pipeline.rs` con ciclo continuo a 30 FPS sin deriva temporal, cero asignaciones en caliente y reporte de telemetría de latencia en tiempo real.
  - Subcomando `reproducir`: reproduce archivos grabados `.ferrite` (con soporte para `--bucle`) o fotogramas sintéticos hacia VRChat (`127.0.0.1:9000`).
  - Subcomando `grabar`: graba sesiones de prueba comprimidas con `zstd` en formato `.ferrite` (60 cuadros en ~320 KB).
  - Subcomando `bench`: evalúa latencias p50/p90/p99 sobre el pipeline completo. **Resultado en CPU en modo release: 14.63 ms p50 y 65.7 FPS globales**, superando con holgura el objetivo de latencia de < 25 ms.
  - Subcomando `run`: preparado para hardware y con opción `--sintetico` para ejecución directa emitiendo los **9 trackers completos** de VRChat.
  - Manejo unificado y seguro de señal Ctrl+C (`ctrlc`) para parada limpia.

**Resultado de verificación:** `./scripts/verificar.sh` pasa al 100% (39 tests unitarios, 11 doc-tests = 50 pruebas en verde, 0 advertencias en rustdoc, licencias validadas con `cargo deny`, 0 fallos ortográficos).

---

## 3. Próximos pasos inmediatos (Por dónde retomar)

El plan maestro detallado con todos los paquetes de trabajo está en `.agents/planes/plan-maestro.md`.

### A. Tareas manuales pendientes para el usuario humano
1. ~~**WP-2.4 Prueba en VRChat con Quest 3S + Steam Link:**~~ **COMPLETADA CON ÉXITO (2026-11-04)**
   - VRChat en Nobara 44 recibió correctamente la telemetría OSC atómica en `127.0.0.1:9000`.
   - Se validó el reconocimiento de esferas y la calibración FBT completa con mandos de Quest 3S.
   - El avatar reflejó el movimiento de marcha armónica sintética.
2. **WP-1.1 Instalación de dependencias del sistema (cuando llegue el sensor físico o se requiera compilar con soporte de hardware):**
   ```bash
   sudo dnf install rustfmt clippy cmake libusb1-devel turbojpeg-devel ocl-icd-devel mesa-libGL-devel glfw-devel clang-devel
   ```

### B. Próximos paquetes de desarrollo para el agente IA

| Si el Kinect físico... | Paquete a ejecutar | Descripción resumida |
|---|---|---|
| **AÚN NO HA LLEGADO** (Desarrollo sin hardware) | **WP-6.2** (Configuración persistente TOML) | Archivo de configuración en `~/.config/ferritevr/ferrite.toml` con opciones de calibración, destino OSC y filtros. |
| **AÚN NO HA LLEGADO** | **WP-6.3** (Visualizador opcional rerun) | Feature `viz` para inspeccionar nube de puntos y esqueleto 3D en tiempo real. |
| **YA LLEGÓ EL KINECT** | **WP-3.1** (`ferrite-freenect2-sys`) | Compilar `libfreenect2` en el sistema (WP-1.1), crear el puente seguro `cxx` en `crates/ferrite-freenect2-sys` (detrás de la feature `kinect`). |
| **YA LLEGÓ EL KINECT** | **WP-3.3** (`FuenteKinect` en vivo) | Conectar `FuenteKinect` sobre el puente `cxx` a `ferrite run` y `ferrite grabar`. |

---

## 4. Reglas innegociables para cualquier agente IA

1. **Rama de trabajo:** Trabajar siempre en `dev`. No tocar `main`. No usar `git push --force`.
2. **Sin comandos privilegiados:** NUNCA ejecutar `sudo`. Si algo requiere permisos elevados, documentar el comando exacto para que el usuario lo ejecute manualmente.
3. **Convenciones de código:**
   - Todos los identificadores en español ASCII (sin tildes ni `ñ`): `Articulacion`, `posicion`, `marca_tiempo_us`, etc.
   - Prohibido `unwrap()` y `expect()` fuera de bloques `#[cfg(test)]`.
   - `thiserror` para errores en librerías (`crates/*`), `anyhow` solo en aplicaciones (`apps/*`).
   - `unsafe` únicamente permitido en `ferrite-freenect2-sys` con comentario explicativo formal `// SEGURIDAD:`.
4. **Validación obligatoria:** Antes de dar cualquier tarea por completada, ejecutar `./scripts/verificar.sh` y asegurarse de que el código de salida sea 0.
5. **Registro de cambios:** Actualizar siempre `CHANGELOG.md` bajo `## [Sin publicar]` con cada cambio de funcionalidad.
6. **Commits:** Conventional Commits con tipo en inglés y descripción en imperativo en español:
   - Ejemplo: `feat(sensor): agrega trait FuenteFrames y reproduccion de grabaciones`
