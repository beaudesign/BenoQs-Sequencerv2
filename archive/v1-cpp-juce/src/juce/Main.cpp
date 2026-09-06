#include <juce_gui_basics/juce_gui_basics.h>
#include <juce_gui_extra/juce_gui_extra.h>

#include "core/Domain.hpp"
#include "core/SequencerEngine.hpp"

#include <cmath>
#include <memory>

namespace {

class SequencerPanel final : public juce::Component, private juce::Timer {
public:
    SequencerPanel()
    {
        grid_ = benoqs::makeDefaultGrid();

        auto& page = grid_.banks[0][0];
        for (int track = 0; track < benoqs::kTrackCount; ++track) {
            page.tracks[static_cast<std::size_t>(track)].steps[0].active = true;
            page.tracks[static_cast<std::size_t>(track)].steps[(track * 3) % benoqs::kStepCount].active = true;
        }

        engine_.setRunning(true);
        startTimerHz(60);
    }

    void paint(juce::Graphics& g) override
    {
        const auto bounds = getLocalBounds().toFloat();
        g.fillAll(juce::Colour::fromRGB(238, 234, 224));

        g.setColour(juce::Colour::fromRGB(38, 34, 28));
        g.setFont(juce::FontOptions(22.0f, juce::Font::italic));
        g.drawText("BenoQs Sequencer", getLocalBounds().removeFromTop(52), juce::Justification::centred);

        const auto matrixArea = bounds.reduced(36.0f, 70.0f).withTrimmedRight(bounds.getWidth() * 0.34f);
        const auto cell = std::min(matrixArea.getWidth() / 18.0f, matrixArea.getHeight() / 12.0f);
        const auto gap = cell * 0.36f;
        const auto origin = matrixArea.getTopLeft() + juce::Point<float>(cell * 1.5f, cell);
        const auto& page = grid_.banks[0][0];

        for (int track = 0; track < benoqs::kTrackCount; ++track) {
            for (int step = 0; step < benoqs::kStepCount; ++step) {
                const auto x = origin.x + step * (cell + gap);
                const auto y = origin.y + track * (cell + gap);
                const auto& stepState = page.tracks[static_cast<std::size_t>(track)].steps[static_cast<std::size_t>(step)];
                const auto isPlayhead = engine_.trackPosition(track) == step;

                auto colour = juce::Colour::fromRGB(154, 148, 138);
                if (stepState.active)
                    colour = juce::Colour::fromRGB(28, 25, 22);
                if (stepState.skip)
                    colour = juce::Colour::fromRGB(166, 32, 28);
                if (isPlayhead)
                    colour = juce::Colour::fromRGB(52, 158, 58);

                g.setColour(colour);
                g.fillEllipse(x, y, cell, cell);
                g.setColour(juce::Colours::white.withAlpha(0.33f));
                g.fillEllipse(x + cell * 0.24f, y + cell * 0.18f, cell * 0.25f, cell * 0.25f);
            }
        }

        const auto rightWidth = bounds.getWidth() * 0.34f;
        const auto circleArea = juce::Rectangle<float>(
            bounds.getRight() - rightWidth,
            bounds.getY(),
            rightWidth,
            bounds.getHeight()).reduced(36.0f, 80.0f);
        g.setColour(juce::Colour::fromRGB(90, 84, 74));
        g.drawEllipse(circleArea, 1.5f);

        const auto centre = circleArea.getCentre();
        for (int i = 0; i < 32; ++i) {
            const auto angle = juce::MathConstants<float>::twoPi * static_cast<float>(i) / 32.0f - juce::MathConstants<float>::halfPi;
            const auto radius = circleArea.getWidth() * (0.18f + 0.32f * static_cast<float>(i % 3) / 2.0f);
            const auto pos = centre + juce::Point<float>(std::cos(angle), std::sin(angle)) * radius;
            g.setColour(i % 5 == 0 ? juce::Colour::fromRGB(54, 156, 62) : juce::Colour::fromRGB(128, 121, 112));
            g.fillEllipse(pos.x - 4.0f, pos.y - 4.0f, 8.0f, 8.0f);
        }
    }

private:
    void timerCallback() override
    {
        for (int i = 0; i < 4; ++i)
            engine_.tick(grid_);

        repaint();
    }

    benoqs::GridState grid_;
    benoqs::SequencerEngine engine_;
};

class MainWindow final : public juce::DocumentWindow {
public:
    MainWindow()
        : DocumentWindow(
            "BenoQs Sequencer",
            juce::Desktop::getInstance().getDefaultLookAndFeel().findColour(juce::ResizableWindow::backgroundColourId),
            DocumentWindow::allButtons)
    {
        setUsingNativeTitleBar(true);
        setContentOwned(new SequencerPanel(), true);
        centreWithSize(1100, 680);
        setVisible(true);
    }

    void closeButtonPressed() override
    {
        juce::JUCEApplication::getInstance()->systemRequestedQuit();
    }
};

class BenoQsApplication final : public juce::JUCEApplication {
public:
    const juce::String getApplicationName() override { return "BenoQs Sequencer"; }
    const juce::String getApplicationVersion() override { return "0.1.0"; }

    void initialise(const juce::String&) override
    {
        window_ = std::make_unique<MainWindow>();
    }

    void shutdown() override
    {
        window_.reset();
    }

private:
    std::unique_ptr<MainWindow> window_;
};

} // namespace

START_JUCE_APPLICATION(BenoQsApplication)
