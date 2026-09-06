#pragma once

#include "core/Domain.hpp"

#include <unordered_set>

namespace benoqs {

using PitchClassSet = std::unordered_set<int>;

int normalisePitchClass(int pitchOrClass);
PitchClassSet buildScalePitchClasses(int rootMidiOrPitchClass, const std::vector<int>& intervals);
int quantizeToScale(int pitch, const PitchClassSet& pitchClasses);

} // namespace benoqs
