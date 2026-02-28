#ifndef NOMAD_ENGINE_H
#define NOMAD_ENGINE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/// Opaque handle to the Nomad engine
typedef struct NomadEngine NomadEngine;

/// A byte buffer returned from the engine
typedef struct {
    uint8_t *data;
    size_t len;
} ByteBuffer;

/// Creates a new Nomad engine instance.
/// Returns NULL on failure.
/// The returned pointer must be freed with nomad_engine_destroy.
NomadEngine* nomad_engine_create(void);

/// Destroys a Nomad engine instance.
void nomad_engine_destroy(NomadEngine* engine);

/// Loads a URL in the engine.
/// Returns 0 on success, non-zero on failure.
int32_t nomad_engine_load_url(NomadEngine* engine, const char* url);

/// Ticks the engine (for animations/updates).
void nomad_engine_tick(NomadEngine* engine);

/// Gets the display list from the engine as a byte buffer.
/// The caller must free the buffer with nomad_free_byte_buffer.
ByteBuffer nomad_engine_get_display_list(const NomadEngine* engine);

/// Sets the viewport width for layout.
void nomad_engine_set_viewport_width(NomadEngine* engine, float width);

/// Frees a byte buffer returned by the engine.
void nomad_free_byte_buffer(ByteBuffer buffer);

/// Frees a string returned by the engine.
void nomad_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif // NOMAD_ENGINE_H
