# Reglas de FFI para ferrite-freenect2-sys

Este documento establece las directrices obligatorias para la capa de interoperabilidad FFI con `libfreenect2` en `crates/ferrite-freenect2-sys`.

## 1. Alcance y permisos de `unsafe`

- Este crate es el **único crate de todo el workspace** de FerriteVR con `#![allow(unsafe_code)]`.
- Todo bloque `unsafe` debe contar de forma obligatoria con un comentario explicativo formal con el prefijo:
  ```rust
  // SEGURIDAD: <justificación rigurosa del invariante de seguridad verificado>
  ```
- El código inseguro debe mantenerse al mínimo estricto, encapsulado en tipos seguros de Rust (RAII) que garanticen la liberación de recursos (`Drop`).

## 2. Frontera C++ y captura de excepciones

- Ninguna excepción de C++ (`std::exception` o no estándar) debe atravesar jamás la frontera FFI hacia Rust, pues provocaría comportamiento indefinido o aborto inmediato del proceso (*panic* no capturable).
- Todas las llamadas a métodos de `libfreenect2` dentro del *shim* C++ (`shim.cc`) deben envolverse en bloques `try/catch` que capturen excepciones estándar y devuelvan códigos de error o errores tipados hacia el puente seguro `cxx`.

## 3. Rendimiento y cero asignaciones en caliente

- Prohibido reservar o liberar memoria (`malloc`, `new`, `Vec::new`) en el ciclo de captura por fotograma.
- Los fotogramas intermedios de registro (`undistorted` y `registered`) se preasignan una sola vez durante la inicialización del objeto `DispositivoKinect`.
- La copia hacia los búferes de Rust se realiza directamente sobre rodajas de memoria (`cxx::Slice` o punteros mutables válidos) previamente asignadas por el consumidor (`FrameSensor`).

## 4. Compilación condicional (`feature = "kinect"`)

- Todo el código que enlace con `libfreenect2` o requiera compiladores C++ para FFI debe compilarse exclusivamente bajo la característica `kinect`.
- Si la característica `kinect` no está activa, el crate debe compilar de forma vacía (o con stubs inertes) para permitir que la integración continua (CI) y las verificaciones locales sin dependencias de hardware pasen sin fallos.
