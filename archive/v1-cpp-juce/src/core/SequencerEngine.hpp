#pragma once

#include "core/Domain.hpp"

#include <random>
#include <vector>

namespace benoqs {

enum class MidiEventType {
    noteOn,
    noteOff,
    controlChange,
    allNotesOff,
};

struct MidiEvent {
    std::int64_t tick = 0;
    MidiEventType type = MidiEventType::noteOn;
    int port = 1;
    int channel = 1;
    int data1 = 0;
    int data2 = 0;
};

struct TrackRuntime {
    double accumulator = 0.0;
    int position = 0;
    int pingDirection = 1;
};

class SequencerEngine {
public:
    explicit SequencerEngine(std::uint32_t seed = 0x0c70'0001);

    void reset();
    std::vector<MidiEvent> setRunning(bool shouldRun);
    std::vector<MidiEvent> tick(const GridState& grid);

    [[nodiscard]] bool isRunning() const { return running_; }
    [[nodiscard]] std::int64_t currentTick() const { return globalTick_; }
    [[nodiscard]] int trackPosition(int trackIndex) const;

private:
    struct PortChannel {
        int port = 1;
        int channel = 1;
    };

    int grooveDelayTicks(int groove);
    int strumOffsetTicks(int levelAbs, int noteNumberOneBased) const;
    PortChannel resolvePortChannel(int midiChannel, RoutingMode routingMode, FixedRouting fixedRouting, int trackIndex) const;

    void schedule(MidiEvent event);
    std::vector<MidiEvent> flushDueEvents();
    void advanceTrack(TrackRuntime& runtime, Direction direction, int pageLength);

    bool running_ = false;
    std::int64_t globalTick_ = 0;
    std::array<TrackRuntime, kTrackCount> trackRuntime_ {};
    std::vector<MidiEvent> eventQueue_;
    std::mt19937 random_;
};

} // namespace benoqs
