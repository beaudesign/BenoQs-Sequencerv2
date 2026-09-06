#include "core/Scale.hpp"

namespace benoqs {

int normalisePitchClass(int pitchOrClass)
{
    auto pc = pitchOrClass % 12;
    if (pc < 0)
        pc += 12;
    return pc;
}

PitchClassSet buildScalePitchClasses(const int rootMidiOrPitchClass, const std::vector<int>& intervals)
{
    const auto root = normalisePitchClass(rootMidiOrPitchClass);
    const auto& activeIntervals = intervals.empty()
        ? Scale {}.intervals
        : intervals;

    PitchClassSet result;
    result.insert(root);

    for (const auto interval : activeIntervals) {
        result.insert(normalisePitchClass(root + interval));
    }

    return result;
}

int quantizeToScale(int pitch, const PitchClassSet& pitchClasses)
{
    pitch = clampMidi(pitch);

    if (pitchClasses.contains(normalisePitchClass(pitch)))
        return pitch;

    for (int distance = 1; distance <= 11; ++distance) {
        const auto down = pitch - distance;
        if (down >= 0 && pitchClasses.contains(normalisePitchClass(down)))
            return clampMidi(down);

        const auto up = pitch + distance;
        if (up <= 127 && pitchClasses.contains(normalisePitchClass(up)))
            return clampMidi(up);
    }

    return pitch;
}

} // namespace benoqs
