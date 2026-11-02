# Documentación de FerriteVR

Bienvenido al centro de documentación técnica de FerriteVR. Esta documentación está organizada siguiendo el marco metodológico [Diátaxis](https://diataxis.fr/), que clasifica la información técnica en cuatro cuadrantes según el propósito y la necesidad del lector, complementado con un registro formal de decisiones arquitectónicas (ADR).

---

## Estructura Diátaxis

```
docs/
├── tutoriales/   # Aprendizaje práctico orientado a principiantes
├── guias/        # Recetas paso a paso para resolver problemas concretos
├── referencia/   # Especificaciones técnicas, APIs y formatos de datos
├── explicacion/  # Análisis conceptual, arquitectura y diseño a fondo
└── adr/          # Registro de Decisiones de Arquitectura (MADR)
```

### 1. Tutoriales (`docs/tutoriales/`)
Lecciones prácticas orientadas a guiar al usuario en sus primeras experiencias con el sistema:
- *Primera prueba con VRChat:* Configuración básica, ejecución en modo sintético (`fake`) y verificación de trackers frente al espejo.

### 2. Guías prácticas (`docs/guias/`)
Recetas y procedimientos directos dirigidos a resolver problemas o configuraciones específicas:
- *Compilación de libfreenect2:* Guía de resolución de problemas e instalación nativa.
- *Instalación de aceleración CUDA y TensorRT:* Pasos para entornos Nobara/Fedora con drivers propietarios de NVIDIA.

### 3. Referencia (`docs/referencia/`)
Especificaciones técnicas rigurosas y descripciones de parámetros y protocolos:
- *Trackers OSC de VRChat:* Direcciones de red, espacios de coordenadas, orden Euler y asignación de ranuras.
- *Modelos de pose:* Detalles del modelo RTMPose Halpe26 ONNX, normalización y formato de entradas/salidas.
- *Formato `.ferrite`:* Estructura de encabezado y flujo de compresión zstd para almacenamiento de sesiones de sensor.
- *Configuración TOML:* Diccionario de opciones soportadas en el archivo de configuración.

### 4. Explicación (`docs/explicacion/`)
Documentos de profundización conceptual, fundamentos teóricos y decisiones de ingeniería:
- *Arquitectura del pipeline:* Funcionamiento de canales último-gana y presupuesto temporal de latencia (< 25 ms).
- *El solver cinemático 3D:* Elevación mediante profundidad mediana, filtro One Euro adaptativo, bases ortonormales para rotaciones y mitigación de oclusiones.
- *Herramientas para agentes:* Evaluación de fuentes y directrices para desarrollo asistido por IA.

---

## Registro de Decisiones de Arquitectura (ADR)

Las decisiones estructurales, de arquitectura y de estándares del proyecto se formalizan en documentos MADR (*Markdown Any Architectural Decision Records*) ubicados en [`docs/adr/`](adr/):

| ADR | Título | Estado | Fecha |
|---|---|---|---|
| [ADR-0000](adr/0000-plantilla.md) | Plantilla de Decisiones de Arquitectura | Aceptado | 2026-11-02 |
| [ADR-0001](adr/0001-workspace-rust.md) | Organización del código como Cargo Workspace | Aceptado | 2026-11-02 |
| [ADR-0002](adr/0002-vrchat-osc-primero.md) | Emisión de trackers mediante protocolo OSC para VRChat en v1 | Aceptado | 2026-11-02 |
| [ADR-0003](adr/0003-pose-2d-mas-profundidad.md) | Estimación de pose 2D combinada con nube de profundidad Time-of-Flight | Aceptado | 2026-11-02 |
| [ADR-0004](adr/0004-rtmpose-por-licencia.md) | Selección de RTMPose Halpe26 bajo licencia Apache-2.0 | Aceptado | 2026-11-02 |
| [ADR-0005](adr/0005-licencia-dual-mit-apache.md) | Licencia dual MIT y Apache-2.0 | Aceptado | 2026-11-02 |
| [ADR-0006](adr/0006-backend-de-computo-por-benchmark.md) | Selección del backend de cómputo predeterminado mediante benchmark con VR activo | Propuesto | 2026-11-02 |
| [ADR-0007](adr/0007-idioma-espanol.md) | Adopción del idioma español en código, documentación e identificadores ASCII | Aceptado | 2026-11-02 |
| [ADR-0008](adr/0008-flujo-git-dev-main.md) | Flujo de trabajo en Git con rama dev de desarrollo y main para versiones estables | Aceptado | 2026-11-02 |
