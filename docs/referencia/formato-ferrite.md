# Especificación del formato de grabación binario `.ferrite`

El formato `.ferrite` es un formato binario secuencial diseñado para FerriteVR, optimizado para almacenar sesiones completas de color registrado y profundidad sincronizadas capturadas desde sensores RGB-D (como el Microsoft Kinect v2).

Permite grabar y reproducir flujos continuos a 30 FPS con compresión por fotograma de alta velocidad mediante el algoritmo `zstd`, habilitando pruebas repetibles de visión computacional y cinemática 3D sin requerir el sensor físico conectado.

---

## 1. Visión general de la estructura

Un archivo `.ferrite` consta de:
1. Un **encabezado global fijo de 48 bytes**.
2. Una **secuencia consecutiva de bloques de fotograma**, cada uno con su propia longitud y compresión independiente.

```text
+-------------------------------------------------------------+
|               Encabezado global (48 bytes)                  |
+-------------------------------------------------------------+
| Fotograma 0: Marca de tiempo (8B) | Longitud (4B) | zstd... |
+-------------------------------------------------------------+
| Fotograma 1: Marca de tiempo (8B) | Longitud (4B) | zstd... |
+-------------------------------------------------------------+
| ...                                                         |
+-------------------------------------------------------------+
| Fotograma N: Marca de tiempo (8B) | Longitud (4B) | zstd... |
+-------------------------------------------------------------+
```

---

## 2. Encabezado global (48 bytes)

Todos los valores numéricos de más de un byte se codifican en orden de bytes **Little-Endian** (estándar nativo x86-64 y ARM64).

| Offset (bytes) | Tamaño (bytes) | Tipo | Nombre de campo | Descripción |
|---|---|---|---|---|
| `0..4` | 4 | `[u8; 4]` | `magic` | Identificador ASCII: `b"FERR"` (`0x46, 0x45, 0x52, 0x52`). |
| `4..6` | 2 | `u16` | `version` | Versión del formato. Actualmente `1`. |
| `6..8` | 2 | `u16` | `banderas` | Banderas de configuración (reservadas para uso futuro, actualmente `0`). |
| `8..12` | 4 | `u32` | `ancho` | Ancho de la matriz de profundidad y color en píxeles (ej. `512`). |
| `12..16` | 4 | `u32` | `alto` | Alto de la matriz de profundidad y color en píxeles (ej. `424`). |
| `16..20` | 4 | `f32` | `fx` | Distancia focal horizontal de la cámara de profundidad en píxeles. |
| `20..24` | 4 | `f32` | `fy` | Distancia focal vertical de la cámara de profundidad en píxeles. |
| `24..28` | 4 | `f32` | `cx` | Coordenada horizontal del centro óptico en píxeles. |
| `28..32` | 4 | `f32` | `cy` | Coordenada vertical del centro óptico en píxeles. |
| `32..36` | 4 | `f32` | `fps_nominal` | Frecuencia de muestreo nominal del sensor (ej. `30.0`). |
| `36..48` | 12 | `[u8; 12]` | `reservado` | Relleno alineado a ceros para compatibilidad y futuras expansiones. |

---

## 3. Estructura de cada bloque de fotograma

Cada fotograma en el archivo contiene un prefijo con su metadato temporal y el tamaño del bloque comprimido:

| Offset relativo | Tamaño (bytes) | Tipo | Nombre de campo | Descripción |
|---|---|---|---|---|
| `0..8` | 8 | `u64` | `marca_tiempo_us` | Marca de tiempo en microsegundos transcurridos desde el inicio de la captura o época del sistema. |
| `8..12` | 4 | `u32` | `tamano_comprimido` | Tamaño $L$ en bytes de la carga útil comprimida con `zstd` que sigue inmediatamente. |
| `12..(12 + L)` | $L$ | `[u8; L]` | `datos_comprimidos` | Flujo de bytes comprimido mediante `zstd` (nivel nominal 3). |

El final del archivo se detecta limpiamente cuando el lector encuentra un fin de flujo (`EOF`) al intentar leer el siguiente campo `marca_tiempo_us`.

---

## 4. Disposición interna de la carga descomprimida

Al descomprimir el bloque `datos_comprimidos` con `zstd`, se obtiene un buffer continuo de tamaño exacto:

$$\text{Tamaño descomprimido} = \text{ancho} \times \text{alto} \times 8 \text{ bytes}$$

Para la resolución nativa de Kinect v2 ($512 \times 424 = 217\,088$ píxeles), el tamaño descomprimido por fotograma es exactamente **1 736 704 bytes** (~1.65 MiB).

La carga descomprimida se organiza en dos secciones contiguas:

```text
+-----------------------------------------+-----------------------------------------+
| Mapa de profundidad (ancho * alto * 4B) | Color alineado RGBA (ancho * alto * 4B) |
+-----------------------------------------+-----------------------------------------+
```

1. **Sección de profundidad (`ancho * alto * 4` bytes):**
   - Arreglo continuo de números de punto flotante IEEE 754 de precisión simple (`f32`) en orden little-endian.
   - Cada valor representa la distancia ortogonal al sensor en **milímetros** ($mm$).
   - Valores menores o iguales a `0.0` o `f32::NAN` indican ausencia de retorno o fuera de rango medido.
2. **Sección de color registrado (`ancho * alto * 4` bytes):**
   - Arreglo continuo de píxeles RGBA de 8 bits por canal (`[u8; 4]`: rojo, verde, azul, alfa = 255).
   - Cada píxel $(u, v)$ de color está registrado y alineado espacialmente con el píxel $(u, v)$ correspondiente en el mapa de profundidad.

---

## 5. Compresión y rendimiento en tiempo real

- **Compresión frame a frame:** A diferencia de formatos de vídeo contenedor monolíticos (como MP4 o MKV), comprimir cada fotograma por separado permite la descompresión sin latencia de decodificación inter-cuadro (*intra-frame* puro), búsqueda instantánea (*seeking*) y resistencia ante terminaciones abruptas (ej. desconexión o Ctrl+C).
- **Razón de compresión:** Debido a la coherencia espacial y superficies planas continuas en mapas de profundidad, `zstd` (nivel 3) comprime el frame de 1.65 MiB a un promedio de 80–180 KiB por fotograma, permitiendo tasas de escritura en disco menores a 5 MB/s para 30 FPS.
- **Reutilización de buffers:** La estructura `FrameSensor` y el `LectorFerrite` decodifican los datos en buffers preasignados (`leer_frame_en`), garantizando **cero asignaciones de memoria dinámicas** en el montículo durante el bucle de reproducción en caliente.
