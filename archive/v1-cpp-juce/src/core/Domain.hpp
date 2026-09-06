#pragma once

#include <array>
#include <cstdint>
#include <optional>
#include <string>
#include <vector>

namespace benoqs {

inline constexpr int kTrackCount = 10;
inline constexpr int kStepCount = 16;
inline constexpr int kBankCount = 10;
inline constexpr int kPageCount = 16;
inline constexpr int kTicksPerQuarter = 192;
inline constexpr int kDefaultStepTicks = 12;

enum class Direction : int {
    forward = 1,
    reverse = 2,
    pingPong = 3,
    brownian = 4,
    random = 5,
};

struct Scale {
    bool enabled = false;
    bool locked = false;
    int root = 60;
    std::vector<int> intervals {0, 2, 4, 5, 7, 9, 11};
    std::string mode = "maj";
};

struct Step {
    bool active = false;
    bool skip = false;
    int pitchOffset = 0;
    int velocityOffset = 0;
    int lengthTicks = kDefaultStepTicks;
    int lengthMultiplier = 1;
    int startOffsetTicks = 0;
    int amount = 0;
    int groove = 0;
    int position = 8;
    std::optional<int> midiControlValue;
    std::vector<int> chordOffsets;
    int polyphony = 1;
    bool ghost = false;
    int strum = 0;
};

struct Track {
    int pitch = 60;
    int velocity = 100;
    int lengthFactor = 8;
    int startFactor = 8;
    Direction direction = Direction::forward;
    int amount = 0;
    int groove = 0;
    std::optional<int> midiControl;
    int midiChannel = 1;
    double multiplier = 1.0;
    bool paused = false;
    bool muted = false;
    bool soloed = false;
    int programChange = 0;
    std::optional<int> bankChange;
    std::optional<int> transposeChannel;
    std::string transposeMode = "relative";
    std::array<Step, kStepCount> steps {};
};

struct Page {
    int pitchOffset = 0;
    int velocityFactor = 8;
    int length = kStepCount;
    int start = 1;
    Scale scale {};
    bool clusterMode = false;
    std::array<bool, kTrackCount> mutePattern {};
    std::array<Track, kTrackCount> tracks {};
};

enum class RoutingMode {
    octopus,
    fixed,
};

struct FixedRouting {
    int baseChannelPort1 = 1;
    int baseChannelPort2 = 1;
};

struct GridState {
    double tempoBpm = 120.0;
    int activeBank = 0;
    int activePage = 0;
    RoutingMode routingMode = RoutingMode::octopus;
    FixedRouting fixedRouting {};
    std::array<std::array<Page, kPageCount>, kBankCount> banks {};
};

Page makeDefaultPage();
GridState makeDefaultGrid();

int clampInt(int value, int low, int high);
int clampMidi(int value);

} // namespace benoqs
