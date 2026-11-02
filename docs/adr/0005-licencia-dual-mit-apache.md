# ADR-0005: Licencia dual MIT y Apache-2.0

* **Estado:** Aceptado
* **Fecha:** 2026-11-02
* **Decisores:** Equipo de FerriteVR

## Contexto y planteamiento del problema

FerriteVR es un proyecto de código abierto desarrollado en Rust para la comunidad de realidad virtual y seguimiento de movimiento. Es indispensable definir el régimen de licenciamiento bajo el cual se distribuye el código fuente, los paquetes y las contribuciones al proyecto.

El licenciamiento elegido debe satisfacer los siguientes requisitos:
1. Máxima adopción y facilidad de uso tanto para usuarios particulares como para integradores de software libre y comercial.
2. Protección jurídica explícita contra agresiones o litigios de patentes para los usuarios y desarrolladores del proyecto.
3. Coherencia con las convenciones estándar del ecosistema de Rust.

## Criterios de decisión

* Alineación con los estándares y normas de facto de la comunidad de Rust (crates.io).
* Concesión explícita de licencias de patentes y defensa contra litigios.
* Compatibilidad amplia con proyectos GPL y no GPL.
* Simplicidad en la atribución y redistribución.

## Opciones consideradas

* **Opción 1: Licencia MIT exclusivamente.** Máxima sencillez y permisividad.
* **Opción 2: Licencia Apache-2.0 exclusivamente.** Cobertura de patentes explícita, pero con ciertas dificultades históricas de compatibilidad estricta con versiones antiguas de licencias como GPLv2.
* **Opción 3: Licencia GNU GPLv3 o AGPLv3.** Licencia con efecto recíproco o viral estricto (*copyleft*).
* **Opción 4: Licencia dual MIT OR Apache-2.0.** Permite al receptor del software optar por cualquiera de las dos licencias.

## Decisión tomada

Opción elegida: **Opción 4: Licencia dual `MIT OR Apache-2.0`**.

El proyecto se distribuye con los archivos `LICENSE-MIT` y `LICENSE-APACHE` en la raíz del repositorio, declarando en los metadatos de Cargo la propiedad:

```toml
[workspace.package]
license = "MIT OR Apache-2.0"
```

El titular de los derechos de autor legalmente reconocido es **"Los contribuyentes de FerriteVR"**, año 2026.

### Consecuencias

* **Positivas:**
  * Sigue la convención estándar del lenguaje Rust y del compilador `rustc`.
  * Quienes requieran máxima compatibilidad o simplicidad pueden adoptar el código bajo los términos de la licencia MIT.
  * Quienes requieran protección legal formal frente a derechos de patente sobre software pueden acogerse a la cláusula de patente explícita de Apache-2.0 (sección 3).
  * Todas las dependencias de terceros añadidas al workspace son auditadas mediante `cargo-deny` para asegurar que respetan licencias compatibles permisivas.
* **Negativas:**
  * Requiere mantener y distribuir dos archivos de licencia en la raíz del proyecto.
* **Neutrales:**
  * Todas las contribuciones al proyecto se aceptan bajo este mismo régimen de licencia dual.

## Pros y contras de las opciones

### Opción 1: MIT exclusiva

* **Bueno:** Texto sumamente breve y conocido universalmente.
* **Malo:** No contiene cláusula explícita de terminación ni de concesión de licencias de patentes.

### Opción 2: Apache-2.0 exclusiva

* **Bueno:** Protección exhaustiva de propiedad intelectual y patentes.
* **Malo:** Incompatible con ciertos proyectos de software libre licenciados estrictamente bajo GPLv2 puro sin cláusula "o posterior".

### Opción 3: Licencias Copyleft (GPL / AGPL)

* **Bueno:** Garantiza que los derivados mantengan el código abierto.
* **Malo:** Inhibe la adopción por parte de desarrolladores que deseen integrar librerías en herramientas con licencias distintas; incompatible con los modelos permisivos estándar de crates.io.

### Opción 4: Dual MIT OR Apache-2.0

* **Bueno:** Proporciona los beneficios de ambas licencias sin sus desventajas.
* **Bueno:** Máxima interoperabilidad dentro del ecosistema de crates de Rust.
* **Malo:** Requiere comprensión del mecanismo de doble licencia.

## Enlaces y referencias

* [Rust API Guidelines on Licensing](https://rust-lang.github.io/api-guidelines/about.html)
* [Apache 2.0 License Text](https://www.apache.org/licenses/LICENSE-2.0)
* [MIT License Text](https://opensource.org/licenses/MIT)
