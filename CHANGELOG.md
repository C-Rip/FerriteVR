# Registro de cambios

Todos los cambios notables en este proyecto se documentarán en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/),
y este proyecto se adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [Sin publicar]

### Agregado
- Binario CLI `apps/ferrite` con parseo declarativo (`clap` derive), trazado estructurado (`tracing` y `tracing-subscriber`), manejo robusto de interrupción limpia con `ctrlc` y reporte contextual con `anyhow`.
- Subcomando `fake` con ticker de alta precisión sin deriva temporal (`--destino`, `--hz`, `--duracion-seg`) para emitir frames de marcha sintética fluida hacia VRChat OSC.
- Módulo cinemático determinista (`sintetico`) con garantía matemática de cota vertical en pies ($Y \ge 0.05$ m), balanceo natural de cadera y braceo coordinado.
- Declaración de subcomandos con stubs informativos (`run`, `grabar`, `reproducir`, `bench`) para las fases posteriores del proyecto.
- Tutorial guiado paso a paso en `docs/tutoriales/primera-prueba-con-vrchat.md` para calibración y validación de FBT en Meta Quest 3S mediante Steam Link.
- Crate `ferrite-osc` para emisión atómica de trackers hacia VRChat por UDP mediante paquetes bundle de OSC (`rosc`), con mapeo determinista de ranuras (`1..=8` y `head`), validación estricta de límites sin pánico y filtrado de articulaciones por confianza.
- Documento de directrices y convenciones OSC de VRChat en `crates/ferrite-osc/AGENTS.md`.
- Pruebas de integración UDP locales verificando codificación y decodificación de `OscBundle`, direcciones (`/tracking/trackers/*`) y tipos flotantes.
- Crate `ferrite-nucleo` con tipos de coordenadas tipados (`PosKinect` diestro, `PosUnity` siniestro), articulaciones (`Articulacion`), poses (`PoseArticulacion`), esqueleto temporal (`Esqueleto3D`) y funciones puras de conversión angular ZXY con normalización en `[-180.0, 180.0]`.
- Suite exhaustiva de pruebas unitarias, doc-tests y pruebas basadas en propiedades con `proptest` para el núcleo matemático.
- Workspace de Cargo inicializado en raíz con edición 2024, resolver v3, perfil release optimizado (LTO thin, 1 codegen-unit), dependencias compartidas y directrices estrictas de linting (`unsafe_code = "deny"`, `non_ascii_idents = "deny"`, `missing_docs = "warn"`).
- Guía técnica de instalación de CUDA 13, cuDNN 9 y TensorRT en Nobara 44 (`docs/guias/instalar-cuda-tensorrt.md`) para aceleración de inferencia con ort en WP-4.4.
- Configuración de herramientas de formateo y calidad: `rustfmt.toml` (Rust 2024, max 100 col), `clippy.toml`, `deny.toml` (auditoría de licencias y seguridad) y `_typos.toml` (corrección ortográfica adaptada a español).
- Script canónico de verificación de calidad `scripts/verificar.sh` con detección modular de herramientas y workspace.
- Hook de pre-commit local en `.githooks/pre-commit` para validación automatizada previa a cada confirmación.
- Flujo de Integración Continua en GitHub Actions (`.github/workflows/ci.yml`) con ejecución de pruebas, auditorías y validación de enlaces con lychee.
- Skills del workspace para desarrollo asistido en `.agents/skills/` (`rust-guidelines`, `ferrite-verificar`, `ferrite-docs`, `vrchat-osc`) con referencias vendorizadas.
- Documento de explicación sobre estándares y herramientas de desarrollo para agentes en `docs/explicacion/herramientas-agente.md`.
- Especificación técnica y matriz de verificación de trackers OSC de VRChat en `docs/referencia/vrchat-osc-trackers.md`.
- Reglas del proyecto para agentes y desarrolladores en `AGENTS.md` (mapa de crates, comandos canónicos, convenciones de código en tiempo real y tipado, DoD y tabla de skills).
- Estructura base del repositorio y licencias permisivas duales (MIT y Apache-2.0).
- Guía de contribución (`CONTRIBUTING.md`) con flujo de trabajo sobre rama `dev`, Conventional Commits y Definition of Done.
- Registro de Decisiones de Arquitectura (ADR) iniciales en `docs/adr/` (ADR-0000 a ADR-0008) en formato MADR.
- Configuración para editores (`.editorconfig`) y exclusiones de Git (`.gitignore`).
- Estructura de documentación del proyecto siguiendo el marco Diátaxis en `docs/`.

### Cambiado
- Formato de `deny.toml` adaptado a la especificación de `cargo-deny` 0.20 manteniendo la denegación de licencias GPL/AGPL y verificación de avisos de seguridad.

### Corregido

### Eliminado
- Archivo `.gitkeep` en `docs/tutoriales/` tras redactar el tutorial de VRChat.
- Archivo `.gitkeep` en `docs/guias/` tras incorporar la guía de instalación de CUDA y TensorRT.
- Archivos `.gitkeep` en `docs/explicacion/` y `docs/referencia/` tras incorporar documentación oficial.
