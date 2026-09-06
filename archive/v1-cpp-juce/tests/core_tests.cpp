#include "core/Domain.hpp"
#include "core/Scale.hpp"
#include "core/SequencerEngine.hpp"

#include <cassert>
#include <iostream>

namespace {

using namespace benoqs;

void defaultGridMatchesOctopusTrackPitches()
{
    const auto grid = makeDefaultGrid();
    const auto& tracks = grid.banks[0][0].tracks;

    assert(tracks[0].pitch == 57);
    assert(tracks[4].pitch == 48);
    assert(tracks[5].pitch == 60);
    assert(tracks[9].pitch == 69);
}

void scaleQuantizationPrefersDownwardTies()
{
    const auto major = buildScalePitchClasses(60, {0, 2, 4, 5, 7, 9, 11});

    assert(quantizeToScale(61, major) == 60);
    assert(quantizeToScale(63, major) == 62);
    assert(quantizeToScale(64, major) == 64);
}

void activeStepEmitsNoteOnAndDelayedNoteOff()
{
    auto grid = makeDefaultGrid();
    auto& step = grid.banks[0][0].tracks[0].steps[0];
    step.active = true;
    step.lengthTicks = 3;

    SequencerEngine engine;
    engine.setRunning(true);

    std::vector<MidiEvent> firstBoundary;
    for (int i = 0; i < kDefaultStepTicks; ++i) {
        auto events = engine.tick(grid);
        if (!events.empty())
            firstBoundary = events;
    }

    assert(firstBoundary.size() == 1);
    assert(firstBoundary[0].type == MidiEventType::noteOn);
    assert(firstBoundary[0].data1 == 57);
    assert(firstBoundary[0].data2 == 100);

    std::vector<MidiEvent> noteOff;
    for (int i = 0; i < 3; ++i) {
        auto events = engine.tick(grid);
        if (!events.empty())
            noteOff = events;
    }

    assert(noteOff.size() == 1);
    assert(noteOff[0].type == MidiEventType::noteOff);
    assert(noteOff[0].data1 == 57);
}

void stopProducesPanicMessages()
{
    SequencerEngine engine;
    engine.setRunning(true);
    const auto panic = engine.setRunning(false);

    assert(panic.size() == 33);
    assert(panic.front().type == MidiEventType::controlChange);
    assert(panic.front().data1 == 123);
    assert(panic.back().type == MidiEventType::allNotesOff);
}

} // namespace

int main()
{
    defaultGridMatchesOctopusTrackPitches();
    scaleQuantizationPrefersDownwardTies();
    activeStepEmitsNoteOnAndDelayedNoteOff();
    stopProducesPanicMessages();

    std::cout << "benoqs_core_tests passed\n";
    return 0;
}
