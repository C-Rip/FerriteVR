# Referencia del protocolo OSC para Trackers de VRChat

Este documento especifica la integración técnica entre **FerriteVR** y **VRChat** a través de Open Sound Control (OSC) para seguimiento corporal completo (*Full Body Tracking* o FBT).

- **Fuente oficial:** [VRChat OSC Trackers Documentation](https://docs.vrchat.com/docs/osc-trackers)
- **Transporte:** UDP IPv4.
- **Dirección y puerto predeterminado:** `127.0.0.1:9000` (puerto receptor de VRChat).

---

## 1. Direcciones OSC y tipos de datos

VRChat expone direcciones OSC para recibir la posición y rotación de hasta 8 trackers genéricos indexados del `1` al `8`, además de un tracker especial para la cabeza (`head`).

| Dirección OSC | Tipo OSC | Significado | Unidades |
|---|---|---|---|
| `/tracking/trackers/head/position` | `(fff)` | Posición estimada o leída de la cabeza | Metros (`f32`) |
| `/tracking/trackers/head/rotation` | `(fff)` | Orientación de la cabeza | Grados Euler (`f32`) |
| `/tracking/trackers/{1..8}/position` | `(fff)` | Posición espacial del tracker `1` al `8` | Metros (`f32`) |
| `/tracking/trackers/{1..8}/rotation` | `(fff)` | Orientación angular del tracker `1` al `8` | Grados Euler (`f32`) |

---

## 2. Sistema de coordenadas y convención de rotación

- **Sistema de coordenadas espaciales (Unity):**
  - Sistema de mano izquierda (*left-handed*):
    - `+X`: Hacia la **derecha**.
    - `+Y`: Hacia **arriba**.
    - `+Z`: Hacia **adelante** (alejándose del punto de vista).
  - Unidades métricas: Todas las posiciones se expresan en metros con valores flotantes `f32`.
- **Rotación:**
  - Valores de rotación expresados en grados angulares `[-180.0, 180.0]` o `[0.0, 360.0]`.
  - Orden de rotación intrínseca: convención Euler **ZXY** de Unity (`roll`, `pitch`, `yaw`).

---

## 3. Asignación de ranuras (`trackers 1..8`)

> [!IMPORTANT]
> **Comportamiento nativo de VRChat:**
> VRChat **no asigna de forma rígida ni fija qué articulación o hueso del avatar corresponde a cada número de tracker `1..8`**. Durante la calibración dentro de VRChat (pose en T / pose de calibración FBT), el sistema de cinemática inversa (IK) de VRChat empareja dinámicamente cada esfera de tracker disponible con el hueso de su avatar humanoide más cercano.

### Convención interna de FerriteVR
Para mantener determinismo en las pruebas automatizadas, consistencia en la emisión OSC y trazabilidad de telemetría, **FerriteVR define una convención interna estándar** para el mapeo de ranuras:

| Ranura | Articulación interna de FerriteVR | Nombre en VRChat IK | Función corporal |
|---|---|---|---|
| **1** | `cadera` | `hip` | Centro de gravedad / raíz pélvica |
| **2** | `pie_izquierdo` | `left_foot` | Posición y apoyo de pie izquierdo |
| **3** | `pie_derecho` | `right_foot` | Posición y apoyo de pie derecho |
| **4** | `pecho` | `chest` | Orientación y flexión del torso superior |
| **5** | `rodilla_izquierda` | `left_knee` | Inclinación y orientación de rodilla izquierda |
| **6** | `rodilla_derecha` | `right_knee` | Inclinación y orientación de rodilla derecha |
| **7** | `codo_izquierdo` | `left_elbow` | Seguimiento de brazo y codo izquierdo |
| **8** | `codo_derecho` | `right_elbow` | Seguimiento de brazo y codo derecho |

Si un modo operativo utiliza únicamente 3 puntos (FBT básico: cadera y ambos pies), FerriteVR transmite exclusivamente las ranuras `1`, `2` y `3`.

---

## 4. Estructura de paquete atómico (`OSC Bundle`)

Para prevenir artefactos visuales (*tearing* articular) o desincronización temporal entre articulaciones dentro del cálculo cinemático de VRChat:
- **Todos los trackers calculados para un mismo instante temporal se empaquetan en un único `#bundle` OSC atómico.**
- El encabezado del bundle debe contener una marca de tiempo válida (o marca de tiempo inmediata NTP `1`).
- La tasa de emisión recomendada es de 30 a 60 Hz, sincronizada con la disponibilidad de frames del pipeline solver.

---

## 5. Matriz de verificación empírica

La siguiente matriz documenta el estado actual de los supuestos técnicos relativos a la integración con VRChat:

| Aspecto técnico | Especificación esperada | Estado actual | Justificación / Plan de prueba |
|---|---|---|---|
| Puerto UDP receptor | `127.0.0.1:9000` | **VERIFICADO** | Documentado oficialmente en la API OSC de VRChat. |
| Coordenadas de posición | Metros en espacio Unity mano izquierda (`+X` der, `+Y` arriba, `+Z` adelante) | **VERIFICADO** | Documentado oficialmente en la API OSC de VRChat. |
| Direcciones OSC de trackers | `/tracking/trackers/{1..8}/position` y `/rotation` | **VERIFICADO** | Especificación estándar de VRChat. |
| Tracker de cabeza | `/tracking/trackers/head/position` y `/rotation` | **VERIFICADO** | Documentado oficialmente para sincronización de origen HMD. |
| Asignación de ranuras 1..8 | Emparejamiento por proximidad en calibración FBT | **VERIFICADO** | Mecánica documentada por VRChat; FerriteVR aplica convención interna propia. |
| Umbral temporal snap/lerp en cabeza | Umbral de ~300 ms entre salto directo y suavizado en `head/rotation` | **Por verificar empíricamente en WP-2.4** | Se evaluará inyectando retardos artificiales con cliente real de VRChat en ejecución. |
| Timeout de inactividad de tracker | 10 segundos antes de considerar el tracker desconectado | **Por verificar empíricamente en WP-2.4** | Se validará interrumpiendo el flujo de paquetes UDP en sesión FBT activa. |
| Descarte de origen neutro `(0,0,0)` | Rechazo o absorción de coordenadas en el origen geométrico | **Por verificar empíricamente en WP-2.4** | Se enviará explícitamente vector nulo para confirmar si VRChat lo ignora o colapsa el avatar. |
| Sentidos de signo en ejes Euler ZXY | Polaridad de cada rotación respecto a la orientación del avatar | **Por verificar empíricamente en WP-2.4** | Se verificarán giros puros sobre X, Y y Z en avatar de calibración durante la prueba manual. |
