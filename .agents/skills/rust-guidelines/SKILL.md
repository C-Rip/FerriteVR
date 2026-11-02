---
name: rust-guidelines
description: Proporciona directrices pragmaticas de diseno y optimizacion en Rust adaptadas a tiempo real, FFI y cero asignaciones en FerriteVR. Usar al disenar o refactorizar codigo Rust en crates del workspace.
---

# Rust Guidelines para FerriteVR

Esta skill sintetiza las mejores prácticas de ingeniería de Rust para **FerriteVR**, adaptando las *Microsoft Pragmatic Rust Guidelines* y las *Rust API Guidelines* a un entorno de visión por computador, tracking y tiempo real estricto (< 25 ms de latencia por frame).

Para la referencia completa de reglas vendorizadas, consultar [`references/pragmatic-rust-guidelines.txt`](references/pragmatic-rust-guidelines.txt).

---

## 1. Reglas prioritarias para FerriteVR

### Rendimiento y Tiempo Real
- **`M-MEM-REUSE`:** Reutiliza buffers preasignados en lugar de asignar memoria por frame. En el bucle de visión y cálculo cinemático nunca se debe crear un `Vec::new()` o clonar buffers grandes en caliente.
- **`M-HOTPATH`:** Mantén el camino caliente libre de E/S, logging excesivo, serialización pesada o contención de bloqueos (`Mutex`/`RwLock`).
- **`M-AVOID-INDIRECTION`:** Prefiere tipos concretos y monomorfización estática (`impl Trait`) sobre *trait objects* dinámicos (`Box<dyn Trait>`) en los pasos internos del pipeline de alta frecuencia.

### Arquitectura de Tipos y Seguridad
- **`M-STRONG-TYPES`:** Modela dominios geométricos incompatibles mediante newtypes explícitos (`PosKinect` vs. `PosUnity`). No utilices tuplas anónimas ni vectores primitivos en APIs públicas que crucen sistemas de referencia.
- **`M-UNSAFE`:** El código `unsafe` está estrictamente restringido a `ferrite-freenect2-sys`. Toda invocación o bloque inseguro debe documentar un comentario explicativo obligatorio con el formato:
  ```rust
  // SEGURIDAD: Invariante verificado que garantiza que el puntero es valido y no nulo...
  ```
- **`M-SYS-CRATES` & `M-FFI-TRANSLATES`:** Los detalles de bajo nivel de C/C++ se encapsulan en el crate `-sys`. Las capas superiores solo consumen abstracciones seguras e idiomáticas en Rust.

### Manejo de Errores
- **`M-ERRORS-CANONICAL-STRUCTS`:** En bibliotecas (`crates/*`), define enums de error exhaustivos y legibles mediante `thiserror`.
- **`M-APP-ERROR`:** En la aplicación de línea de comandos (`apps/ferrite`), utiliza `anyhow::Result` para propagación enriquecida con contexto de usuario.
- **`M-PANIC-ON-BUG`:** No uses `.unwrap()` ni `.expect()` en código de producción. Propaga errores con `?` o maneja explícitamente `Result`/`Option`. Los panics solo se toleran si representan violaciones matemáticas irrecuperables o en tests unitarios.

---

## 2. Convenciones de estilo específicas del proyecto

1. **Identificadores en español ASCII:**
   - Todo nombre de función, método, estructura, variable y constante debe redactarse en español empleando únicamente caracteres ASCII (`pos_cadera`, `Articulacion`, `longitud_brazo`).
   - El código se compila con `#![deny(non_ascii_idents)]`.
2. **Documentación:**
   - Rustdoc en español para cada función, tipo o constante pública, detallando precondiciones, unidades de medida y errores retornados.
