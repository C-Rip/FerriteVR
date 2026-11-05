---
name: ferrite-docs
description: Guia para la redaccion y mantenimiento de la documentacion de FerriteVR segun el marco Diataxis, gestion del archivo CHANGELOG.md y creacion y numeracion de registros de decision arquitectonica (ADR). Usar al redactar documentacion o actualizar el historial de cambios.
---

# Gestión Documental en FerriteVR

Esta skill establece las pautas para estructurar la documentación técnica, registrar cambios de versión y formalizar decisiones de arquitectura.

---

## 1. Matriz de documentación Diátaxis

FerriteVR organiza su documentación en cuatro cuadrantes según el marco [Diátaxis](https://diataxis.fr/). Utiliza esta matriz para decidir qué archivo crear o modificar:

| Tipo de cambio o necesidad | Carpeta de destino | Enfoque de redacción | Ejemplo en FerriteVR |
|---|---|---|---|
| Enseñar a un usuario a dar sus primeros pasos de principio a fin | `docs/tutoriales/` | Práctico, paso a paso, garantizando un resultado tangible. | `docs/tutoriales/primera-prueba-vrchat.md` |
| Resolver un problema técnico concreto o procedimiento de configuración | `docs/guias/` | Orientado a metas específicas, conciso y modular. | `docs/guias/compilar-libfreenect2.md` |
| Describir con exactitud parámetros, APIs, formatos de red o protocolos | `docs/referencia/` | Técnico, objetivo, exhaustivo y estructurado en tablas/listas. | `docs/referencia/vrchat-osc-trackers.md` |
| Explicar la teoría, conceptos de fondo o justificaciones de diseño | `docs/explicacion/` | Discursivo, analítico y conceptual. | `docs/explicacion/herramientas-agente.md` |

---

## 2. Mantenimiento del Registro de Cambios (`CHANGELOG.md`)

Todo cambio que agregue, modifique, corrija o elimine funcionalidad debe registrarse en `CHANGELOG.md` siguiendo el estándar [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/):

- Todas las entradas nuevas se colocan siempre bajo la sección `## [Sin publicar]`.
- Se agrupan bajo subencabezados estándar en español:
  - `### Agregado`: Nuevas funcionalidades o archivos principales.
  - `### Cambiado`: Modificaciones en comportamiento existente o refactorizaciones de interfaz.
  - `### Corregido`: Corrección de errores (*bug fixes*).
  - `### Eliminado`: Características o archivos retirados.
- Cada entrada debe ser un punto de viñeta conciso, redactado en español y mencionando los crates o archivos afectados si aplica.

---

## 3. Creación y numeración de ADRs (`docs/adr/`)

Cuando se tome una decisión estructural, de dependencias o de arquitectura que comprometa el desarrollo futuro:

1. **Numeración secuencial:** Identifica el número más alto en `docs/adr/` (ej. `0008`) e incrementa en uno con formato de 4 dígitos (ej. `0009-<nombre-descriptivo>.md`).
2. **Plantilla estándar:** Copia la estructura de [`docs/adr/0000-plantilla.md`](../../../docs/adr/0000-plantilla.md).
3. **Secciones obligatorias en español:**
   - **Título:** Formato `# ADR-XXXX: <Título>`
   - **Contexto y planteamiento del problema:** Motivación del cambio y restricciones técnicas.
   - **Conductores de la decisión:** Factores críticos evaluados (rendimiento, compatibilidad, licencias).
   - **Opciones consideradas:** Lista de alternativas analizadas con pros y contras.
   - **Decisión adoptada:** Elección final y justificación.
   - **Consecuencias:** Efectos positivos, negativos y riesgos asumidos.
4. **Registro en el índice:** Enlaza el nuevo ADR en la tabla de `docs/README.md`.
