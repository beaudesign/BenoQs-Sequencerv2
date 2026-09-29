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

// 1 was this header before the command ring and the snapshot; 2 adds them. Nothing that 1
// declared has changed. octocore_abi_version() returns the number the library was built with.
#define OCTOFFI_ABI_VERSION 2

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct OctoEngine OctoEngine; // opaque

// --- Attribute numbers, shared by the setters below and by the SetTrack / SetStep commands ---
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
    OCTO_CMD_SET_TRACK, // added with SPEC-0001 O6; earlier tags unchanged. Sizeof(OctoCommand) stays 24.
    OCTO_CMD_SET_STEP,  // added with SPEC-0001 O6
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
        struct { uint8_t track; OctoTrackAttr attr; int32_t value; } set_track;                // active page; track < 10
        struct { uint8_t track; uint8_t step; OctoStepAttr attr; int32_t value; } set_step;   // active page; track < 10, step < 16
    };
} OctoCommand;

// --- Event, mirrors octocore::types::Event (#[repr(C)]) ---
typedef enum {
    OCTO_EVT_NOTE_ON = 0,
    OCTO_EVT_NOTE_OFF,
    OCTO_EVT_CC,
    OCTO_EVT_PITCH_BEND,       // added with SPEC-0001 O10; earlier tags unchanged
    OCTO_EVT_CHANNEL_PRESSURE, // added with SPEC-0001 O10
} OctoEventTag;

typedef struct {
    OctoEventTag tag;
    union {
        struct { uint8_t port, ch, note, vel; uint32_t at_sample; } note_on;
        struct { uint8_t port, ch, note; uint32_t at_sample; } note_off;
        struct { uint8_t port, ch, cc, val; uint32_t at_sample; } cc;
        struct { uint8_t port, ch; uint16_t value; uint32_t at_sample; } pitch_bend; // 0..16383, 8192 = centre
        struct { uint8_t port, ch, value; uint32_t at_sample; } channel_pressure;    // 0..127
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

// Health counters, cumulative since octocore_engine_new. See octocore::engine::Diagnostics.
typedef struct {
    uint32_t queue_overflows;        // notes/CCs refused because the event queue was full
    uint32_t queue_high_water;       // most events the queue has held at once (cap 1024)
    uint32_t deferred_events;        // due events held back because out_events was full
    uint32_t late_events;            // events emitted at sample 0 after their time
    uint32_t unusable_tempo_renders; // running renders with bpm outside 1..999 or sample_rate outside 8000..768000
} OctoDiagnostics;

// Returns 0, or -1 for a null engine, -2 for a null out.
int32_t octocore_engine_diagnostics(const OctoEngine *engine, OctoDiagnostics *out);

// --- Grid-mutation surface ---
// Logical (track, step) addressing, not a physical panel ControlId — see
// lib.rs's module comment on why this doesn't need panel.truth.json.
// track is always < 10, step < 16 (octocore::domain::TRACK_COUNT/STEP_COUNT).

// All setters return false (a no-op) for an out-of-range track/step index
// rather than crashing. All getters return 0 in that case too, which is
// indistinguishable from a real 0 — validate track < 10 / step < 16
// yourself first if that matters.
bool octocore_track_set_i32(OctoEngine *engine, uint8_t track, OctoTrackAttr attr, int32_t value);
int32_t octocore_track_get_i32(const OctoEngine *engine, uint8_t track, OctoTrackAttr attr);
bool octocore_step_set_i32(OctoEngine *engine, uint8_t track, uint8_t step, OctoStepAttr attr, int32_t value);
int32_t octocore_step_get_i32(const OctoEngine *engine, uint8_t track, uint8_t step, OctoStepAttr attr);

// --- Link: the command ring and the snapshot (SPEC-0001 O6) ---
// The engine is single-threaded. Only the audio thread calls octocore_engine_render, and the
// grid setters above are for offline use, never while a render can run. Everything a host
// does from another thread goes through the two ends below: an OctoSender for the main thread
// (commands in) and an OctoReader for the render thread (snapshots out). Each end belongs to
// one thread at a time. Both are wait-free, and neither allocates after open_link. They keep
// the shared buffers alive by reference count, so they stay safe to use after the engine is
// freed.

typedef struct OctoSender OctoSender; // opaque
typedef struct OctoReader OctoReader; // opaque

uint32_t octocore_abi_version(void);

typedef struct {
    uint64_t pushed;  // commands pushed
    uint64_t dropped; // overwritten before the engine took them, or not a command
    uint64_t applied; // taken and applied by the engine
} OctoLinkStats;

// Once per engine. 0 on success and *sender, *reader are set. -1 null engine, -2 null out
// pointer, -3 already open (nothing is written). Allocates: call at setup.
int32_t octocore_engine_open_link(OctoEngine *engine, OctoSender **sender, OctoReader **reader);

// Wait-free. If the ring (1024 commands) is full, the oldest is overwritten and counted as
// dropped. Applied by the next render, in order, at most 256 per render, before any tick.
// false only for a null sender. A tag outside OctoCommandTag is undefined behaviour.
bool octocore_sender_push(OctoSender *sender, OctoCommand cmd);

// 0, or -1 for a null sender, -2 for a null out.
int32_t octocore_sender_stats(const OctoSender *sender, OctoLinkStats *out);

// The snapshot, as the engine publishes it at the end of every render. LED and encoder fields
// are zero until panel.truth.json exists to say which control is which.
#define OCTO_MAX_CONTROLS 512
#define OCTO_ENCODER_COUNT 20
#define OCTO_TRACK_COUNT 10

typedef struct { float r, g, b; } OctoLedColor;
typedef struct { OctoLedColor color; float target; } OctoLed;
typedef struct { float angle_radians; int32_t detent_index; } OctoEncoderState;
typedef struct { uint8_t step_index; uint8_t track_index; } OctoPlayheadState;
typedef struct { bool playing; uint64_t tick; } OctoTransportState;
typedef struct { uint8_t bank; uint8_t page; } OctoActiveRefs;

typedef struct {
    uint64_t generation; // number of renders published; 0 before the first
    OctoLed leds[OCTO_MAX_CONTROLS];
    OctoEncoderState encoders[OCTO_ENCODER_COUNT];
    OctoPlayheadState playheads[OCTO_TRACK_COUNT];
    OctoTransportState transport;
    uint8_t mode; // 0=Grid 1=Page 2=Track 3=Step
    OctoActiveRefs active;
} OctoSnapshot;

// Takes the newest published snapshot if there is one newer than the last claim, and returns
// a pointer to the reader's copy. With nothing new it returns the last one (all zero, generation 0,
// before the first publish): compare generation. The pointer is the same on every call and stays
// valid until octocore_reader_free; its contents change only inside this function. NULL for a
// null reader.
const OctoSnapshot *octocore_reader_claim(OctoReader *reader);

void octocore_sender_free(OctoSender *sender);
void octocore_reader_free(OctoReader *reader);

#ifdef __cplusplus
}
#endif

#endif // OCTOFFI_H
