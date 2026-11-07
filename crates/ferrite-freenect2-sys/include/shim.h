#pragma once

#include <cstdint>
#include <memory>
#include <string>
#include "rust/cxx.h"

namespace libfreenect2 {
class Freenect2;
class Freenect2Device;
class SyncMultiFrameListener;
class Registration;
class Frame;
} // namespace libfreenect2

namespace ferrite {
namespace freenect2 {

/**
 * Encapsula la gestión de ciclo de vida del sensor Kinect v2 y registro 3D/color.
 */
class DispositivoKinect {
public:
    DispositivoKinect();
    ~DispositivoKinect();

    void abrir(int32_t modo_pipeline);
    void iniciar();
    void detener();
    bool esperar_y_registrar(
        int32_t tiempo_espera_ms,
        rust::Slice<float> buffer_profundidad,
        rust::Slice<uint8_t> buffer_color_rgba
    );
    void obtener_intrinsecos(float &fx, float &fy, float &cx, float &cy) const;
    rust::String obtener_numero_serie() const;
    rust::String obtener_version_firmware() const;

private:
    std::unique_ptr<libfreenect2::Freenect2> freenect2_;
    libfreenect2::Freenect2Device *dev_;
    std::unique_ptr<libfreenect2::SyncMultiFrameListener> listener_;
    std::unique_ptr<libfreenect2::Registration> registration_;
    std::unique_ptr<libfreenect2::Frame> undistorted_;
    std::unique_ptr<libfreenect2::Frame> registered_;
    bool iniciado_;
};

std::unique_ptr<DispositivoKinect> crear_dispositivo(int32_t modo_pipeline);
void configurar_nivel_registro(int32_t nivel);

} // namespace freenect2
} // namespace ferrite
