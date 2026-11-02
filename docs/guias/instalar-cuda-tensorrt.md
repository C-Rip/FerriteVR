# Instalación y configuración de CUDA, cuDNN y TensorRT en Nobara 44

Esta guía describe el procedimiento técnico para habilitar la aceleración por hardware NVIDIA (CUDA, cuDNN y TensorRT) en **Nobara Linux 44**, preparando el entorno para la inferencia de RTMPose con la crate `ort` 2.x en **FerriteVR** (requerido para el WP-4.4).

---

## 1. Diagnóstico del entorno en Nobara 44

El sistema base detectado presenta las siguientes características:

- **Sistema Operativo:** Nobara Linux 44 (KDE Plasma Wayland, base Fedora 44).
- **GPU:** NVIDIA GeForce RTX 3060 12 GB GDDR6.
- **Controlador NVIDIA:** Serie 595 empaquetada por el proyecto Negativo17 y gestionada nativamente por los repositorios de Nobara (`nvidia-driver-595`, `dkms-nvidia`, `nvidia-driver-cuda-libs`).
- **Repositorio de producción activo:** `nobara-nvidia-production` (`/etc/yum.repos.d/nv-nvp.repo`), configurado con prioridad alta.
- **Paquetes de ONNX del sistema:** El sistema cuenta con `onnxruntime-1.22.2` provisto por los repositorios de Fedora. **Nota técnica:** este paquete solo incluye el backend de CPU; Fedora no distribuye compilaciones con CUDA por razones de licenciamiento.

> [!WARNING]
> **Posible desalineación de versión (Driver/library mismatch):**
> Si `nvidia-smi` arroja el error `Failed to initialize NVML: Driver/library version mismatch`, se debe a que las bibliotecas en disco (`nvidia-driver-595.112`) fueron actualizadas recientemente mientras el módulo del kernel cargado en memoria sigue en una versión previa (`595.104`). Un reinicio del sistema (`sudo reboot`) normaliza el estado del controlador.

---

## 2. Requerimientos de la crate `ort` 2.x

La crate `ort` (versión `2.0.0-rc.15` en adelante) ofrece dos estrategias principales para vincular el runtime:

### Comparativa: `download-binaries` vs. `load-dynamic`

| Característica | `download-binaries` (Predeterminado) | `load-dynamic` |
|---|---|---|
| **Funcionamiento** | Descarga automáticamente binarios estáticos precompilados de ONNX Runtime desde la CDN de Pyke durante `cargo build`. | No enlaza ONNX Runtime en compilación; carga dinámicamente un archivo `.so` externo en tiempo de ejecución. |
| **Soporte GPU** | El paquete para `x86_64-unknown-linux-gnu` incluye los proveedores de ejecución **CUDA 13**, **TensorRT** y **NVRtx** precompilados. | Requiere que el usuario consiga o compile manualmente un `libonnxruntime.so` con soporte GPU habilitado. |
| **Compatibilidad con RPM de Fedora** | No depende del RPM de Fedora; evita el problema de que `onnxruntime.rpm` sea solo CPU. | Incompatible con el RPM de Fedora si se desea usar GPU. |
| **Carga de bibliotecas NVIDIA** | Carga dinámica vía `dlopen` de las bibliotecas de CUDA/cuDNN del sistema en tiempo de ejecución. | Carga dinámica vía `dlopen`. |

### Veredicto para FerriteVR
Se adopta la estrategia predeterminada **`download-binaries`** en `ferrite-pose`. No requiere compilar ONNX Runtime y asegura total compatibilidad con las APIs de la versión 2.x de `ort`.

### Bibliotecas del sistema requeridas en tiempo de ejecución
Al inicializar los Execution Providers, ONNX Runtime buscará dinámicamente en las rutas estándar (`/usr/lib64` o `LD_LIBRARY_PATH`):

1. **CUDA 13:** `libcudart.so.13`, `libcublas.so.13`, `libcublasLt.so.13`, `libnvrtc.so.13`, `libcurand.so.10`, `libcufft.so.12`.
2. **cuDNN 9:** `libcudnn.so.9`, `libcudnn_graph.so.9`, `libcudnn_ops.so.9`, `libcudnn_cnn.so.9`, `libcudnn_adv.so.9`, `libcudnn_heuristic.so.9`, `libcudnn_engines_precompiled.so.9`, `libcudnn_engines_runtime_compiled.so.9`.
3. **TensorRT 10 (opcional para el backend TensorRT):** `libnvinfer.so.10`, `libnvinfer_plugin.so.10`.

---

## 3. Vía recomendada de instalación en Nobara 44

> [!CAUTION]
> **¡NO instales el repositorio upstream oficial de NVIDIA ni el metapaquete `cuda`!**
> Agregar el repositorio `developer.download.nvidia.com` o instalar el metapaquete `cuda` reemplaza los controladores del kernel gestionados por Nobara (`dkms-nvidia` y `nvidia-driver-595`). Esto rompe con frecuencia el arranque gráfico de Wayland, SteamVR y Steam Link.

La vía limpia y recomendada consiste en aprovechar el repositorio **`nobara-nvidia-production`**, el cual proporciona paquetes modulares de CUDA 13.2 y cuDNN 9.20 diseñados específicamente para convivir con el controlador `nvidia-driver-595` sin tocar los módulos del kernel:

- `cuda-cudart`: Biblioteca del runtime de CUDA (`libcudart.so.13`).
- `cuda-extra-libs`: Bibliotecas complementarias (`libcublas`, `libcufft`, `libcurand`).
- `cuda-nvrtc`: Compilador en tiempo de ejecución de CUDA (`libnvrtc.so.13`).
- `cuda-cudnn`: Primitivas de redes neuronales cuDNN 9 (`libcudnn.so.9`).

---

## 4. Instrucciones paso a paso (requiere `sudo`)

### Paso 1: Reiniciar el equipo si hay desalineación del driver
Si `nvidia-smi` arroja error de mismatch de bibliotecas:
```bash
sudo reboot
```

Al reiniciar, comprueba que el controlador responda correctamente:
```bash
nvidia-smi
```

### Paso 2: Instalar CUDA 13 y cuDNN 9 desde los repositorios de Nobara
Ejecuta la instalación de las bibliotecas modulares de cómputo:
```bash
sudo dnf install -y cuda-cudart cuda-extra-libs cuda-nvrtc cuda-cudnn
```

### Paso 3: (Opcional) Instalar TensorRT 10
Nobara y Fedora no distribuyen TensorRT mediante paquetes RPM. Para evaluar el backend TensorRT en el WP-4.4, la forma más limpia que no contamina el sistema de paquetes es utilizar el tarball oficial:

1. Descarga el archivo comprimido de **TensorRT 10.x para Linux x86_64 (compatible con CUDA 12/13)** desde el [portal de desarrolladores de NVIDIA](https://developer.nvidia.com/tensorrt).
2. Descomprime el paquete en `/opt`:
   ```bash
   sudo tar -xvf TensorRT-10.*.Linux.x86_64-gnu.cuda-*.tar.gz -C /opt/
   sudo ln -s /opt/TensorRT-10.* /opt/tensorrt
   ```
3. Registra las bibliotecas de TensorRT en el enlazador dinámico del sistema:
   ```bash
   echo "/opt/tensorrt/lib" | sudo tee /etc/ld.so.conf.d/tensorrt.conf
   sudo ldconfig
   ```

---

## 5. Verificación de la instalación

### 1. Comprobar bibliotecas de CUDA 13 y cuDNN 9
```bash
ldconfig -p | grep -E "libcudart\.so\.13|libcublas\.so\.13|libcudnn\.so\.9"
```

### 2. Comprobar bibliotecas de TensorRT 10 (si se instaló)
```bash
ldconfig -p | grep -E "libnvinfer\.so|libnvinfer_plugin\.so"
```
