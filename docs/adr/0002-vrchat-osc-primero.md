# ADR-0002: Emisión de trackers mediante protocolo OSC para VRChat en v1

* **Estado:** Aceptado
* **Fecha:** 2026-11-02
* **Decisores:** Equipo de FerriteVR

## Contexto y planteamiento del problema

El objetivo principal de FerriteVR es proporcionar seguimiento de cuerpo completo (*Full Body Tracking* o FBT) para el usuario jugando en VRChat sobre Linux (Nobara 44 / Wayland) mediante un visor Meta Quest 3S conectado por Steam Link a SteamVR (ejecutando VRChat bajo Proton).

Existen dos vías principales para inyectar datos de seguimiento de cuerpo completo en este entorno:
1. Crear un controlador virtual de OpenVR / SteamVR (`driver_ferritevr.so`) que registre dispositivos virtuales de seguimiento (`TrackedDeviceClass_GenericTracker`) directamente en SteamVR.
2. Utilizar el protocolo Open Sound Control (OSC) que VRChat soporta de forma nativa a través de red UDP (`127.0.0.1:9000`), enviando las posiciones y orientaciones de los trackers directamente al juego.

## Criterios de decisión

* Facilidad y robustez de integración sin riesgo de inestabilidad o caídas en SteamVR / Steam Link.
* Compatibilidad con entornos Linux ejecutando VRChat mediante la capa de compatibilidad Valve Proton.
* Simplicidad en la arquitectura de red y capacidad de depuración inmediata.
* Rapidez de ciclo de desarrollo para alcanzar una versión funcional utilizable (v1).

## Opciones consideradas

* **Opción 1: Driver OpenVR nativo para SteamVR.** Implementar una librería dinámica C++/Rust que se registre en SteamVR como controlador de seguimiento.
* **Opción 2: Emisión OSC para VRChat.** Transmitir paquetes UDP según la especificación de VRChat OSC Trackers (`/tracking/trackers/...`).
* **Opción 3: Protocolo SlimeVR / SolarXR.** Emular el protocolo de comunicación de SlimeVR Server para que el software de SlimeVR se encargue de alimentar SteamVR.

## Decisión tomada

Opción elegida: **Opción 2: Emisión OSC para VRChat** como alcance prioritario y exclusivo para la versión v1. El desarrollo de un driver OpenVR queda formalmente pospuesto para versiones futuras.

### Consecuencias

* **Positivas:**
  * No requiere instalar binarios compartidos dentro del árbol de SteamVR ni manipular configuraciones complejas de SteamVR en Linux.
  * Funciona perfectamente con VRChat corriendo bajo Proton sobre Wayland y con Steam Link inalámbrico.
  * Los mensajes OSC se pueden inspeccionar, interceptar y emular trivialmente con herramientas estándar de red UDP.
  * Permite enviar datos consolidados en un único `OscBundle` por fotograma garantizando atomicidad en la actualización del esqueleto en VRChat.
* **Negativas:**
  * El seguimiento solo funciona en VRChat; otros juegos de SteamVR (como Blade & Sorcery o Beat Saber) no reciben estos trackers en v1.
* **Neutrales:**
  * Requiere que el usuario active la opción OSC dentro del menú de acción de VRChat.

## Pros y contras de las opciones

### Opción 1: Driver OpenVR nativo

* **Bueno:** Funciona globalmente para cualquier juego de SteamVR.
* **Malo:** La API de controladores de OpenVR es compleja en C++, sufre de roturas entre versiones de SteamVR y es propensa a causar cuelgues del servidor SteamVR en Linux bajo Proton.
* **Malo:** Mucho mayor esfuerzo de ingeniería y depuración para la v1.

### Opción 2: Emisión OSC para VRChat

* **Bueno:** Protocolo documentado, estable, liviano y transportado sobre UDP estándar.
* **Bueno:** Cero riesgo de desestabilizar SteamVR o la sesión de streaming de Steam Link.
* **Bueno:** Implementación 100% en Rust seguro mediante el crate `rosc`.
* **Malo:** Limitado al ecosistema de VRChat.

### Opción 3: Protocolo SlimeVR

* **Bueno:** Ecosistema consolidado de trackers.
* **Malo:** Introduce una capa de software intermedia innecesaria (SlimeVR Server en Java) y desacopla el control directo del pipeline de FerriteVR.

## Enlaces y referencias

* [VRChat OSC Trackers Documentation](https://creators.vrchat.com/features/osc/trackers/)
* [Crate rosc (Rust OSC)](https://crates.io/crates/rosc)
