// octoffi.h — hand-written C header for crates/octoffi/src/lib.rs.
// Owner: Conductor. Keep this in exact sync with the Rust `#[no_mangle]`
// functions and `#[repr(C)]` types until `cbindgen` is available in this dev
// environment to generate it instead (it isn't yet — no network/install done
// for it here).
//
// Swift imports this via a bridging header / module map once octopanel or
// octoshell exist to link against `liboctoffi.a` / `liboctoffi.dylib`.

#ifndef OCTOFFI_H
#define OCTOFFI_H

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct OctoEngine OctoEngine; // opaque

// --- Command, mirrors octocore::types::Command (#[repr(C)]) ---
// A Rust `#[repr(C)]` enum with data is a C-ABI-compatible tagged union: a
// discriminant tag followed by the active variant's payload. The tag order
// below must exactly match declaration order in types.rs.
typedef enum {
    OCTO_CMD_PLAY = 0,
    OCTO_CMD_STOP,
    OCTO_CMD_CONTINUE,
    OCTO_CMD_RESET,
    OCTO_CMD_BUTTON_DOWN,
    OCTO_CMD_BUTTON_UP,
    OCTO_CMD_ENCODER_TURN,
    OCTO_CMD_SET_ACTIVE_PAGE,
    OCTO_CMD_SET_MODE,
    OCTO_CMD_HOST_TRANSPORT,
    OCTO_CMD_LOAD_STATE,
} OctoCommandTag;

typedef struct {
    OctoCommandTag tag;
    union {
        struct { uint32_t control; float velocity_mm_s; } button_down;
        struct { uint32_t control; } button_up;
        struct { uint32_t control; int16_t detents; float angular_velocity; } encoder_turn;
        struct { uint8_t bank; uint8_t page; } set_active_page;
        struct { uint8_t mode; } set_mode; // 0=Grid 1=Page 2=Track 3=Step
        struct { uint64_t ppqn_pos; float bpm; bool playing; } host_transport;
        struct { uint64_t handle; } load_state;
    };
} OctoCommand;

// --- Event, mirrors octocore::types::Event (#[repr(C)]) ---
typedef enum {
    OCTO_EVT_NOTE_ON = 0,
    OCTO_EVT_NOTE_OFF,
    OCTO_EVT_CC,
} OctoEventTag;

typedef struct {
    OctoEventTag tag;
    union {
        struct { uint8_t port, ch, note, vel; uint32_t at_sample; } note_on;
        struct { uint8_t port, ch, note; uint32_t at_sample; } note_off;
        struct { uint8_t port, ch, cc, val; uint32_t at_sample; } cc;
    };
} OctoEvent;

typedef struct {
    float sample_rate;
    uint32_t buffer_len;
    float bpm;
    bool playing;
} OctoRenderParams;

OctoEngine *octocore_engine_new(uint64_t seed);
void octocore_engine_free(OctoEngine *engine);
void octocore_engine_handle_command(OctoEngine *engine, OctoCommand cmd);
int32_t octocore_engine_render(OctoEngine *engine, OctoRenderParams params,
                                OctoEvent *out_events, size_t out_capacity,
                                size_t *out_count);
bool octocore_engine_is_running(const OctoEngine *engine);

#ifdef __cplusplus
}
#endif

#endif // OCTOFFI_H
