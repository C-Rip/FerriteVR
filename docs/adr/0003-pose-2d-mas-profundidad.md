# ADR-0003: Estimación de pose 2D combinada con nube de profundidad Time-of-Flight

* **Estado:** Aceptado
* **Fecha:** 2026-11-02
* **Decisores:** Equipo de FerriteVR

## Contexto y planteamiento del problema

El hardware utilizado en FerriteVR es un sensor Kinect v2 para Xbox One. Este dispositivo proporciona dos flujos principales de datos sincronizados geométricamente:
1. Imagen en color RGB (Full HD 1920×1080) o imagen infrarroja IR (512×424).
2. Mapa de profundidad Time-of-Flight (ToF) a resolución de 512×424 píxeles a 30 FPS, con precisión milimétrica en un rango de 0.5 a 4.5 metros.

Para obtener un esqueleto 3D preciso del usuario en tiempo real con un presupuesto de latencia estricto (< 25 ms), se debe determinar cómo transformar las imágenes capturadas por el sensor en coordenadas espaciales 3D tridimensionales (`PosKinect` y posteriormente `PosUnity`).

## Criterios de decisión

* Precisión métrica absoluta en la distancia y posición espacial de las articulaciones.
* Latencia de procesamiento inferior a 15 ms para la etapa de inferencia y cálculo geométrico.
* Consumo moderado de recursos computacionales sin saturar la GPU durante sesiones intensivas de VR.
* Robustez ante variaciones de ropa, fondos y condiciones de iluminación en habitaciones domésticas.

## Opciones consideradas

* **Opción 1: Estimación de pose 3D puramente monocular desde RGB.** Utilizar redes neuronales que estiman directamente coordenadas 3D relativas a partir de una única imagen RGB (como MediaPipe Pose 3D o modelos tipo HybrIK/Vibe).
* **Opción 2: Segmentación y ajuste de esqueleto 3D directo sobre la nube de puntos.** Procesar la nube de puntos de profundidad sin inferencia RGB (similar al enfoque clásico del SDK de Kinect mediante bosques de decisión aleatorios o redes PointNet/VoxelNet).
* **Opción 3: Pipeline híbrido (Detección de pose 2D en color/IR + Mediana de profundidad ToF medida).** Detectar puntos clave 2D (`u, v`) con una red neuronal convolucional ultrarrápida (RTMPose) y elevar cada articulación a 3D muestreando la profundidad física medida en una ventana local (5×5) del sensor ToF, aplicando proyección estenopeica (*pinhole*).

## Decisión tomada

Opción elegida: **Opción 3: Pipeline híbrido (Pose 2D + Muestreo de profundidad ToF medida)**.

La elevación de coordenadas 2D a 3D en el espacio de la cámara Kinect (`PosKinect`) se realiza mediante la fórmula estenopeica:

$$X = \frac{(u - c_x) \cdot z}{f_x}$$
$$Y = \frac{(v - c_y) \cdot z}{f_y}$$
$$Z = z$$

donde $(c_x, c_y)$ y $(f_x, f_y)$ son los parámetros intrínsecos de la cámara de profundidad, y $z$ es la mediana de los valores válidos de profundidad en una vecindad de 5×5 píxeles alrededor de $(u, v)$. Si no hay profundidad física válida, la confianza de la articulación se reduce a cero para evitar artefactos o valores NaN.

### Consecuencias

* **Positivas:**
  * La coordenada $Z$ (profundidad métrica) proviene de una medición física real por tiempo de vuelo (ToF), eliminando la ambigüedad de escala inherente a los estimadores 3D monoculares RGB.
  * Los modelos 2D contemporáneos son órdenes de magnitud más rápidos y ligeros que los modelos 3D densos o redes sobre nubes de puntos.
  * La mediana local de 5×5 absorbe eficazmente los píxeles negros sin retorno causados por superficies absorbentes de infrarrojos o bordes de silueta.
  * Permite delimitar dinámicamente el área de interés (*bounding box*) del usuario mediante un umbral de profundidad previo (0.5 a 4.5 m), acelerando el preprocesamiento de la red 2D.
* **Negativas:**
  * Depende de la correcta alineación y calibración geométrica entre el sensor de color y el de profundidad (facilitada por el registro de `libfreenect2`).
  * Requiere manejar la pérdida transitoria de profundidad si una extremidad queda temporalmente fuera del cono de emisión ToF.
* **Neutrales:**
  * Las rotaciones de las articulaciones deben derivarse cinemáticamente a partir de las relaciones geométricas entre articulaciones vecinas (manejado por `ferrite-solver`).

## Pros y contras de las opciones

### Opción 1: Pose 3D monocular RGB

* **Bueno:** No requiere calibrar ni registrar la profundidad con el color.
* **Malo:** La estimación de profundidad $Z$ es aproximada y sufre de deriva y oscilaciones constantes; el avatar flota o se hunde en el suelo en VR.
* **Malo:** Desperdicia la mayor fortaleza del Kinect v2: su sensor físico ToF.

### Opción 2: Nube de puntos directa

* **Bueno:** No depende del canal de color ni de la iluminación ambiental visible.
* **Malo:** Procesar nubes de puntos 3D densas o vóxeles en tiempo real a 30 FPS consume una cantidad desproporcionada de recursos de GPU/CPU.
* **Malo:** Dificultad para resolver articulaciones delgadas (pies, tobillos, codos) únicamente con siluetas de profundidad ruidosas.

### Opción 3: Híbrido 2D + Profundidad ToF

* **Bueno:** Máxima precisión métrica real combinada con la velocidad y exactitud visual de redes 2D modernas.
* **Bueno:** Inferencia en < 10 ms sobre GPU o CPU.
* **Malo:** Requiere el pipeline de registro de `libfreenect2`.

## Enlaces y referencias

* [OpenKinect libfreenect2 Registration Architecture](https://github.com/OpenKinect/libfreenect2)
* [RTMPose: Real-Time Multi-Person Pose Estimation](https://arxiv.org/abs/2303.07399)
