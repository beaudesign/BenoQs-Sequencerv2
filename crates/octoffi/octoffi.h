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

// --- Grid-mutation surface ---
// Logical (track, step) addressing, not a physical panel ControlId — see
// lib.rs's module comment on why this doesn't need panel.truth.json.
// track is always < 10, step < 16 (octocore::domain::TRACK_COUNT/STEP_COUNT).

typedef enum {
    OCTO_TRACK_PITCH = 0,
    OCTO_TRACK_VELOCITY,
    OCTO_TRACK_LENGTH_FACTOR,
    OCTO_TRACK_START_FACTOR,
    OCTO_TRACK_DIRECTION_RAW,
    OCTO_TRACK_ROTATION,
    OCTO_TRACK_AMOUNT,
    OCTO_TRACK_GROOVE,
    OCTO_TRACK_MIDI_CHANNEL,
    OCTO_TRACK_MUTED,
    OCTO_TRACK_SOLOED,
    OCTO_TRACK_PAUSED,
    OCTO_TRACK_RECORD_ARMED,
    OCTO_TRACK_IS_FEEDER,
    OCTO_TRACK_IS_LISTENER,
} OctoTrackAttr;

typedef enum {
    OCTO_STEP_ACTIVE = 0,
    OCTO_STEP_SKIP,
    OCTO_STEP_PITCH_OFFSET,
    OCTO_STEP_VELOCITY_OFFSET,
    OCTO_STEP_LENGTH_TICKS,
    OCTO_STEP_LENGTH_MULTIPLIER,
    OCTO_STEP_START_OFFSET,
    OCTO_STEP_AMOUNT,
    OCTO_STEP_STRUM,
    OCTO_STEP_HYPERSTEP,
    OCTO_STEP_PHRASE,
    OCTO_STEP_PHRASE_POS,
} OctoStepAttr;

// All setters return false (a no-op) for an out-of-range track/step index
// rather than crashing. All getters return 0 in that case too, which is
// indistinguishable from a real 0 — validate track < 10 / step < 16
// yourself first if that matters.
bool octocore_track_set_i32(OctoEngine *engine, uint8_t track, OctoTrackAttr attr, int32_t value);
int32_t octocore_track_get_i32(const OctoEngine *engine, uint8_t track, OctoTrackAttr attr);
bool octocore_step_set_i32(OctoEngine *engine, uint8_t track, uint8_t step, OctoStepAttr attr, int32_t value);
int32_t octocore_step_get_i32(const OctoEngine *engine, uint8_t track, uint8_t step, OctoStepAttr attr);

#ifdef __cplusplus
}
#endif

#endif // OCTOFFI_H
