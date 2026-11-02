# Reglas del Agente - FerriteVR

FerriteVR es un programa nativo de Linux en Rust que convierte un sensor Kinect v2 (Xbox One) en seguimiento corporal completo (*full body tracking*) para VRChat mediante el protocolo OSC.
El sistema procesa en tiempo real color y profundidad con pose 2D (RTMPose) y elevacion 3D, buscando latencia minima (< 25 ms) para Meta Quest 3S via Steam Link sobre Nobara 44 (KDE Wayland).
Este documento fija las directrices operativas obligatorias para cualquier agente o colaborador.

## 1. Mapa de crates del workspace

| Crate | Ubicacion | Responsabilidad |
|---|---|---|
| `ferrite-nucleo` | `crates/ferrite-nucleo/` | Tipos comunes, espacios tipados (`PosKinect`, `PosUnity`), config y utilidades. |
| `ferrite-osc` | `crates/ferrite-osc/` | Emisor OSC hacia VRChat por UDP (bundle atomico `/tracking/trackers/*`). |
| `ferrite-sensor` | `crates/ferrite-sensor/` | Trait `FuenteFrames`, reproduccion `.ferrite` y soporte Kinect v2 (feature). |
| `ferrite-freenect2-sys` | `crates/ferrite-freenect2-sys/` | FFI con `libfreenect2` via `cxx`. **Unico crate con `unsafe`**. |
| `ferrite-pose` | `crates/ferrite-pose/` | Inferencia ONNX (`ort`) de pose 2D (RTMPose Halpe26). |
| `ferrite-solver` | `crates/ferrite-solver/` | Desproyeccion 3D, filtros One Euro, oclusion, rotaciones y calibracion. |
| `ferrite` (CLI) | `apps/ferrite/` | Binario CLI con subcomandos (`fake`, `grabar`, `reproducir`, `run`, `benchmark`). |

## 2. Comandos canonicos

- **Verificacion integral:** `./scripts/verificar.sh` (fmt, clippy, test, doc, deny, typos).
- **Ejecucion simulada (sin Kinect):** `cargo run -p ferrite -- fake`

## 3. Convenciones de codigo

1. **Identificadores en espanol (ASCII estricto):**
   - Variables, funciones, structs, enums y modulos en espanol ASCII (sin tildes ni `ñ`, e.g., `pos_cadera`, `Articulacion`, `Esqueleto3D`). Se aplica con `#![deny(non_ascii_idents)]`. Comentarios y rustdoc usan espanol estandar.
2. **Errores:**
   - `thiserror` en librerias (`crates/*`) y `anyhow::Result` exclusivamente en binarios (`apps/*`).
   - Prohibido `unwrap()` o `expect()` fuera de tests.
3. **Codigo inseguro (`unsafe`):**
   - Permitido **unicamente** en `crates/ferrite-freenect2-sys/`.
   - Requiere obligatoriamente comentario: `// SEGURIDAD: <justificacion del invariante>`.
4. **Tiempo real:**
   - Cero asignaciones de memoria por frame en el camino caliente (reutilizacion de buffers).
   - Sin E/S bloqueante ni contencion de cerraduras (`Mutex`) en hilos del pipeline.
   - Canales "ultimo gana" (capacidad 1) entre hilos: descartan frames viejos ante atrasos.
5. **Espacios de coordenadas tipados:**
   - Newtypes explicitos `PosKinect` (mano derecha) y `PosUnity` (mano izquierda, +Y arriba).
   - Prohibido `Vec3` crudo en interfaces publicas que crucen espacios geometricos.

## 4. Definition of Done (DoD)

1. `./scripts/verificar.sh` termina con codigo 0 sin advertencias.
2. Tests automatizados para toda logica nueva o modificada.
3. Rustdoc completo en espanol en cada elemento publico.
4. Actualizacion de `CHANGELOG.md` bajo `## [Sin publicar]`.
5. Nuevo ADR en `docs/adr/` ante cualquier decision de arquitectura relevante.

## 5. Reglas de Git

- **Rama de trabajo:** Exclusivamente sobre `dev`.
- **Prohibiciones:**
  - NUNCA hacer commit ni merge directo a `main`.
  - NUNCA usar `git push --force` ni reescribir historial.
  - No hacer `git push` a remotos salvo orden explicita del usuario.
- **Formato:** Conventional Commits en espanol (imperativo) con tipo en ingles (`feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `ci`). Ejemplo: `feat(osc): agrega emision de trackers`.

## 6. Prohibicion estricta de `sudo`

Los agentes y scripts **NUNCA DEBEN EJECUTAR `sudo`**. Toda accion que requiera privilegios de superusuario debe reportarse al usuario para su ejecucion manual.

## 7. Tabla de skills en `.agents/skills/`

| Skill | Directorio | Proposito y momento de uso |
|---|---|---|
| `rust-guidelines` | `.agents/skills/rust-guidelines/` | Directrices pragmaticas de Rust adaptadas a FerriteVR (FFI, tiempo real, robustez). |
| `ferrite-verificar` | `.agents/skills/ferrite-verificar/` | Ejecucion e interpretacion de `./scripts/verificar.sh` y resolucion de fallos. |
| `ferrite-docs` | `.agents/skills/ferrite-docs/` | Estructura Diataxis, entradas en `CHANGELOG.md` y redaccion de ADRs en `docs/adr/`. |
| `vrchat-osc` | `.agents/skills/vrchat-osc/` | Protocolo OSC de VRChat (`/tracking/trackers/*`, unidades, ejes y slots). |
| `kinect-depuracion-hw` | `.agents/skills/kinect-depuracion-hw/` | Diagnostico de hardware Kinect v2, enlace USB 3 y libfreenect2 (Fase 3). |
