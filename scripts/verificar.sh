#!/usr/bin/env bash
# scripts/verificar.sh - Suite canónica de verificación de calidad de FerriteVR
set -euo pipefail

# Asegurar que ejecutables de ~/.cargo/bin estén en el PATH si existen
export PATH="${HOME}/.cargo/bin:${PATH}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

echo "=== Verificando calidad en FerriteVR ==="

# 1. Verificación del Workspace de Cargo (si existe Cargo.toml)
if [[ -f "${REPO_ROOT}/Cargo.toml" ]]; then
    echo "--- Comprobando workspace de Cargo ---"

    # Formato de código (cargo fmt)
    if cargo fmt --version >/dev/null 2>&1; then
        echo "Ejecutando: cargo fmt --all --check"
        cargo fmt --all --check
    else
        echo "[AVISO] 'cargo fmt' no está disponible en este entorno; omitiendo verificación de formato."
    fi

    # Linter (cargo clippy)
    if cargo clippy --version >/dev/null 2>&1; then
        echo "Ejecutando: cargo clippy --workspace --all-targets -- -D warnings"
        cargo clippy --workspace --all-targets -- -D warnings
    else
        echo "[AVISO] 'cargo clippy' no está disponible en este entorno; omitiendo análisis de clippy."
    fi

    # Pruebas unitarias y de integración
    echo "Ejecutando: cargo test --workspace"
    cargo test --workspace

    # Documentación estricta
    echo "Ejecutando: RUSTDOCFLAGS=\"-D warnings\" cargo doc --workspace --no-deps"
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

    # Auditoría de dependencias y licencias (cargo-deny)
    if command -v cargo-deny >/dev/null 2>&1 || cargo deny --version >/dev/null 2>&1; then
        echo "Ejecutando: cargo deny check"
        cargo deny check
    else
        echo "[AVISO] 'cargo-deny' no está instalado; omitiendo auditoría de licencias y vulnerabilidades."
    fi
else
    echo "[AVISO] No se encontró Cargo.toml en la raíz del repositorio."
    echo "        El workspace de Cargo aún no está inicializado (se creará en WP-2.1)."
fi

# 2. Verificación ortográfica (typos)
if command -v typos >/dev/null 2>&1; then
    echo "--- Ejecutando corrector ortográfico (typos) ---"
    typos
else
    echo "[AVISO] 'typos' no está instalado en el PATH; omitiendo revisión ortográfica."
fi

echo "=== Todas las verificaciones completadas con éxito ==="
exit 0
