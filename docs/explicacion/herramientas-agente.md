# Herramientas y estándares para agentes en FerriteVR

Este documento explica las fuentes metodológicas y técnicas evaluadas para gobernar el desarrollo asistido por agentes de inteligencia artificial y desarrolladores humanos en FerriteVR, detallando el veredicto de adopción y su justificación técnica.

## 1. Fuentes evaluadas y veredictos

| Fuente | Autoridad / Organización | Licencia | Veredicto | Propósito en FerriteVR |
|---|---|---|---|---|
| **Microsoft Pragmatic Rust Guidelines** | Microsoft | MIT | **Adoptar** (adaptada) | Reglas pragmáticas de ingeniería Rust. Se vendoriza en `.agents/skills/rust-guidelines/references/` filtrando servicios web/Tokio y priorizando tiempo real, FFI y cero asignaciones. |
| **Rust API Guidelines** | Rust Project | MIT / Apache-2.0 | **Inspiración / adopción selectiva** | Principios idiomáticos oficiales (traits estándar, newtypes, robustez), adaptados a identificadores en español ASCII. |
| **Conventional Commits v1.0.0** | Comunidad abierta | CC BY 3.0 | **Adoptar** | Estandarización de historial Git. Tipos canónicos en inglés (`feat`, `fix`, `docs`), alcance y descripción imperativa en español. |
| **Keep a Changelog v1.1.0** | Olivier Lacan | MIT | **Adoptar** | Mantenimiento de `CHANGELOG.md` estructurado en español bajo `## [Sin publicar]`. |
| **Semantic Versioning (SemVer) 2.0.0** | Tom Preston-Werner | CC BY 3.0 | **Adoptar** | Numeración semántica (`MAJOR.MINOR.PATCH`) para lanzamientos estables. |
| **Marco Diátaxis** | Daniele Procida | CC BY-SA 4.0 | **Adoptar** | Arquitectura de documentación en cuatro cuadrantes (`tutoriales/`, `guias/`, `referencia/`, `explicacion/`). |
| **Markdown Any Decision Record (MADR)** | adr.github.io | CC0 / MIT | **Adoptar** | Plantilla y numeración secuencial para decisiones arquitectónicas en `docs/adr/`. |
| **VRChat OSC Trackers Spec** | VRChat Inc. | Documentación técnica oficial | **Adoptar** | Especificación de red y protocolo OSC para trackers de cuerpo completo vía UDP 9000. |
| **Repositorios comunitarios de skills** (`rust-skills`, `lang-guidelines`) | Varios desarrolladores | Varias | **Descartar adopción directa** | Solo inspiración estructural. Se prohíbe la instalación desatendida vía gestores como `npx` por riesgos de seguridad en la cadena de suministro. |

---

## 2. Detalle de decisiones por estándar

### Microsoft Pragmatic Rust Guidelines
- **URL fuente:** `https://microsoft.github.io/rust-guidelines/agents/all.txt`
- **Justificación:** Ofrece recomendaciones directas orientadas a la legibilidad y predictibilidad de código por agentes.
- **Filtro aplicado para FerriteVR:**
  - *Adoptadas:* `M-MEM-REUSE` (reutilización de memoria en pipeline), `M-HOTPATH` (cero asignaciones en caminos críticos), `M-AVOID-INDIRECTION` (evitar indirecciones dinámicas innecesarias), `M-STRONG-TYPES` (uso de newtypes como `PosKinect` y `PosUnity`), `M-FFI-TRANSLATES` y `M-SYS-CRATES` (aislamiento FFI en `ferrite-freenect2-sys`), `M-UNSAFE` (justificación estricta de invariantes con `// SEGURIDAD:`), `M-ERRORS-CANONICAL-STRUCTS` (`thiserror` en crates de biblioteca), `M-APP-ERROR` (`anyhow` en la aplicación CLI) y `M-PANIC-ON-BUG` (prohibición de `unwrap`/`expect` fuera de tests).
  - *Descartadas:* Pautas específicas para microservicios asíncronos cloud y Tokio intensivo, dado que el pipeline de visión y tracking opera sobre hilos dedicados de baja latencia con canales de capacidad 1.

### Rust API Guidelines
- **URL fuente:** `https://rust-lang.github.io/api-guidelines/`
- **Justificación:** Referencia estándar del ecosistema. Se adoptan los lineamientos de interoperabilidad de tipos (`Clone`, `Debug`, `Default`, `Eq`), constructores estándar (`new`), newtypes para dimensionalidad y separación de responsabilidades, respetando la directriz de FerriteVR de identificadores en español con caracteres exclusivamente ASCII (`non_ascii_idents = "deny"`).

### Diátaxis y MADR
- **Diátaxis:** Evita mezclar explicaciones teóricas con guías paso a paso o referencias puras.
  - `docs/tutoriales/`: Aprendizaje práctico guiado para principiantes.
  - `docs/guias/`: Recetas orientadas a tareas específicas (e.g., compilación de `libfreenect2`, instalación de CUDA).
  - `docs/referencia/`: Especificaciones técnicas objetivas (e.g., protocolo OSC de VRChat).
  - `docs/explicacion/`: Discusión y fundamentación conceptual (e.g., este documento).
- **MADR:** Todo cambio arquitectónico significativo (ADR) se documenta de forma inmutable y correlativa en `docs/adr/` partiendo de `0000-plantilla.md`.

### Seguridad en la cadena de suministro de skills
- Se rechaza explícitamente el uso de ejecutables o instaladores desatendidos de la comunidad (`npx skills add`, descargas de scripts curl piped to bash no verificados).
- Todas las skills residen vendorizadas de forma local en `.agents/skills/`, auditables mediante Git y ejecutables bajo sandbox sin permisos de superusuario (`sudo`).
