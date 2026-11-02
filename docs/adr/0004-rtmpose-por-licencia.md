# ADR-0004: Selección de RTMPose Halpe26 bajo licencia Apache-2.0

* **Estado:** Aceptado
* **Fecha:** 2026-11-02
* **Decisores:** Equipo de FerriteVR

## Contexto y planteamiento del problema

Para implementar la estimación de puntos clave 2D en FerriteVR se requiere un modelo de redes neuronales optimizado para inferencia en tiempo real que pueda exportarse y ejecutarse en formato ONNX mediante `ort`.

Además de los requisitos de latencia (< 10 ms por frame) y precisión visual, existen dos factores determinantes:
1. **Puntos clave de los pies:** Para un seguimiento de cuerpo completo creíble en VRChat, la orientación e inclinación de los pies y tobillos es crítica. Los modelos con topología estándar COCO de 17 puntos no incluyen los dedos o talones del pie, lo que impide calcular rotaciones precisas para los trackers de los pies.
2. **Licenciamiento y cumplimiento legal:** FerriteVR es un proyecto de código abierto distribuido bajo licencia dual MIT y Apache-2.0. Las dependencias y modelos incluidos no deben introducir restricciones virales incompatibles ni riesgos de litigio por patentes o uso comercial.

## Criterios de decisión

* Licencia permisiva de código abierto compatible con la licencia dual MIT/Apache-2.0 de FerriteVR (prohibición estricta de licencias recíprocas como AGPLv3).
* Topología de esqueleto con soporte explícito para pies y articulaciones completas (mínimo 26 puntos clave).
* Compatibilidad nativa con exportación ONNX e inferencia vía ONNX Runtime (`ort`).
* Desempeño en latencia adaptado a tiempo real en hardware de escritorio.

## Opciones consideradas

* **Opción 1: Ultralytics YOLOv8-pose o YOLOv11-pose.** Modelos muy populares de detección y estimación de pose en tiempo real.
* **Opción 2: Google MediaPipe Pose (BlazePose).** Modelo optimizado para dispositivos móviles con 33 puntos clave.
* **Opción 3: RTMPose-m Halpe26 (MMPose / OpenMMLab).** Modelo contemporáneo de estimación de pose de alto rendimiento basado en representación SimCC, con topología Halpe26 (26 puntos clave) y licencia Apache-2.0.

## Decisión tomada

Opción elegida: **Opción 3: RTMPose-m Halpe26 (Apache-2.0)**. Se prohíbe explícitamente el uso de modelos o librerías de Ultralytics YOLO debido a su licencia AGPL-3.0.

### Consecuencias

* **Positivas:**
  * **Licencia limpia y segura:** RTMPose está publicado por el consorcio OpenMMLab bajo licencia **Apache-2.0**, plenamente compatible con FerriteVR y apto para uso libre sin riesgo de contagio viral de licencias.
  * **Topología completa Halpe26:** Incluye 26 puntos clave con desglose detallado de pies (`pie_izq`, `pie_der`, `talon_izq`, `talon_der`, `dedos_pie_izq`, `dedos_pie_der`), lo que permite al solver construir bases ortonormales para la orientación de los pies en VRChat.
  * **Representación SimCC:** Proporciona coordenadas continuas de subpíxel con argmax unidimensional, reduciendo la necesidad de posprocesamiento de mapas de calor pesados y optimizando la latencia a unos pocos milisegundos.
  * El modelo se descarga como archivo estático ONNX verificando su suma de control SHA-256 en los scripts de despliegue.
* **Negativas:**
  * Requiere un paso previo de recorte (*crop*) del sujeto mediante bounding box, el cual FerriteVR resuelve eficazmente a partir de la segmentación por umbral de profundidad.
* **Neutrales:**
  * El modelo se ejecuta desacoplado mediante el crate `ort` en Rust.

## Pros y contras de las opciones

### Opción 1: Ultralytics YOLO (v8 / v11)

* **Bueno:** Detección de persona y pose en una sola pasada.
* **Malo:** Licencia **AGPL-3.0** obligatoria para uso gratuito; prohíbe la distribución bajo licencias permisivas MIT/Apache-2.0 e impone riesgos comerciales inaceptables.
* **Malo:** Topología básica COCO de 17 puntos carece de puntos clave en los dedos y talones de los pies.

### Opción 2: MediaPipe Pose (BlazePose)

* **Bueno:** Licencia Apache-2.0 y 33 puntos clave.
* **Malo:** La exportación a ONNX no es de primera clase y la integración nativa en Rust es compleja y propensa a dependencias con el runtime de C++ de MediaPipe.
* **Malo:** Diseñado primariamente para cámaras RGB de teléfonos móviles; menor precisión en extremidades inferiores en tomas de plano general de habitación.

### Opción 3: RTMPose Halpe26

* **Bueno:** Licencia formal Apache-2.0.
* **Bueno:** 26 puntos clave específicos para cuerpo y pies.
* **Bueno:** Excelente latencia y exactitud en ONNX Runtime.
* **Malo:** Requiere generar una caja delimitadora (*bounding box*) previa.

## Enlaces y referencias

* [Repositorio oficial MMPose / RTMPose](https://github.com/open-mmlab/mmpose/tree/main/projects/rtmpose)
* [Especificación Halpe26 Dataset y Keypoints](https://github.com/Fang-Haoshu/Halpe-FullBody)
