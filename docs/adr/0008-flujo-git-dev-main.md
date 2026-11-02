# ADR-0008: Flujo de trabajo en Git con rama dev de desarrollo y main para versiones estables

* **Estado:** Aceptado
* **Fecha:** 2026-11-02
* **Decisores:** Equipo de FerriteVR

## Contexto y planteamiento del problema

FerriteVR se desarrolla en un entorno iterativo donde interactúan ingenieros y agentes autónomos de inteligencia artificial. Es crucial contar con una estrategia de ramificación (*branching strategy*) y flujo de trabajo en Git que sea:
1. Extremadamente clara y libre de fricciones burocráticas innecesarias.
2. Segura para evitar romper la estabilidad de la rama pública o principal del proyecto.
3. Determinista para facilitar la trazabilidad de paquetes de trabajo (WPs) y evitar conflictos en la integración continua.

Modelos complejos como GitFlow tradicional introducen un exceso de ramas intermedias (ramas de características, ramas de soporte, ramas de hotfix) que generan sobrecarga de sincronización para un equipo compacto o asistido por IA. Por otro lado, un modelo de rama única (*trunk-based development* directo a `main`) expone a los usuarios a clonar estados inestables o a medio implementar durante el ciclo activo de desarrollo.

## Criterios de decisión

* Simplicidad operativa para humanos y subagentes de desarrollo.
* Garantía de que la rama `main` siempre represente un estado verificado, estable y publicado con versión formal.
* Preservación estricta de la integridad histórica (prohibición de `push --force`).
* Facilidad para revertir o auditar cambios a nivel de paquetes de trabajo.

## Opciones consideradas

* **Opción 1: Desarrollo directo en `main` (Trunk-based directo).** Todo commit se realiza directamente sobre la rama principal.
* **Opción 2: GitFlow tradicional.** Ramas `feature/*`, `release/*`, `hotfix/*`, `develop` y `master`.
* **Opción 3: Flujo dual continuo (`dev` y `main`).** Rama `dev` como tronco de desarrollo continuo donde se integran commits pequeños y atómicos; rama `main` reservada exclusivamente para versiones estables etiquetadas (`vX.Y.Z`).

## Decisión tomada

Opción elegida: **Opción 3: Flujo dual continuo (`dev` y `main`)**.

Se establecen las siguientes directrices operativas:

1. **Rama de desarrollo activo (`dev`):**
   - Es la rama de trabajo por defecto para todos los subagentes e ingenieros.
   - Se realizan commits pequeños, atómicos y frecuentes directamente sobre `dev`.
   - Cada commit debe compilar y cumplir los criterios de verificación de su paquete de trabajo.
2. **Rama principal estable (`main`):**
   - Queda estrictamente prohibido realizar commits directos sobre `main`.
   - La integración de `dev` a `main` se realiza exclusivamente cuando se alcanza un hito de versión estable (por ejemplo `v0.1.0`), validado previamente por el usuario con pruebas de hardware en VRChat.
   - Cada fusión a `main` se acompaña inmediatamente de una etiqueta Git anotada (`git tag -a vX.Y.Z -m "..."`).
3. **Inmutabilidad del historial:**
   - Queda terminantemente prohibido el uso de `git push --force` o la reescritura de historial en ramas compartidas.
   - Las correcciones se aplican mediante nuevos commits directos o reversiones explícitas (`git revert`).

### Consecuencias

* **Positivas:**
  * Cualquier usuario externo que clone el repositorio desde `main` tiene la certeza de obtener una versión funcional y probada.
  * Los agentes y desarrolladores pueden iterar ágilmente en `dev` sin el overhead de crear y destruir ramas efímeras para cambios pequeños.
  * Trazabilidad limpia y lineal en el historial de commits.
* **Negativas:**
  * Si un subagente comete un error en `dev`, debe corregirlo con otro commit en lugar de reescribir con rebase interactivo.
* **Neutrales:**
  * La fusión a `main` es un acto deliberado y controlado por el mantenedor humano del proyecto.

## Pros y contras de las opciones

### Opción 1: Directo en main

* **Bueno:** Máxima simplicidad mecánica.
* **Malo:** `main` puede quedar rota durante horas mientras se completa una fase intermedia de un paquete de trabajo.
* **Malo:** No hay distinción clara entre código en desarrollo y versión estable para el público.

### Opción 2: GitFlow tradicional

* **Bueno:** Altamente formalizado para organizaciones grandes.
* **Malo:** Demasiadas ramas efímeras, conflictos de fusión frecuentes y complejidad innecesaria para el alcance de FerriteVR.

### Opción 3: Flujo dual continuo (dev + main)

* **Bueno:** Equilibrio óptimo entre agilidad y seguridad.
* **Bueno:** Reglas claras para agentes: trabajar siempre en `dev`, nunca tocar `main`.
* **Malo:** Requiere recordar cambiar de rama o verificar la rama activa antes de iniciar tareas.

## Enlaces y referencias

* [A successful Git branching model (Vincent Driessen)](https://nvie.com/posts/a-successful-git-branching-model/)
* [Trunk Based Development](https://trunkbaseddevelopment.com/)
