# Registro de cambios

Todos los cambios notables en este proyecto se documentarán en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/),
y este proyecto se adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [Sin publicar]

### Corregido
- Enlaces relativos rotos, rutas absolutas `file://` y URLs remotas de documentación en Markdown, agregando archivo de configuración `lychee.toml` para que la validación de enlaces en CI pase con éxito.
- Advertencias de Clippy (`needless_range_loop`, `assign_op_pattern`, `len_zero`) y formateo de código con `rustfmt` en todo el workspace para que la suite de verificación canónica termine con código 0.
- Exención en pruebas (`cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))`) para permitir aserciones directas en tests unitarios e integrados sin violar la prohibición en código de producción.

### Agregado
- Calibración y alineación espacial (`CalibracionEspacio`) en `ferrite-solver` con elevación respecto al piso, recentrado en Z y corrección de inclinación (*pitch*) hacia el espacio de juego de VRChat.
- Mitigación de oclusiones (`GestorOclusion`) con retención de últimas poses válidas, decaimiento exponencial suave y detección de orientación de espaldas.
- Silueta humanoide con brazos, pies y rostro en `GeneradorSinteticoSensor` para reconocimiento robusto en RTMPose emitiendo los 9 trackers de VRChat.
- Conexión del pipeline de seguimiento corporal en tiempo real en la CLI (`apps/ferrite`) unificando `ferrite-sensor` -> `ferrite-pose` -> `ferrite-solver` -> `ferrite-osc`.
- Subcomando `reproducir` en CLI para emitir seguimiento de cuerpo completo desde archivos de sesión `.ferrite` o fotogramas sintéticos hacia VRChat OSC a 30 FPS.
- Subcomando `grabar` en CLI para registrar sesiones de prueba comprimidas con `zstd` en formato `.ferrite`.
- Subcomando `bench` en CLI para medición comparativa de latencias p50/p90/p99 y tasa de fotogramas global (alcanzando 65.7 FPS y 14.63 ms p50 en CPU release).
- Rebobinado y reproducción en bucle continuo (`--bucle`) en `LectorFerrite` y `FuenteReproduccion`.
- Crate `ferrite-solver` con elevación 3D y desproyección estenopeica mediante muestreo de mediana $5 \times 5$, filtro adaptativo temporal One Euro para posiciones y rotaciones (SLERP), cálculo de bases ortonormales anatómicas para cadera y pies con Gram-Schmidt sin división por cero ni `NaN`, y orquestación integral en `SolverCinematico`.
- Documento de explicación de la arquitectura cinemática 3D en `docs/explicacion/solver.md`.
- Motor de inferencia neuronal `MotorPose` con ONNX Runtime (`ort`) para RTMPose-m (Halpe26) y decodificador SimCC (`argmax` a medio píxel) que extrae los 26 puntos clave 2D con su nivel de confianza y los reproyecta al espacio original del sensor.
- Permiso para licencia `CDLA-Permissive-2.0` en `deny.toml` para dependencias de certificados de red requeridas por `ort-sys`.
- Crate `ferrite-pose` con segmentación de sujeto en profundidad (`CajaDelimitadora`), recorte afín, interpolación bilineal y normalización continua NCHW (256×192) con preasignación de buffers para cero asignaciones por fotograma.
- Script canónico `scripts/descargar-modelos.sh` para descarga idempotente y verificación de suma SHA-256 de RTMPose-m (Halpe26) en formato ONNX.
- Especificación técnica del modelo de estimación de pose en `docs/referencia/modelos.md` detallando entradas normalizadas, representación SimCC de 26 articulaciones y su correspondencia con las ranuras de seguimiento de VRChat.
- Crate `ferrite-sensor` con el trait `FuenteFrames`, estructura `FrameSensor` (reutilización de buffers en caliente), formato de archivo de grabación binario comprimido con zstd (`.ferrite`), decodificador `LectorFerrite`, codificador `GrabadorFerrite`, reproductor en tiempo real `FuenteReproduccion` y generador sintético determinista `GeneradorSinteticoSensor`.
- Especificación técnica del formato de grabación `.ferrite` en `docs/referencia/formato-ferrite.md`.
- Estructura `IntrinsecosProfundidad` en `ferrite-nucleo` con métodos geométricos de proyección y desproyección estenopeica entre píxeles 2D y el espacio diestro `PosKinect`.
- Carpeta de memoria y planes en `.agents/planes/` con `plan-maestro.md` (especificación completa de paquetes de trabajo WP-0.1 a WP-6.4 con plantillas de prompts para subagentes) y `memoria-estado.md` (resumen ejecutivo de estado y traspaso para futuros agentes de IA como Claude, Gemini, Grok, DeepSeek o GLM).
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
