# Guía de contribución a FerriteVR

¡Gracias por tu interés en contribuir a FerriteVR! Este documento describe las normas, el flujo de trabajo de Git y las convenciones de código necesarias para mantener la calidad y consistencia del proyecto.

---

## 1. Flujo de trabajo en Git

- **Rama de desarrollo (`dev`):** Todo el desarrollo activo se integra en la rama `dev`. Todos los cambios, ramas de trabajo y pull requests deben tener como destino `dev`.
- **Rama principal (`main`):** Reservada exclusivamente para versiones estables publicadas. Solo se fusiona `dev` en `main` cuando se prepara un lanzamiento oficial acompañado de una etiqueta de versión (`vX.Y.Z`).
- **Commits directos y atómicos:** Realiza commits pequeños, autocontenidos y frecuentes.
- **Historial protegido:** Queda estrictamente prohibido usar `git push --force` o reescribir el historial en ramas compartidas o públicas.
- **Sin privilegios elevados:** Nunca ejecutes comandos con `sudo` en tareas automatizadas o scripts. Si una tarea requiere privilegios de administración (como configurar reglas de udev o instalar paquetes del sistema), debe documentarse para que el usuario la revise y ejecute manualmente.

---

## 2. Formato de commits (Conventional Commits)

Los mensajes de commit deben seguir la especificación [Conventional Commits](https://www.conventionalcommits.org/es/). La descripción debe estar redactada en **español**, en modo **imperativo**, y los tipos se mantienen en minúsculas en inglés conforme a la especificación estándar:

- `feat`: Nueva funcionalidad para el usuario o sistema.
- `fix`: Corrección de un error o fallo.
- `docs`: Modificaciones exclusivamente en la documentación o comentarios.
- `test`: Adición o corrección de pruebas unitarias o de integración.
- `refactor`: Refactorización de código sin cambios en funcionalidad ni correcciones.
- `perf`: Mejoras de rendimiento y optimización temporal o espacial.
- `chore`: Tareas de mantenimiento, actualización de dependencias o configuración interna.
- `ci`: Modificaciones en flujos de integración continua o scripts de verificación.

### Formato general:

```text
tipo(alcance): descripción breve en imperativo y español

[cuerpo opcional explicando el porqué del cambio]
```

**Ejemplos:**
- `feat(osc): agrega envío de trackers en un único bundle`
- `fix(sensor): corrige desbordamiento temporal en el buffer de profundidad`
- `docs(adr): registra decisión sobre backend de inferencia`

---

## 3. Convenciones de código y arquitectura

FerriteVR es un proyecto de alto rendimiento en Rust para tiempo real (realidad virtual). Sigue rigurosamente estas pautas:

1. **Idioma español (ASCII):**
   - Toda la documentación, comentarios, rustdoc y mensajes de commit deben estar en español.
   - Los **identificadores** de código (funciones, variables, structs, enums, módulos) se redactan en español utilizando exclusivamente caracteres ASCII (sin tildes ni `ñ`; por ejemplo: `Articulacion`, `pos_cadera`, `anio`, `Rotacion`). Esto se audita mediante el lint `non_ascii_idents = "deny"`.
   - Los nombres de crates internos siguen el patrón `ferrite-*` (por ejemplo, `ferrite-nucleo`, `ferrite-osc`).

2. **Gestión de errores:**
   - En librerías (`crates/*`): Emplea exclusivamente tipos de error específicos y tipados usando `thiserror`.
   - En binarios CLI (`apps/*`): Se permite el uso de `anyhow` para el manejo en el nivel superior de la aplicación.
   - **Prohibido `unwrap()` o `expect()`** fuera de los tests unitarios.

3. **Código inseguro (`unsafe`):**
   - El uso de `unsafe` está estrictamente restringido al crate FFI `ferrite-freenect2-sys`.
   - Todo bloque o función `unsafe` debe ir precedido obligatoriamente por un comentario explicativo `// SEGURIDAD:` que justifique por qué se cumplen los invariantes de seguridad.

4. **Rendimiento en tiempo real:**
   - Cero asignaciones dinámicas por frame en el camino caliente (reutilización de buffers preasignados).
   - Sin operaciones bloqueantes de entrada/salida (E/S) en los hilos del pipeline de procesamiento.
   - Canales "último-gana" entre hilos para descartar fotogramas antiguos si una etapa se retrasa y evitar acumulación de latencia.

5. **Espacios de coordenadas tipados:**
   - Se utilizan tipos específicos (`PosKinect` para mano derecha en metros, `PosUnity` para mano izquierda con eje Y hacia arriba en metros). Nunca pases vectores genéricos sin contexto entre módulos que crucen sistemas de referencia.

---

## 4. Activación de Git Hooks

Para asegurar que todo commit cumpla con los estándares de formato y análisis estático antes de integrarse, activa los hooks configurados en el repositorio:

```bash
git config core.hooksPath .githooks
```

El hook de pre-commit validará automáticamente el formateo de código (`rustfmt`), lints de calidad (`clippy`) y ortografía (`typos`).

---

## 5. Criterios de aceptación (Definition of Done)

Para considerar completado cualquier paquete de trabajo o contribución:

- [ ] `./scripts/verificar.sh` finaliza exitosamente sin errores ni advertencias (a partir de su introducción en la fase de tooling; antes de ello, `cargo build` y `cargo test`).
- [ ] Pruebas unitarias o de integración implementadas para toda lógica nueva o modificada.
- [ ] Documentación rustdoc en español para todos los elementos públicos (`pub`).
- [ ] Entrada correspondiente en `CHANGELOG.md` bajo la sección `## [Sin publicar]` si existe cambio observable de comportamiento.
- [ ] Registro de Decisión de Arquitectura (ADR) creado en `docs/adr/` si se adopta o altera una decisión técnica relevante.
- [ ] Cumplimiento total de las licencias permitidas (MIT o Apache-2.0, sin dependencias GPL/AGPL).
