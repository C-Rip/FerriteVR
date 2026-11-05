# Especificación del modelo de estimación de pose RTMPose-m (Halpe26)

FerriteVR utiliza **RTMPose-m (Halpe26)** como red neuronal de visión artificial para la estimación de pose humana en dos dimensiones a partir de imágenes RGB.

Esta arquitectura fue elegida debido a su velocidad en tiempo real en CPU y GPU, alta precisión en extremidades inferiores y pies, y su licencia de código abierto permisiva.

---

## 1. Identificación y origen

| Parámetro | Detalle |
|---|---|
| **Modelo** | RTMPose-m (`rtmpose-m_8xb512-700e_body8-halpe26-256x192`) |
| **Autor y origen** | OpenMMLab / MMPose ([Repositorio GitHub](https://github.com/open-mmlab/mmpose)) |
| **Licencia** | **Apache-2.0** (compatible con la licencia dual MIT / Apache-2.0 de FerriteVR) |
| **Formato de archivo** | ONNX (Open Neural Network Exchange), Opset 11 |
| **Ubicación en repo** | `modelos/rtmpose-m-halpe26.onnx` (ignorado en Git por tamaño) |
| **Suma SHA-256** | `26f3a19e61304a600dfb82d1001d41d24343b89fc70a33ffc84657e0b0bf2ecf` |
| **Tamaño en disco** | ~55.6 MB (55 685 444 bytes) |

---

## 2. Parámetros de entrada

El modelo acepta un único tensor de entrada con imágenes normalizadas de personas recortadas (o la imagen completa reescalada manteniendo la relación de aspecto):

| Propiedad | Especificación |
|---|---|
| **Nombre de entrada** | `"input"` |
| **Dimensiones** | `[batch_size, 3, 256, 192]` (orden NCHW) |
| **Resolución** | Alto: $256$ píxeles, Ancho: $192$ píxeles |
| **Espacio de color** | RGB con valores en rango flotante $[0.0, 255.0]$ |
| **Media ($\mu$)** | `[123.675, 116.280, 103.530]` (canales R, G, B) |
| **Desviación ($\sigma$)** | `[58.395, 57.120, 57.375]` (canales R, G, B) |

### Fórmula de normalización de píxeles:

$$x_{\text{normalizado}} = \frac{x_{\text{RGB}} - \mu}{\sigma}$$

---

## 3. Representación de salida (SimCC)

A diferencia de los detectores de pose tradicionales que utilizan mapas de calor bidimensionales densos ($H \times W$), RTMPose implementa la representación **SimCC** (*Simple Coordinate Classification*).

SimCC reformula la regresión de coordenadas como dos tareas independientes de clasificación 1D (un vector horizontal $X$ y un vector vertical $Y$), reduciendo drásticamente la carga computacional y acelerando la inferencia en tiempo real.

| Salida | Nombre | Dimensiones | Descripción |
|---|---|---|---|
| **Coordenada X** | `"simcc_x"` | `[batch_size, 26, 384]` | Probabilidades de posición horizontal ($192 \times 2.0$) |
| **Coordenada Y** | `"simcc_y"` | `[batch_size, 26, 512]` | Probabilidades de posición vertical ($256 \times 2.0$) |

El factor de escala (*split ratio*) es de **2.0** en ambas dimensiones, permitiendo una resolución subpíxel de medio píxel.

---

## 4. Conjunto de articulaciones (Halpe26)

El conjunto de 26 puntos clave de Halpe incluye el seguimiento detallado de cabeza, torso y pies (incluyendo talón y dedos), lo cual resulta crítico para la cinemática de pie y tobillo en VRChat:

| Índice | Nombre en Halpe26 | Correspondencia en FerriteVR / VRChat |
|---|---|---|
| **0** | Nariz | Orientación facial |
| **1** | Ojo izquierdo | Seguimiento de mirada |
| **2** | Ojo derecho | Seguimiento de mirada |
| **3** | Oreja izquierda | Estimación de rotación de cabeza |
| **4** | Oreja derecha | Estimación de rotación de cabeza |
| **5** | Hombro izquierdo | Base clavicular izquierda |
| **6** | Hombro derecho | Base clavicular derecha |
| **7** | Codo izquierdo | Tracker OSC Ranura `7` |
| **8** | Codo derecho | Tracker OSC Ranura `8` |
| **9** | Muñeca izquierda | Posición de antebrazo |
| **10** | Muñeca derecha | Posición de antebrazo |
| **11** | Cadera izquierda | Pelvis izquierda |
| **12** | Cadera derecha | Pelvis derecha |
| **13** | Rodilla izquierda | Tracker OSC Ranura `5` |
| **14** | Rodilla derecha | Tracker OSC Ranura `6` |
| **15** | Tobillo izquierdo | Cinemática de pierna izquierda |
| **16** | Tobillo derecho | Cinemática de pierna derecha |
| **17** | Cabeza | Tracker OSC Ranura `head` |
| **18** | Cuello | Base de rotación cervical |
| **19** | Cadera central (Pelvis) | Tracker OSC Ranura `1` (Cadera / Raíz) |
| **20** | Dedo gordo pie izquierdo | Tracker OSC Ranura `2` (Pie izquierdo) |
| **21** | Dedo pequeño pie izquierdo | Tracker OSC Ranura `2` (Pie izquierdo) |
| **22** | Talón pie izquierdo | Tracker OSC Ranura `2` (Pie izquierdo) |
| **23** | Dedo gordo pie derecho | Tracker OSC Ranura `3` (Pie derecho) |
| **24** | Dedo pequeño pie derecho | Tracker OSC Ranura `3` (Pie derecho) |
| **25** | Talón pie derecho | Tracker OSC Ranura `3` (Pie derecho) |

> [!NOTE]
> El punto del **Pecho (Tracker Ranura 4)** se calcula en el solver cinemático como el punto medio entre los hombros (5 y 6) y el cuello (18).

---

## 5. Algoritmo de decodificación SimCC a píxeles 2D

Para obtener la posición $(u, v)$ en píxeles de la imagen de entrada (256×192) y su confianza $c$ para la articulación $k$:

1. Encontrar el índice de valor máximo en cada vector:
   $$i_x = \arg\max_{i} (\text{simcc\_x}[k, i])$$
   $$i_y = \arg\max_{j} (\text{simcc\_y}[k, j])$$
2. Dividir por el factor de escala `simcc_split_ratio` ($2.0$):
   $$u_k = \frac{i_x}{2.0}, \quad v_k = \frac{i_y}{2.0}$$
3. La confianza $c_k$ se obtiene a partir del valor del pico (o de la probabilidad softmax normalizada):
   $$c_k = \min(\text{simcc\_x}[k, i_x], \text{simcc\_y}[k, i_y])$$

---

## 6. Descarga y validación automatizada

El script canónico [`scripts/descargar-modelos.sh`](file:///home/crivera/proyectos/FerriteVR/scripts/descargar-modelos.sh) descarga el archivo comprimido oficial, extrae el archivo ONNX y valida su suma SHA-256. Es completamente idempotente:

```bash
./scripts/descargar-modelos.sh
```
