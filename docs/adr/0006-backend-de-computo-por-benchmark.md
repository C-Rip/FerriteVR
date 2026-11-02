# ADR-0006: Selección del backend de cómputo predeterminado mediante benchmark con VR activo

* **Estado:** Propuesto
* **Fecha:** 2026-11-02
* **Decisores:** Equipo de FerriteVR
* **Hito de resolución:** Se cerrará formalmente en el paquete de trabajo WP-4.4 tras la ejecución de pruebas empíricas.

## Contexto y planteamiento del problema

La etapa de inferencia de redes neuronales (RTMPose) en FerriteVR se ejecuta a través del motor ONNX Runtime (`ort`), el cual admite diversos *Execution Providers* (proveedores de ejecución o backends de cómputo):
1. **CPU:** Ejecución en procesador central optimizada con instrucciones vectoriales (AVX2 / AVX-512) mediante OpenVINO o CPU EP nativo.
2. **CUDA:** Aceleración por GPU mediante NVIDIA CUDA y cuDNN.
3. **TensorRT:** Compilación de grafos neuronales y optimización de precisión mixta (FP16 / INT8) directamente para los núcleos Tensor de NVIDIA.

El entorno de ejecución del usuario consta de una CPU Intel Core Ultra 5 225F y una GPU NVIDIA GeForce RTX 3060 12 GB bajo Nobara Linux 44. Simultáneamente, la GPU tiene que atender tareas de alta exigencia en tiempo real:
- Renderizado de VRChat en Proton/SteamVR.
- Codificación de vídeo por hardware en tiempo real (NVENC) requerida por Steam Link para la transmisión inalámbrica a las Meta Quest 3S.
- Composición de Wayland y escritorio KDE.

Elegir ciegamente el backend más rápido en pruebas aisladas de inferencia (por ejemplo, TensorRT o CUDA) podría saturar la GPU, provocando pérdida de fotogramas (*frame drops*), reproyección temporal en el visor VR o saltos de audio. Por otro lado, forzar el uso de CPU podría incrementar la latencia o sobrecargar los hilos del planificador del sistema operativo.

## Criterios de decisión

* **Estabilidad de VR inmutable:** Cero reproyección añadida y mantenimiento de 90 / 120 FPS estables en el visor VR (medido en el gráfico de frame timing de SteamVR).
* **Latencia de inferencia:** Tiempo de inferencia de pose (p50 y p99) inferior a 12 ms por fotograma.
* **Consumo de memoria VRAM y CPU:** Evitar la contención de ancho de banda de memoria de vídeo (VRAM) con el juego.
* **Complejidad de dependencias y despliegue:** Facilidad de instalación y compatibilidad con drivers NVIDIA 595 en Nobara Linux.

## Opciones consideradas

* **Opción 1: Fijar CUDA como backend predeterminado.** Asumir aceleración estándar por GPU.
* **Opción 2: Fijar TensorRT (FP16) como backend predeterminado.** Máximo rendimiento teórico de inferencia en tarjetas RTX.
* **Opción 3: Fijar CPU como backend predeterminado.** Aislar por completo la inferencia de la GPU para reservar todos los recursos de vídeo a VRChat y Steam Link.
* **Opción 4: Selección empírica del backend predeterminado mediante benchmark en condiciones reales con VR activo.** Implementar soporte configurable para los tres backends y seleccionar el valor predeterminado solo tras medir la latencia p50/p99 y la reproyección en SteamVR con VRChat + Steam Link en marcha (WP-4.4).

## Decisión tomada

Opción elegida: **Opción 4: Selección empírica del backend predeterminado mediante benchmark en condiciones reales con VR activo**.

El subcomando `ferrite bench` medirá la latencia p50 y p99 de cada etapa del pipeline y de cada backend de inferencia disponible. El backend que se configure por defecto en FerriteVR será **aquel que ofrezca la menor latencia de inferencia sin degradar en absoluto los fotogramas por segundo (FPS) ni provocar reproyección en SteamVR durante el juego activo**.

### Plan de validación empírica (WP-4.4)

Se completará la siguiente matriz de evaluación bajo carga real de juego:

| Backend | Latencia Inferencia (p50) | Latencia Inferencia (p99) | Uso VRAM adicional | Reproyección en SteamVR (%) | FPS en VRChat | Veredicto |
|---|---|---|---|---|---|---|
| CPU (OpenVINO / Default) | *Por medir* | *Por medir* | 0 MB | *Por medir* | *Por medir* | Pendiente WP-4.4 |
| CUDA | *Por medir* | *Por medir* | *Por medir* | *Por medir* | *Por medir* | Pendiente WP-4.4 |
| TensorRT (FP16) | *Por medir* | *Por medir* | *Por medir* | *Por medir* | *Por medir* | Pendiente WP-4.4 |

### Consecuencias

* **Positivas:**
  * Garantiza que la experiencia de realidad virtual sea fluida y confortable sin inducir mareo por movimiento (*motion sickness*) debido a contención de GPU.
  * Flexibilidad para que el usuario pueda alternar entre backends mediante configuración TOML o flags de CLI (`--backend cpu|cuda|tensorrt`).
* **Negativas:**
  * El estado de esta decisión permanece como *Propuesto* hasta que el usuario y el subagente ejecuten las pruebas con hardware en el WP-4.4.
* **Neutrales:**
  * Requiere mantener dependencias opcionales en Cargo mediante features (`cuda`, `tensorrt`).

## Pros y contras de las opciones

### Opción 1: CUDA estático

* **Bueno:** Rendimiento predecible y acelerado.
* **Malo:** Puede competir con NVENC y la memoria VRAM de VRChat en mundos muy cargados.

### Opción 2: TensorRT estático

* **Bueno:** Máxima velocidad de cálculo en arquitecturas Ampere.
* **Malo:** Requiere tiempo de compilación del motor (*engine compilation*) en el primer arranque y consume recursos de memoria significativos.

### Opción 3: CPU estática

* **Bueno:** Desacopla 100% la carga de la tarjeta gráfica.
* **Malo:** Puede elevar la latencia por encima del presupuesto de 25 ms del pipeline.

### Opción 4: Benchmark empírico bajo carga VR

* **Bueno:** Decisión basada en datos reales de hardware y telemetría de juego.
* **Malo:** Pospone la decisión definitiva a la fase de pruebas de inferencia.

## Enlaces y referencias

* [ONNX Runtime Execution Providers](https://onnxruntime.ai/docs/execution-providers/)
* [SteamVR Frame Timing Guide](https://developer.valvesoftware.com/wiki/SteamVR/Frame_Timing)
