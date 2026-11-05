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

**Resultado de verificación:** `./scripts/verificar.sh` pasa al 100% (20 tests en verde, 0 advertencias en rustdoc, licencias validadas con `cargo deny`, 0 fallos ortográficos).

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
| **AÚN NO HA LLEGADO** (Desarrollo sin hardware) | **WP-3.2** (`ferrite-sensor`: trait + reproducción) | Crear el crate `crates/ferrite-sensor` con el trait `FuenteFrames`, la estructura `FrameSensor`, el formato `.ferrite` comprimido con zstd, `FuenteReproduccion` y un generador sintético de frames para tests. |
| **AÚN NO HA LLEGADO** | **F5** (Solver cinemático 3D) | Se puede avanzar con `crates/ferrite-solver`: elevación 3D (WP-5.1), filtro One Euro (WP-5.2), cálculo de bases ortonormales para rotaciones de cadera y pies (WP-5.3), y mitigación de oclusiones/giro de espaldas (WP-5.4). |
| **YA LLEGÓ EL KINECT** | **WP-3.1** (`ferrite-freenect2-sys`) | Compilar `libfreenect2` en el sistema (WP-1.1), crear el puente seguro `cxx` en `crates/ferrite-freenect2-sys` (detrás de la feature `kinect`). |
| **YA LLEGÓ EL KINECT** | **WP-3.3** (`FuenteKinect` + grabar/reproducir) | Implementar `FuenteKinect` sobre el puente `cxx` y los subcomandos `ferrite grabar` y `ferrite reproducir`. |

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
