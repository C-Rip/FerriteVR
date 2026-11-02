# Resumen técnico de integración OSC Trackers de VRChat

Este documento técnico de referencia rápida resume las especificaciones del protocolo OSC de VRChat empleadas por FerriteVR para el envío de telemetría de seguimiento corporal completo.

- **Fuente:** [VRChat OSC Trackers Docs](https://docs.vrchat.com/docs/osc-trackers)
- **Transporte y puerto:** UDP en `127.0.0.1:9000`.

---

## 1. Direcciones y tipos de argumentos

| Dirección OSC | Argumentos | Tipos | Descripción |
|---|---|---|---|
| `/tracking/trackers/head/position` | `x y z` | `(fff)` | Posición de cabeza en metros (`f32`) |
| `/tracking/trackers/head/rotation` | `x y z` | `(fff)` | Rotación de cabeza en grados Euler (`f32`) |
| `/tracking/trackers/{1..8}/position` | `x y z` | `(fff)` | Posición del tracker `1..8` en metros (`f32`) |
| `/tracking/trackers/{1..8}/rotation` | `x y z` | `(fff)` | Rotación del tracker `1..8` en grados Euler (`f32`) |

---

## 2. Sistema de coordenadas espaciales y angulares

- **Espacio métrico (Unity - mano izquierda):**
  - Eje `+X`: Hacia la derecha del observador.
  - Eje `+Y`: Hacia arriba vertical.
  - Eje `+Z`: Hacia adelante (profundidad).
- **Rotación angular:**
  - Valores expresados en grados sexagesimales `[-180.0, 180.0]`.
  - Orden intrínseco de rotación Euler de Unity: **ZXY** (`roll`, `pitch`, `yaw`).

---

## 3. Asignación de ranuras `1..8` y emparejamiento

- **Comportamiento nativo de VRChat:**
  VRChat **no asume una articulación fija** por ranura numérica `1..8`. Durante la calibración FBT en el juego, VRChat asocia cada tracker recibido al hueso de la armadura más próximo.
- **Convención interna estándar en FerriteVR:**
  Para emisión coherente, FerriteVR emplea la siguiente convención interna:
  - `1`: Cadera (`hip`)
  - `2`: Pie izquierdo (`left_foot`)
  - `3`: Pie derecho (`right_foot`)
  - `4`: Pecho (`chest`)
  - `5`: Rodilla izquierda (`left_knee`)
  - `6`: Rodilla derecha (`right_knee`)
  - `7`: Codo izquierdo (`left_elbow`)
  - `8`: Codo derecho (`right_elbow`)

---

## 4. Paquetes atómicos (`OSC Bundle`)

- Todos los mensajes de posición y rotación correspondientes a un mismo frame temporal se envían en un único paquete atómico `#bundle` con timestamp NTP inmediato (`1`).
- Esto evita el desfase articular en el motor de cinemática inversa de VRChat.

---

## 5. Estado de supuestos técnicos y calibración

| Aspecto | Estado | Notas |
|---|---|---|
| Puerto UDP 9000 y formato `(fff)` | **VERIFICADO** | Especificación oficial de VRChat. |
| Coordenadas Unity (mano izquierda) | **VERIFICADO** | Especificación oficial de VRChat. |
| Umbral snap/lerp en `head` (~300 ms) | **Por verificar empíricamente en WP-2.4** | Validación con cliente VRChat en vivo. |
| Timeout de desconexión (10 s) | **Por verificar empíricamente en WP-2.4** | Validación con cliente VRChat en vivo. |
| Comportamiento ante vector nulo `(0,0,0)` | **Por verificar empíricamente en WP-2.4** | Validación con cliente VRChat en vivo. |
| Polaridad exacta en ejes Euler ZXY | **Por verificar empíricamente en WP-2.4** | Validación con cliente VRChat en vivo. |
