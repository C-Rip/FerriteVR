---
name: ferrite-verificar
description: Explica como ejecutar el script canonico de verificacion de calidad scripts/verificar.sh, como interpretar fallos de formato, linter, tests, documentacion, licencias y ortografia, y como subsanarlos. Usar antes de preparar commits o al validar el DoD.
---

# Verificación de Calidad en FerriteVR

Esta skill describe el uso del script canónico `./scripts/verificar.sh` y el procedimiento estándar para diagnosticar y corregir anomalías en las etapas de calidad del proyecto.

---

## 1. Ejecución del script

El comando principal debe invocarse desde la raíz del repositorio:

```bash
./scripts/verificar.sh
```

El script se detiene inmediatamente ante el primer fallo (`set -euo pipefail`) y evalúa:
1. Formato de código (`cargo fmt`)
2. Linter y advertencias del compilador (`cargo clippy`)
3. Pruebas automatizadas (`cargo test`)
4. Documentación pública estricta (`RUSTDOCFLAGS="-D warnings" cargo doc`)
5. Cumplimiento de licencias y vulnerabilidades (`cargo deny check`)
6. Corrección ortográfica (`typos`)

---

## 2. Diagnóstico y corrección de fallos típicos

### 1. Formato de código (`cargo fmt`)
- **Síntoma:** `Diff in <archivo.rs> at line ...` o aviso de que el formato no coincide con `rustfmt.toml`.
- **Causa:** Indentación incorrecta, líneas con longitud superior a 100 caracteres o espaciado no canónico.
- **Corrección:**
  ```bash
  cargo fmt --all
  ```
  Revisa las líneas largas manuales si `rustfmt` no puede dividirlas automáticamente (por ejemplo, cadenas de texto o URLs largas).

### 2. Linter Clippy (`cargo clippy`)
- **Síntoma:** Error de compilación con bandera `-D warnings` (ej. advertencias de clonación innecesaria, variables sin usar o tipos no idiomáticos).
- **Causa común en FerriteVR:**
  - Uso de caracteres no ASCII en identificadores (`non_ascii_idents`).
  - Creación de asignaciones innecesarias dentro de bucles frecuentes.
  - Implementación manual de traits estándar en vez de `#[derive(...)]`.
- **Corrección:**
  Aplica las recomendaciones dadas por Clippy en el mensaje de error o ejecuta `cargo clippy --fix --workspace --all-targets` con precaución, revisando el diff resultante.

### 3. Documentación estricta (`cargo doc`)
- **Síntoma:** `error: missing documentation for a struct` o `broken intra-doc links`.
- **Causa:** Elementos públicos (`pub struct`, `pub fn`, `pub mod`) sin comentarios `///` o enlaces rotos a otros tipos en el rustdoc.
- **Corrección:**
  Añade comentarios rustdoc claros en español describiendo el propósito del elemento, parámetros y casos de error.

### 4. Auditoría de dependencias y licencias (`cargo deny`)
- **Síntoma:** `license <NOMBRE> is not explicitly allowed` o `advisory found for crate`.
- **Causa:** Inclusión de una dependencia con licencia no permitida (como GPL/AGPL) o con advertencias de seguridad en RustSec Advisory Database.
- **Corrección:**
  - Sustituye la dependencia por una alternativa permisiva (MIT, Apache-2.0, BSD).
  - Si la licencia es permisiva pero no figura en `deny.toml`, evalúa si cumple la política y regístrala en la sección `licenses.allow`.

### 5. Ortografía de código y documentación (`typos`)
- **Síntoma:** `error: '<palabra>' should be '<correccion>'`.
- **Causa:** `typos` evalúa ortografía asumiendo inglés por defecto; palabras válidas en español pueden considerarse errores ortográficos en inglés.
- **Corrección:**
  Si la palabra es correcta en español o es un término técnico propio del dominio (ej. `cadera`, `rotacion`, `posicion`), agrégala al diccionario de excepciones en `_typos.toml` bajo `[default.extend-words]`:
  ```toml
  [default.extend-words]
  cadera = "cadera"
  ```
