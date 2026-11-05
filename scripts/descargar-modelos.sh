#!/usr/bin/env bash
# Script canónico para descargar y verificar modelos ONNX de FerriteVR.
# Modelo: RTMPose-m (Halpe26) de OpenMMLab (Licencia Apache-2.0).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RAIZ_REPO="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIR_MODELOS="${RAIZ_REPO}/modelos"
DESTINO_ONNX="${DIR_MODELOS}/rtmpose-m-halpe26.onnx"
SHA256_ESPERADO="26f3a19e61304a600dfb82d1001d41d24343b89fc70a33ffc84657e0b0bf2ecf"

URL_ZIP="https://download.openmmlab.com/mmpose/v1/projects/rtmposev1/onnx_sdk/rtmpose-m_simcc-body7_pt-body7-halpe26_700e-256x192-4d3e73dd_20230605.zip"

echo "=== FerriteVR: Descargador de modelos ONNX ==="
mkdir -p "${DIR_MODELOS}"

# Función para verificar el hash SHA-256
verificar_hash() {
    local archivo="$1"
    local esperado="$2"
    local hash_actual
    hash_actual=$(sha256sum "${archivo}" | awk '{print $1}')
    if [[ "${hash_actual}" == "${esperado}" ]]; then
        return 0
    else
        echo "[ERROR] El hash SHA-256 no coincide."
        echo "        Esperado: ${esperado}"
        echo "        Obtenido: ${hash_actual}"
        return 1
    fi
}

# 1. Comprobar si el archivo ya existe y es válido (idempotencia)
if [[ -f "${DESTINO_ONNX}" ]]; then
    echo "Comprobando integridad del modelo existente en ${DESTINO_ONNX}..."
    if verificar_hash "${DESTINO_ONNX}" "${SHA256_ESPERADO}"; then
        echo "[OK] El modelo RTMPose-m (Halpe26) ya está presente y verificado."
        exit 0
    else
        echo "[AVISO] El archivo existente está dañado o desactualizado. Descargando de nuevo..."
        rm -f "${DESTINO_ONNX}"
    fi
fi

# 2. Descargar paquete ZIP oficial
TEMP_ZIP="$(mktemp "${DIR_MODELOS}/rtmpose_download_XXXXXX.zip")"
TEMP_DIR="$(mktemp -d "${DIR_MODELOS}/rtmpose_extract_XXXXXX")"

cleanup() {
    rm -f "${TEMP_ZIP}"
    rm -rf "${TEMP_DIR}"
}
trap cleanup EXIT

echo "Descargando paquete oficial de OpenMMLab..."
curl -fL --progress-bar "${URL_ZIP}" -o "${TEMP_ZIP}"

echo "Extrayendo modelo ONNX..."
unzip -q -o "${TEMP_ZIP}" -d "${TEMP_DIR}"

ONNX_EXTRAIDO=$(find "${TEMP_DIR}" -name "end2end.onnx" | head -n 1)
if [[ -z "${ONNX_EXTRAIDO}" ]]; then
    echo "[ERROR] No se encontró 'end2end.onnx' en el paquete descargado."
    exit 1
fi

echo "Verificando suma de verificación SHA-256..."
if ! verificar_hash "${ONNX_EXTRAIDO}" "${SHA256_ESPERADO}"; then
    echo "[ERROR] La verificación de integridad del modelo falló."
    exit 1
fi

mv "${ONNX_EXTRAIDO}" "${DESTINO_ONNX}"
echo "[OK] Modelo RTMPose-m (Halpe26) instalado con éxito en:"
echo "     ${DESTINO_ONNX}"
