#include "core/Domain.hpp"

#include <algorithm>

namespace benoqs {

int clampInt(const int value, const int low, const int high)
{
    return std::clamp(value, low, high);
}

int clampMidi(const int value)
{
    return clampInt(value, 0, 127);
}

Page makeDefaultPage()
{
    Page page;

    constexpr std::array<int, kTrackCount> manualPitches {
        57, 55, 52, 50, 48, 60, 62, 64, 67, 69
    };

    for (int trackIndex = 0; trackIndex < kTrackCount; ++trackIndex) {
        auto& track = page.tracks[static_cast<std::size_t>(trackIndex)];
        track.pitch = manualPitches[static_cast<std::size_t>(trackIndex)];
        track.midiChannel = 1;
    }

    return page;
}

GridState makeDefaultGrid()
{
    GridState grid;

    for (auto& bank : grid.banks) {
        for (auto& page : bank) {
            page = makeDefaultPage();
        }
    }

    return grid;
}

} // namespace benoqs
