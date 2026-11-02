# Convenciones OSC de VRChat en FerriteVR

Este documento establece las especificaciones y contratos que debe respetar cualquier implementación de telemetría hacia VRChat en `crates/ferrite-osc`.

## 1. Protocolo y transporte

- **Protocolo de transporte:** Paquetes UDP IPv4.
- **Destino predeterminado:** `127.0.0.1:9000` (puerto por defecto del receptor OSC de VRChat).
- **Paquete atómico (`#bundle`):** Cada cuadro temporal (*frame*) completo debe emitirse en un único paquete `OscBundle` con timetag NTP inmediato (`1`). Esto evita artefactos de desincronización (*tearing*) entre articulaciones en el avatar.

## 2. Direcciones OSC y formatos

Para cada tracker activo se transmiten dos mensajes en el mismo bundle:
- **Posición:** `{direccion}/position` con argumentos `(fff)` de tipo `OscType::Float` (metros).
- **Rotación:** `{direccion}/rotation` con argumentos `(fff)` de tipo `OscType::Float` (grados sexagesimales en orden ZXY).

Donde `{direccion}` corresponde a:
- `/tracking/trackers/head` para el tracker de la cabeza.
- `/tracking/trackers/1` a `/tracking/trackers/8` para los trackers de cuerpo numerados.

## 3. Espacio de coordenadas métrico y angular

- **Posición:** Espacio de Unity (sistema siniestro / *left-handed*):
  - `+X`: Derecha del observador.
  - `+Y`: Arriba vertical.
  - `+Z`: Adelante (profundidad).
- **Rotación:** Grados sexagesimales normalizados en el rango `[-180.0, 180.0]`.
  - Orden intrínseco de rotación: **ZXY** (`roll`, `pitch`, `yaw`).
  - La terna de flotantes transmitida representa `[x, y, z]` donde `x` es la rotación alrededor de X, `y` alrededor de Y, y `z` alrededor de Z.

## 4. Convención interna de ranuras de FerriteVR

VRChat empareja trackers numerados por cercanía espacial durante la pose en T. Para garantizar repetibilidad determinista, FerriteVR utiliza la siguiente asignación fija:

| Ranura | Articulación FerriteVR | Nombre en VRChat |
|---|---|---|
| `head` | `Articulacion::Cabeza` | Cabeza |
| `1` | `Articulacion::Cadera` | Cadera (`hip`) |
| `2` | `Articulacion::PieIzq` | Pie Izquierdo (`left_foot`) |
| `3` | `Articulacion::PieDer` | Pie Derecho (`right_foot`) |
| `4` | `Articulacion::Pecho` | Pecho (`chest`) |
| `5` | `Articulacion::RodillaIzq` | Rodilla Izquierda (`left_knee`) |
| `6` | `Articulacion::RodillaDer` | Rodilla Derecha (`right_knee`) |
| `7` | `Articulacion::CodoIzq` | Codo Izquierdo (`left_elbow`) |
| `8` | `Articulacion::CodoDer` | Codo Derecho (`right_elbow`) |

Las articulaciones con confianza estadística menor o igual a `0.0` no deben incluirse en el frame OSC emitido.
