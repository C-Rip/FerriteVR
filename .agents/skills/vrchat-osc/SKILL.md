---
name: vrchat-osc
description: Detalla las especificaciones del protocolo OSC de VRChat para seguimiento de trackers corporales, espacios de coordenadas, formato de paquetes y convenciones internas de asignacion de FerriteVR. Usar al trabajar en ferrite-osc o en la integracion de telemetria hacia VRChat.
---

# Integración OSC de Trackers para VRChat

Esta skill proporciona las especificaciones necesarias para interactuar con la API OSC de seguimiento corporal de VRChat, cubriendo transporte, formato de datos, espacio de coordenadas y convenciones de ranura.

Para detalles técnicos y tablas de parámetros, consultar el documento complementario [`references/osc-trackers.md`](references/osc-trackers.md) y la especificación de referencia [`docs/referencia/vrchat-osc-trackers.md`](../../../docs/referencia/vrchat-osc-trackers.md).

---

## 1. Principios de integración

1. **Protocolo y puerto:**
   - Protocolo UDP IPv4 enviado a `127.0.0.1:9000` (puerto por defecto en VRChat).
2. **Bundle atómico obligatorio:**
   - Cada frame de seguimiento debe transmitirse en un único paquete `#bundle` de OSC que contenga los mensajes de posición y rotación de todos los trackers activos.
   - Prohibido enviar mensajes individuales desincronizados para evitar el *tearing* visual de extremidades.
3. **Espacio de coordenadas métrico Unity:**
   - Coordenadas de posición en metros (`f32`), sistema siniestro (*left-handed*): `+X` derecha, `+Y` arriba, `+Z` adelante.
   - Rotaciones en grados Euler con orden intrínseco **ZXY** (`roll`, `pitch`, `yaw`).
4. **Mapeo de ranuras 1..8:**
   - VRChat **no impone** qué articulación corresponde a cada número (las empareja por proximidad durante la pose en T).
   - FerriteVR utiliza la **convención interna fija**: `1` = Cadera, `2` = Pie Izquierdo, `3` = Pie Derecho, `4` = Pecho, `5` = Rodilla Izquierda, `6` = Rodilla Derecha, `7` = Codo Izquierdo, `8` = Codo Derecho.
