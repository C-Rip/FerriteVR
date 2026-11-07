#include "shim.h"

#include <libfreenect2/libfreenect2.hpp>
#include <libfreenect2/frame_listener_impl.h>
#include <libfreenect2/registration.h>
#include <libfreenect2/packet_pipeline.h>
#include <libfreenect2/logger.h>

#include <stdexcept>
#include <cstring>

namespace ferrite {
namespace freenect2 {

DispositivoKinect::DispositivoKinect()
    : dev_(nullptr), iniciado_(false) {}

DispositivoKinect::~DispositivoKinect() {
    try {
        detener();
        if (dev_) {
            dev_->close();
            dev_ = nullptr;
        }
    } catch (...) {
        // En destructores FFI no se deben propagar excepciones.
    }
}

void DispositivoKinect::abrir(int32_t modo_pipeline) {
    freenect2_.reset(new libfreenect2::Freenect2());
    int cantidad = freenect2_->enumerateDevices();
    if (cantidad <= 0) {
        throw std::runtime_error("No se encontro ningun sensor Kinect v2 conectado via USB");
    }

    std::unique_ptr<libfreenect2::PacketPipeline> pipeline;
    if (modo_pipeline == 1) { // Cpu
        pipeline.reset(new libfreenect2::CpuPacketPipeline());
    } else if (modo_pipeline == 2) { // OpenGl
#ifdef LIBFREENECT2_WITH_OPENGL_SUPPORT
        pipeline.reset(new libfreenect2::OpenGLPacketPipeline());
#else
        throw std::runtime_error("OpenGL no esta disponible en esta compilacion de libfreenect2");
#endif
    } else if (modo_pipeline == 3) { // OpenCl
#ifdef LIBFREENECT2_WITH_OPENCL_SUPPORT
        pipeline.reset(new libfreenect2::OpenCLPacketPipeline());
#else
        throw std::runtime_error("OpenCL no esta disponible en esta compilacion de libfreenect2");
#endif
    }

    if (pipeline) {
        dev_ = freenect2_->openDefaultDevice(pipeline.release());
    } else {
        dev_ = freenect2_->openDefaultDevice();
    }

    if (!dev_) {
        throw std::runtime_error("Fallo al inicializar el dispositivo Kinect v2");
    }

    listener_.reset(new libfreenect2::SyncMultiFrameListener(
        libfreenect2::Frame::Color | libfreenect2::Frame::Depth
    ));

    dev_->setColorFrameListener(listener_.get());
    dev_->setIrAndDepthFrameListener(listener_.get());

    undistorted_.reset(new libfreenect2::Frame(512, 424, 4));
    registered_.reset(new libfreenect2::Frame(512, 424, 4));
}

void DispositivoKinect::iniciar() {
    if (!dev_) {
        throw std::runtime_error("Dispositivo Kinect v2 no inicializado");
    }
    if (!iniciado_) {
        if (!dev_->start()) {
            throw std::runtime_error("Fallo al iniciar recepcion de flujos del sensor Kinect v2");
        }
        iniciado_ = true;

        // La calibracion de fabrica de la camara solo esta disponible tras start()
        registration_.reset(new libfreenect2::Registration(
            dev_->getIrCameraParams(),
            dev_->getColorCameraParams()
        ));
    }
}

void DispositivoKinect::detener() {
    if (iniciado_ && dev_) {
        dev_->stop();
        iniciado_ = false;
        registration_.reset();
    }
}

bool DispositivoKinect::esperar_y_registrar(
    int32_t tiempo_espera_ms,
    rust::Slice<float> buffer_profundidad,
    rust::Slice<uint8_t> buffer_color_rgba
) {
    if (!iniciado_ || !dev_ || !listener_ || !registration_) {
        throw std::runtime_error("El sensor Kinect v2 no ha iniciado transmision");
    }

    constexpr size_t total_pixeles = 512 * 424;
    if (buffer_profundidad.size() < total_pixeles) {
        throw std::runtime_error("Buffer de profundidad insuficiente (< 217088 f32)");
    }
    if (buffer_color_rgba.size() < total_pixeles * 4) {
        throw std::runtime_error("Buffer de color insuficiente (< 868352 u8)");
    }

    libfreenect2::FrameMap frames;
    if (!listener_->waitForNewFrame(frames, tiempo_espera_ms)) {
        return false;
    }

    libfreenect2::Frame *rgb = frames[libfreenect2::Frame::Color];
    libfreenect2::Frame *depth = frames[libfreenect2::Frame::Depth];

    if (!rgb || !depth) {
        listener_->release(frames);
        return false;
    }

    registration_->apply(rgb, depth, undistorted_.get(), registered_.get());

    // Copia directa de profundidad sin distorsion (512x424 f32 en milimetros)
    std::memcpy(buffer_profundidad.data(), undistorted_->data, total_pixeles * sizeof(float));

    // Conversion rapida BGRX -> RGBA
    const uint8_t *src = registered_->data;
    uint8_t *dst = buffer_color_rgba.data();
    for (size_t i = 0; i < total_pixeles; ++i) {
        dst[i * 4 + 0] = src[i * 4 + 2]; // R
        dst[i * 4 + 1] = src[i * 4 + 1]; // G
        dst[i * 4 + 2] = src[i * 4 + 0]; // B
        dst[i * 4 + 3] = 255;            // A
    }

    listener_->release(frames);
    return true;
}

void DispositivoKinect::obtener_intrinsecos(float &fx, float &fy, float &cx, float &cy) const {
    if (!dev_) {
        throw std::runtime_error("Dispositivo Kinect v2 no inicializado");
    }
    libfreenect2::Freenect2Device::IrCameraParams ir = dev_->getIrCameraParams();
    fx = ir.fx;
    fy = ir.fy;
    cx = ir.cx;
    cy = ir.cy;
}

rust::String DispositivoKinect::obtener_numero_serie() const {
    if (!dev_) {
        return rust::String("");
    }
    return rust::String(dev_->getSerialNumber());
}

rust::String DispositivoKinect::obtener_version_firmware() const {
    if (!dev_) {
        return rust::String("");
    }
    return rust::String(dev_->getFirmwareVersion());
}

std::unique_ptr<DispositivoKinect> crear_dispositivo(int32_t modo_pipeline) {
    auto dev = std::unique_ptr<DispositivoKinect>(new DispositivoKinect());
    dev->abrir(modo_pipeline);
    return dev;
}

void configurar_nivel_registro(int32_t nivel) {
    libfreenect2::Logger::Level lvl = libfreenect2::Logger::Warning;
    switch (nivel) {
        case 0: lvl = libfreenect2::Logger::None; break;
        case 1: lvl = libfreenect2::Logger::Error; break;
        case 2: lvl = libfreenect2::Logger::Warning; break;
        case 3: lvl = libfreenect2::Logger::Info; break;
        case 4: lvl = libfreenect2::Logger::Debug; break;
        default: lvl = libfreenect2::Logger::Warning; break;
    }
    libfreenect2::setGlobalLogger(libfreenect2::createConsoleLogger(lvl));
}

} // namespace freenect2
} // namespace ferrite
