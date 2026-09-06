cmake_minimum_required(VERSION 3.22)

project(BenoQsSequencer VERSION 0.1.0 LANGUAGES CXX)

option(BENOQS_BUILD_TESTS "Build the standalone core tests" ON)
option(BENOQS_BUILD_JUCE_APP "Build the optional JUCE desktop prototype" OFF)

add_library(benoqs_core
    src/core/Domain.cpp
    src/core/Scale.cpp
    src/core/SequencerEngine.cpp
)

target_include_directories(benoqs_core
    PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}/src
)

target_compile_features(benoqs_core PUBLIC cxx_std_20)

if(MSVC)
    target_compile_options(benoqs_core PRIVATE /W4)
else()
    target_compile_options(benoqs_core PRIVATE -Wall -Wextra -Wpedantic)
endif()

if(BENOQS_BUILD_TESTS)
    enable_testing()

    add_executable(benoqs_core_tests
        tests/core_tests.cpp
    )

    target_link_libraries(benoqs_core_tests PRIVATE benoqs_core)
    target_compile_features(benoqs_core_tests PRIVATE cxx_std_20)

    add_test(NAME benoqs_core_tests COMMAND benoqs_core_tests)
endif()

if(BENOQS_BUILD_JUCE_APP)
    include(FetchContent)

    set(BENOQS_JUCE_TAG "8.0.12" CACHE STRING "JUCE tag to fetch when building the optional app")

    FetchContent_Declare(
        JUCE
        GIT_REPOSITORY https://github.com/juce-framework/JUCE.git
        GIT_TAG ${BENOQS_JUCE_TAG}
        GIT_SHALLOW TRUE
    )

    FetchContent_MakeAvailable(JUCE)

    juce_add_gui_app(BenoQsSequencerApp
        VERSION ${PROJECT_VERSION}
        COMPANY_NAME "BenoQs"
        PRODUCT_NAME "BenoQs Sequencer"
    )

    target_sources(BenoQsSequencerApp PRIVATE
        src/juce/Main.cpp
    )

    target_link_libraries(BenoQsSequencerApp
        PRIVATE
            benoqs_core
            juce::juce_gui_basics
            juce::juce_gui_extra
        PUBLIC
            juce::juce_recommended_config_flags
            juce::juce_recommended_lto_flags
            juce::juce_recommended_warning_flags
    )

    target_compile_definitions(BenoQsSequencerApp
        PRIVATE
            JUCE_WEB_BROWSER=0
            JUCE_USE_CURL=0
    )
endif()
