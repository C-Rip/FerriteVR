# Primera prueba de seguimiento corporal en VRChat con el modo simulado

Este tutorial te guiará paso a paso para validar la comunicación OSC y el correcto funcionamiento del sistema de seguimiento corporal completo (*Full Body Tracking* o FBT) entre **FerriteVR** y **VRChat** en **Nobara Linux 44**, utilizando un visor **Meta Quest 3S** conectado mediante **Steam Link** (o Virtual Desktop), sin necesidad de contar con el sensor Kinect v2 conectado.

---

## 1. Requisitos previos

Antes de comenzar, asegúrate de disponer de:

1. **Nobara Linux 44** con Steam, SteamVR y VRChat actualizados y operativos.
2. Visor **Meta Quest 3S** conectado al PC mediante **Steam Link** o **Virtual Desktop** en la misma red local (preferiblemente enlace Wi-Fi 6 a 5 GHz o cable).
3. Repositorio de FerriteVR clonado y dependencias del sistema compiladas (`cargo build --workspace`).
4. Un avatar compatible con FBT en tu cuenta de VRChat (la mayoría de los avatares humanoides modernos admiten seguimiento de 6 u 8 puntos).

---

## 2. Paso a paso

### Paso 1: Habilitar el protocolo OSC en VRChat

VRChat requiere autorizar explícitamente la recepción de paquetes OSC en cada cliente:

1. Ponte el visor y entra en VRChat en modo SteamVR.
2. Abre el **Action Menu** (menú circular en la mano mediante pulsación prolongada de la tecla de menú o joystick).
3. Navega a **Options** $\rightarrow$ **OSC**.
4. Asegúrate de que la opción **OSC** esté en estado **Enabled** (icono activado en azul).
5. Si habías usado previamente otros sistemas de seguimiento (como SlimeVR o Standable), es recomendable presionar **Reset OSC Config** para limpiar las definiciones cacheadas.

> [!NOTE]
> Por defecto, VRChat abre un puerto receptor UDP en `127.0.0.1:9000`.

---

### Paso 2: Lanzar el emisor simulado en la terminal

En una terminal en tu equipo host, ejecuta el subcomando `fake` en modo optimizado (*release*):

```bash
cargo run --release -p ferrite -- fake --destino 127.0.0.1:9000 --hz 30
```

Verás una salida de registro similar a la siguiente:

```text
2026-11-02T02:00:00.000Z  INFO ferrite: Iniciando emisor simulado (modo fake) hacia 127.0.0.1:9000 a 30 Hz...
2026-11-02T02:00:00.001Z  INFO ferrite: Transmitiendo telemetria OSC. Presione Ctrl+C para detener.
2026-11-02T02:00:05.002Z  INFO ferrite: Cuadros transmitidos: 150 (tiempo transcurrido: 5.0 s)
```

El emisor calculará y transmitirá de forma determinista y sin acumular deriva temporal un esqueleto 3D completo que emula una marcha armónica en el sitio.

---

### Paso 3: Calibrar los trackers dentro de VRChat

1. Dirígete a cualquier sala que disponga de un espejo grande (por ejemplo, tu *Home World* privado).
2. Al activar el modo `fake`, verás flotando alrededor de tu avatar esferas blancas o indicadores visuales correspondientes a los trackers activos:
   - Tracker `head` (cabeza)
   - Tracker `1` (cadera)
   - Tracker `2` y `3` (pies izquierdo y derecho)
   - Tracker `4` (pecho)
   - Tracker `5` y `6` (rodillas)
   - Tracker `7` y `8` (codos)
3. Abre el **Action Menu** y selecciona la opción **Calibrate FBT** (o pulsa la opción de calibración en el menú principal).
4. El avatar entrará en pose de calibración (pose en T o pose en I).
5. Colócate físicamente de pie en el centro de tu área de juego, alinea tu cabeza con el visor y presiona simultáneamente **ambos gatillos** de los controladores de la Quest 3S.

---

### Paso 4: Verificar la cinemática en el espejo

Observa tu reflejo en el espejo virtual:

- Las piernas del avatar deben levantarse alternadamente imitando una caminata fluida en el sitio.
- La cadera mostrará un ligero balanceo lateral armónico al transferir peso entre apoyos.
- La base de los pies respetará una cota vertical estricta ($Y \ge 0.05\text{ m}$), garantizando que las extremidades inferiores no se hundan en el suelo del mundo virtual.
- Los codos oscilarán suavemente a los costados en contraposición armónica a las piernas.

---

### Paso 5: Finalizar la prueba

Cuando desees concluir la sesión de prueba:

1. Regresa a la terminal donde se ejecuta FerriteVR.
2. Presiona `Ctrl+C`.
3. El programa interceptará limpiamente la señal y mostrará el balance total de cuadros emitidos:

```text
2026-11-02T02:01:30.000Z  INFO ferrite: Emision detenida. Total de cuadros enviados: 2700 en 90.00 s.
```

---

## 3. Diagnóstico y solución de problemas

| Síntoma | Causa probable | Solución |
|---|---|---|
| No aparecen esferas de trackers en VRChat | El puerto UDP 9000 está bloqueado o OSC está desactivado | Revisa que OSC esté en *Enabled* en el Action Menu de VRChat. Comprueba con `ss -uln \| grep 9000` si VRChat está escuchando. |
| El avatar vuela o se deforma al calibrar | Altura o proporciones de avatar no configuradas | En VRChat, ajusta la altura real de tu usuario en *Settings* $\rightarrow$ *Avatar* $\rightarrow$ *User Real Height*. |
| Tearing o saltos bruscos en las extremidades | Congestión de red o caída de FPS | Asegúrate de transmitir a 30 o 60 Hz estables (`--hz 30`). Evita ejecutar otras aplicaciones intensivas en el bucle local. |
| Conflicto de puertos UDP | Otra aplicación (e.g. SlimeVR Server) está usando el puerto 9000 | Cierra cualquier otro emisor o servidor de FBT antes de iniciar FerriteVR. |

---

## 4. Siguientes pasos

Una vez validada la cadena de transmisión OSC en modo simulado:
- Proceder con la Fase 3 (**Sensor Kinect**) para conectar el sensor físico y verificar el streaming de profundidad y color en tiempo real.
