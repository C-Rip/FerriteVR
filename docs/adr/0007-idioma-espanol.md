# ADR-0007: Adopción del idioma español en código, documentación e identificadores ASCII

* **Estado:** Aceptado
* **Fecha:** 2026-11-02
* **Decisores:** Equipo de FerriteVR

## Contexto y planteamiento del problema

FerriteVR es un proyecto concebido, desarrollado y mantenido por un equipo hispanohablante. Tradicionalmente en el desarrollo de software se adopta el idioma inglés por defecto; sin embargo, esto suele introducir fricciones de traducción, inconsistencias terminológicas y pérdida de matices en la documentación de dominio cinemático y geométrico.

Se debe definir una política idiomática uniforme y coherente que aplique a:
1. Documentación general, guías y tutoriales (`docs/`, `README.md`, etc.).
2. Documentación interna de código (`rustdoc`, comentarios en el código fuente).
3. Mensajes de commit y registro de cambios (`CHANGELOG.md`).
4. Nombres de identificadores en el código (nombres de funciones, structs, enums, variables, módulos y constantes).

Adicionalmente, el uso de caracteres tipográficos especiales del español (como tildes `á`, `é`, `í`, `ó`, `ú` o la letra `ñ`) dentro de identificadores de código fuente puede generar problemas de interoperabilidad en herramientas de compilación, terminales, generación de bindings FFI con C++ (`cxx`) o sistemas de archivos.

## Criterios de decisión

* Coherencia total entre la documentación, el dominio del problema y el código fuente.
* Eliminación de barreras idiomáticas para el equipo de desarrollo principal.
* Compatibilidad técnica absoluta con compiladores, FFI de C++ y herramientas de análisis estático sin problemas de codificación.
* Capacidad de verificación automatizada mediante lints del compilador de Rust.

## Opciones consideradas

* **Opción 1: Todo en inglés.** Identificadores, rustdoc, documentación externa y commits redactados íntegramente en inglés.
* **Opción 2: Documentación en español y código en inglés.** Modelo mixto con código tradicional en inglés y explicaciones en español.
* **Opción 3: Todo en español con identificadores en español Unicode.** Permitir identificadores con caracteres como `articulación` o `diseño`.
* **Opción 4: Todo en español con identificadores en español estrictamente ASCII.** Documentación, rustdoc, comentarios y commits en español estándar; identificadores en español pero restringidos al alfabeto ASCII sin acentos ni eñes (ej. `Articulacion`, `pos_cadera`, `disenio`, `anio`).

## Decisión tomada

Opción elegida: **Opción 4: Todo en español con identificadores en español estrictamente ASCII**.

Se establecen las siguientes reglas obligatorias:
1. **Documentación, comentarios y commits:** Todo el contenido de `docs/`, `README.md`, `CHANGELOG.md`, `rustdoc` (incluyendo ejemplos y doc-tests) y mensajes de commit se escribe en español correcto, utilizando ortografía y acentuación estándar.
2. **Identificadores de código (ASCII):** Todos los identificadores (crates, módulos, structs, enums, variantes, traits, funciones, métodos, variables y constantes) deben reflejar nombres en español pero empleando exclusivamente caracteres ASCII (sin tildes, sin diéresis y reemplazando la `ñ` por `ni` o equivalentes fonéticos claros: ej. `Articulacion`, `Rotacion`, `MuestraTracker`, `pos_cadera`, `anio`).
3. **Control automatizado:** Se activa el lint del compilador de Rust:
   ```toml
   [workspace.lints.rust]
   non_ascii_idents = "deny"
   ```
4. **Nomenclatura de crates:** Todos los paquetes del workspace adoptan el prefijo `ferrite-` seguido de su nombre en español ASCII (`ferrite-nucleo`, `ferrite-osc`, `ferrite-sensor`, `ferrite-pose`, `ferrite-solver`).
5. **Conventional Commits:** Los tipos de commit (`feat`, `fix`, `docs`, `chore`, etc.) se conservan en inglés según la especificación internacional, pero el alcance y la descripción se redactan en imperativo en español (ej. `feat(osc): agrega emisor de trackers`).

### Consecuencias

* **Positivas:**
  * Máxima claridad y naturalidad conceptual al programar; la nomenclatura del código coincide directamente con los términos discutidos en el diseño.
  * Cero ambigüedades derivadas de traducciones intermedias.
  * Compatibilidad técnica absoluta: ningún enlace FFI con C++ ni herramienta externa falla por discrepancias de codificación UTF-8 en identificadores.
  * Verificación rigurosa mediante `cargo check` y `clippy`: cualquier introducción accidental de caracteres no ASCII en nombres es rechazada inmediatamente por el compilador.
* **Negativas:**
  * Colaboradores no hispanohablantes requerirán herramientas de traducción automática para comprender la base de código.
* **Neutrales:**
  * Requiere disciplina para no incluir tildes ni la letra `ñ` en nombres de variables y métodos.

## Pros y contras de las opciones

### Opción 1: Todo en inglés

* **Bueno:** Facilita potenciales contribuciones de la comunidad internacional de código abierto.
* **Malo:** Desconecta el código de la lengua nativa de los desarrolladores principales y ralentiza la conceptualización del dominio.

### Opción 2: Modelo mixto

* **Bueno:** Práctica habitual en algunos equipos locales.
* **Malo:** Provoca inconsistencias constantes ("Spanglish"), mezcla de términos y duplicación conceptual.

### Opción 3: Identificadores en español con Unicode

* **Bueno:** Respeta la ortografía formal de la lengua española en el código.
* **Malo:** Falla o introduce dolores de cabeza con FFI C++ (`cxx_build`), ciertas macros de Rust y herramientas de generación de código.

### Opción 4: Todo en español con ASCII estricto

* **Bueno:** Claridad conceptual total sin ningún riesgo de incompatibilidad técnica.
* **Bueno:** Auditable de manera automática mediante `non_ascii_idents = "deny"`.
* **Malo:** Requiere adaptación a nombres como `Articulacion` o `anio`.

## Enlaces y referencias

* [Rust Reference: Identifiers](https://doc.rust-lang.org/reference/identifiers.html)
* [Rust Compiler Lint: non_ascii_idents](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#non-ascii-idents)
