# ADR-0001: Organización del código como Cargo Workspace

* **Estado:** Aceptado
* **Fecha:** 2026-11-02
* **Decisores:** Equipo de FerriteVR

## Contexto y planteamiento del problema

FerriteVR es un sistema modular compuesto por diversas responsabilidades con requisitos de compilación y dependencias muy distintos:
1. Definición de tipos comunes, representaciones matemáticas y espacios de coordenadas.
2. Comunicación por red UDP con codificación de paquetes OSC para VRChat.
3. Abstracción de captura de frames, lectura de sensores y reproducción de grabaciones.
4. Vinculación FFI con la librería en C++ `libfreenect2` (código que requiere compilador C++, `pkg-config` y manejo de `unsafe`).
5. Inferencia de redes neuronales con ONNX Runtime (`ort`).
6. Solver cinemático 3D, filtros temporales y algoritmos de mitigación de oclusiones.
7. Aplicación ejecutable de línea de comandos (CLI).

Mantener todo en un solo paquete (*monolithic crate*) provocaría tiempos de compilación lentos, dificultad para aislar el código inseguro (`unsafe`), y problemas en los entornos de integración continua (CI) donde librerías nativas como `libfreenect2` o aceleradores de GPU no están presentes.

## Criterios de decisión

* Aislamiento estricto de código inseguro FFI (`unsafe`).
* Modularidad y tiempos de compilación incrementales eficientes.
* Capacidad de ejecutar CI y tests sin requerir hardware físico (Kinect) ni dependencias nativas complejas.
* Facilidad de reutilización interna de estructuras y tipos de coordenadas.

## Opciones consideradas

* **Opción 1: Crate monolítico único.** Todo el código en un único crate `ferritevr` con múltiples módulos internos (`src/nucleo/`, `src/osc/`, `src/sys/`, etc.).
* **Opción 2: Repositorios separados (multi-repo).** Un repositorio independiente de Git para cada librería (`ferrite-osc`, `ferrite-freenect2`, `ferrite-solver`, etc.).
* **Opción 3: Cargo Workspace (monorepo con crates internos).** Un único repositorio organizado con un workspace de Cargo con miembros `crates/*` y `apps/*`.

## Decisión tomada

Opción elegida: **Opción 3: Cargo Workspace (monorepo con crates internos)**, estructurado de la siguiente forma:

```
crates/
├── ferrite-nucleo/        # Tipos, espacios de coordenadas y modelos matemáticos
├── ferrite-osc/           # Emisor OSC para VRChat
├── ferrite-sensor/        # Trait FuenteFrames, reproducción y Kinect
├── ferrite-freenect2-sys/ # Puente CXX con libfreenect2 (único con unsafe)
├── ferrite-pose/          # Inferencia ONNX (ort)
└── ferrite-solver/        # Solver 3D, One Euro, oclusión, rotaciones
apps/
└── ferrite/               # Binario CLI
```

### Consecuencias

* **Positivas:**
  * Separación rigurosa de responsabilidades con fronteras de visibilidad claras (`pub`, `pub(crate)`).
  * Aislamiento total de `unsafe_code` exclusivamente en `ferrite-freenect2-sys`; el resto de crates compila con `#![forbid(unsafe_code)]` o `#![deny(unsafe_code)]`.
  * La CI puede compilar y testear la mayor parte del sistema omitiendo las features de hardware (`kinect`, `cuda`).
  * Dependencias compartidas unificadas en `[workspace.dependencies]` del `Cargo.toml` raíz.
* **Negativas:**
  * Mayor cantidad de archivos `Cargo.toml` que gestionar en el repositorio.
* **Neutrales:**
  * Requiere configurar perfiles y lints a nivel de workspace.

## Pros y contras de las opciones

### Opción 1: Crate monolítico único

* **Bueno:** Fácil de configurar inicialmente con un solo `Cargo.toml`.
* **Malo:** No permite restringir `unsafe` a nivel de crate; cualquier fallo de compilación en FFI bloquea todo el proyecto.
* **Malo:** Acoplamiento elevado e imposibilidad de ejecutar tests en entornos sin `libfreenect2` instalada.

### Opción 2: Repositorios separados

* **Bueno:** Máxima independencia entre módulos.
* **Malo:** Sobrecarga masiva de sincronización de versiones, issues y pull requests para un proyecto coordinado.
* **Malo:** Dificulta el refactorizado atómico entre tipos del solver y del núcleo.

### Opción 3: Cargo Workspace

* **Bueno:** Flujo unificado en un solo repositorio con modularidad estricta y compilación paralela.
* **Bueno:** Facilita la adopción de features opcionales por componente.
* **Malo:** Requiere disciplina en la gestión de dependencias entre crates internos.

## Enlaces y referencias

* [Cargo Workspaces Documentation](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
* [FerriteVR Plan Maestro v3 - Sección 1: Estructura objetivo del repositorio](../../.agents/planes/plan-maestro.md)
