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

/// Navigates to a URL, resolving it relative to the current page if needed.
/// This should be used for link clicks instead of load_url.
/// Returns 0 on success, non-zero on failure.
int32_t nomad_engine_navigate(NomadEngine* engine, const char* url);

/// Submits a form with the given input values.
/// inputs_json should be a JSON string containing an array of [name, value] pairs.
/// Example: [["q", "search term"], ["lang", "en"]]
/// Returns 0 on success, non-zero on failure.
int32_t nomad_engine_submit_form(NomadEngine* engine, size_t form_index, const char* inputs_json);

/// Goes back in navigation history.
/// Returns 0 on success, non-zero on failure.
int32_t nomad_engine_go_back(NomadEngine* engine);

/// Goes forward in navigation history.
/// Returns 0 on success, non-zero on failure.
int32_t nomad_engine_go_forward(NomadEngine* engine);

/// Checks if the engine can go back in history.
/// Returns 1 if can go back, 0 if cannot.
int32_t nomad_engine_can_go_back(const NomadEngine* engine);

/// Checks if the engine can go forward in history.
/// Returns 1 if can go forward, 0 if cannot.
int32_t nomad_engine_can_go_forward(const NomadEngine* engine);

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
