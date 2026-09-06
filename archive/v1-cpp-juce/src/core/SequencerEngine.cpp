#include "core/SequencerEngine.hpp"

#include "core/Scale.hpp"

#include <algorithm>
#include <cmath>

namespace benoqs {

namespace {

int scaleFactor(int value, int neutral)
{
    return std::clamp(value, 0, neutral * 2);
}

} // namespace

SequencerEngine::SequencerEngine(const std::uint32_t seed)
    : random_ {seed}
{
    reset();
}

void SequencerEngine::reset()
{
    running_ = false;
    globalTick_ = 0;
    eventQueue_.clear();
    trackRuntime_ = {};
}

std::vector<MidiEvent> SequencerEngine::setRunning(const bool shouldRun)
{
    running_ = shouldRun;

    if (running_)
        return {};

    eventQueue_.clear();

    std::vector<MidiEvent> panic;
    panic.reserve(32);

    for (int port = 1; port <= 2; ++port) {
        for (int channel = 1; channel <= 16; ++channel) {
            panic.push_back({globalTick_, MidiEventType::controlChange, port, channel, 123, 0});
        }
    }

    panic.push_back({globalTick_, MidiEventType::allNotesOff, 1, 1, 0, 0});
    return panic;
}

std::vector<MidiEvent> SequencerEngine::tick(const GridState& grid)
{
    if (!running_)
        return {};

    ++globalTick_;

    const auto bankIndex = clampInt(grid.activeBank, 0, kBankCount - 1);
    const auto pageIndex = clampInt(grid.activePage, 0, kPageCount - 1);
    const auto& page = grid.banks[static_cast<std::size_t>(bankIndex)][static_cast<std::size_t>(pageIndex)];
    const auto pageLength = clampInt(page.length, 1, kStepCount);

    for (int trackIndex = 0; trackIndex < kTrackCount; ++trackIndex) {
        const auto& track = page.tracks[static_cast<std::size_t>(trackIndex)];
        if (track.paused || track.muted)
            continue;

        auto& runtime = trackRuntime_[static_cast<std::size_t>(trackIndex)];
        const auto multiplier = track.multiplier > 0.0 ? track.multiplier : 1.0;
        const auto stepTicks = static_cast<double>(kDefaultStepTicks) / multiplier;

        runtime.accumulator += 1.0;
        if (runtime.accumulator + 1.0e-9 < stepTicks)
            continue;

        runtime.accumulator -= stepTicks;

        auto stepIndex = runtime.position % pageLength;
        if (stepIndex < 0)
            stepIndex += pageLength;

        const auto& step = track.steps[static_cast<std::size_t>(stepIndex)];
        const auto shuffleDelay = (stepIndex % 2) == 1 ? grooveDelayTicks(track.groove) : 0;

        if (step.active && !step.skip) {
            auto pitch = clampMidi(page.pitchOffset + track.pitch + step.pitchOffset);
            if (page.scale.enabled) {
                pitch = quantizeToScale(pitch, buildScalePitchClasses(page.scale.root, page.scale.intervals));
            }

            auto velocity = clampMidi(track.velocity + step.velocityOffset);
            velocity = clampMidi(static_cast<int>(std::lround((velocity * scaleFactor(page.velocityFactor, 8)) / 8.0)));

            const auto startScale = scaleFactor(track.startFactor, 8) / 8.0;
            const auto startOffset = static_cast<int>(std::lround(step.startOffsetTicks * startScale));

            const auto baseLength = clampInt(step.lengthTicks, 1, kTicksPerQuarter);
            const auto lengthMultiplier = clampInt(step.lengthMultiplier, 1, 8);
            const auto rawLength = std::min(kTicksPerQuarter, baseLength * lengthMultiplier);
            const auto lengthScale = scaleFactor(track.lengthFactor, 8) / 8.0;
            const auto finalLength = clampInt(static_cast<int>(std::lround(rawLength * lengthScale)), 1, kTicksPerQuarter);

            const auto portChannel = resolvePortChannel(track.midiChannel, grid.routingMode, grid.fixedRouting, trackIndex);
            const auto onTick = globalTick_ + shuffleDelay + startOffset;

            std::vector<int> noteOffsets {0};
            if (!step.chordOffsets.empty()) {
                std::vector<int> pool {0};
                pool.insert(pool.end(), step.chordOffsets.begin(), step.chordOffsets.end());

                const auto polyphony = clampInt(step.polyphony, 1, 7);
                const auto toPlay = std::min(polyphony, static_cast<int>(pool.size()));

                noteOffsets.clear();

                if (toPlay == static_cast<int>(pool.size())) {
                    noteOffsets = pool;
                } else {
                    std::shuffle(pool.begin(), pool.end(), random_);
                    noteOffsets.insert(noteOffsets.end(), pool.begin(), pool.begin() + toPlay);
                }
            }

            std::vector<int> pitches;
            pitches.reserve(noteOffsets.size());
            for (const auto noteOffset : noteOffsets)
                pitches.push_back(clampMidi(pitch + noteOffset));

            std::sort(pitches.begin(), pitches.end());

            const auto strum = clampInt(step.strum, -9, 9);
            if (strum < 0)
                std::reverse(pitches.begin(), pitches.end());

            const auto strumLevel = std::abs(strum);

            if (strumLevel > 0 && pitches.size() == 1) {
                for (int noteNumber = 1; noteNumber <= 7; ++noteNumber) {
                    const auto offset = strumOffsetTicks(strumLevel, noteNumber);
                    schedule({onTick + offset, MidiEventType::noteOn, portChannel.port, portChannel.channel, pitches.front(), velocity});
                    schedule({onTick + offset + finalLength, MidiEventType::noteOff, portChannel.port, portChannel.channel, pitches.front(), 0});
                }
            } else {
                for (std::size_t noteIndex = 0; noteIndex < pitches.size(); ++noteIndex) {
                    const auto offset = strumOffsetTicks(strumLevel, static_cast<int>(noteIndex) + 1);
                    schedule({onTick + offset, MidiEventType::noteOn, portChannel.port, portChannel.channel, pitches[noteIndex], velocity});
                    schedule({onTick + offset + finalLength, MidiEventType::noteOff, portChannel.port, portChannel.channel, pitches[noteIndex], 0});
                }
            }

            if (track.midiControl && step.midiControlValue) {
                schedule({onTick, MidiEventType::controlChange, portChannel.port, portChannel.channel, clampMidi(*track.midiControl), clampMidi(*step.midiControlValue)});
            }
        }

        advanceTrack(runtime, track.direction, pageLength);
    }

    return flushDueEvents();
}

int SequencerEngine::trackPosition(const int trackIndex) const
{
    return trackRuntime_[static_cast<std::size_t>(clampInt(trackIndex, 0, kTrackCount - 1))].position;
}

int SequencerEngine::grooveDelayTicks(const int groove)
{
    const auto value = clampInt(groove, 0, 16);
    if (value == 0)
        return 0;

    if ((value % 2) == 1)
        return (value + 1) / 2;

    const auto low = (value / 2) - 1;
    const auto high = low + 2;
    std::uniform_int_distribution<int> distribution {low, high};
    return distribution(random_);
}

int SequencerEngine::strumOffsetTicks(const int levelAbs, const int noteNumberOneBased) const
{
    if (noteNumberOneBased <= 1)
        return 0;

    const auto level = clampInt(levelAbs, 0, 9);
    if (level == 0)
        return 0;

    static constexpr std::array<std::array<int, 9>, 6> table {{
        {{0, 1, 1, 2, 2, 3, 3, 4, 5}},
        {{1, 2, 3, 4, 5, 6, 7, 8, 10}},
        {{1, 2, 4, 6, 8, 9, 10, 13, 17}},
        {{2, 3, 5, 9, 11, 13, 15, 19, 23}},
        {{2, 3, 6, 12, 15, 18, 21, 27, 30}},
        {{3, 6, 9, 16, 19, 24, 29, 36, 45}},
    }};

    if (noteNumberOneBased < 2 || noteNumberOneBased > 7)
        return 0;

    return table[static_cast<std::size_t>(noteNumberOneBased - 2)][static_cast<std::size_t>(level - 1)];
}

SequencerEngine::PortChannel SequencerEngine::resolvePortChannel(
    const int midiChannel,
    const RoutingMode routingMode,
    const FixedRouting fixedRouting,
    const int trackIndex) const
{
    if (routingMode == RoutingMode::fixed) {
        const auto base = clampInt(fixedRouting.baseChannelPort1, 1, 16);
        return {1, ((base - 1 + trackIndex) % 16) + 1};
    }

    const auto channel = std::max(1, midiChannel);
    if (channel <= 16)
        return {1, channel};

    return {2, clampInt(channel - 16, 1, 16)};
}

void SequencerEngine::schedule(MidiEvent event)
{
    const auto insertAt = std::upper_bound(
        eventQueue_.begin(),
        eventQueue_.end(),
        event.tick,
        [](const std::int64_t tick, const MidiEvent& queued) {
            return tick < queued.tick;
        });

    eventQueue_.insert(insertAt, event);
}

std::vector<MidiEvent> SequencerEngine::flushDueEvents()
{
    std::vector<MidiEvent> due;

    auto firstFuture = std::find_if(eventQueue_.begin(), eventQueue_.end(), [this](const MidiEvent& event) {
        return event.tick > globalTick_;
    });

    due.insert(due.end(), eventQueue_.begin(), firstFuture);
    eventQueue_.erase(eventQueue_.begin(), firstFuture);
    return due;
}

void SequencerEngine::advanceTrack(TrackRuntime& runtime, const Direction direction, const int pageLength)
{
    switch (direction) {
        case Direction::forward:
            runtime.position = (runtime.position + 1) % pageLength;
            break;
        case Direction::reverse:
            runtime.position = (runtime.position - 1 + pageLength) % pageLength;
            break;
        case Direction::pingPong:
            if (runtime.position <= 0)
                runtime.pingDirection = 1;
            else if (runtime.position >= pageLength - 1)
                runtime.pingDirection = -1;
            runtime.position += runtime.pingDirection;
            break;
        case Direction::brownian: {
            std::bernoulli_distribution forwardDistribution {2.0 / 3.0};
            runtime.position = (runtime.position + (forwardDistribution(random_) ? 1 : -1) + pageLength) % pageLength;
            break;
        }
        case Direction::random: {
            std::uniform_int_distribution<int> distribution {0, pageLength - 1};
            runtime.position = distribution(random_);
            break;
        }
    }
}

} // namespace benoqs
