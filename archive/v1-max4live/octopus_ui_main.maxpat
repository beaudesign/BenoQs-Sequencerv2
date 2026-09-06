{
  "patcher" : {
    "fileversion" : 1,
    "appversion" : {
      "major" : 9,
      "minor" : 0,
      "revision" : 0,
      "architecture" : "x64"
    },
    "rect" : [ 0.0, 0.0, 800.0, 520.0 ],
    "bglocked" : 0,
    "defrect" : [ 0.0, 0.0, 800.0, 520.0 ],
    "openrect" : [ 50.0, 50.0, 850.0, 570.0 ],
    "openinpresentation" : 1,
    "default_fontsize" : 12.0,
    "default_fontface" : 0,
    "default_fontname" : "Arial",
    "gridonopen" : 0,
    "gridsize" : [ 15.0, 15.0 ],
    "gridsnaponopen" : 0,
    "objectsnaponopen" : 0,
    "statusbarvisible" : 2,
    "toolbarvisible" : 1,
    "boxanimatetime" : 200,
    "enablehscroll" : 1,
    "enablevscroll" : 1,
    "devicewidth" : 800.0,
    "boxes" : [
      {
        "box" : {
          "id" : "dict-1",
          "maxclass" : "dict",
          "name" : "octopus_state",
          "embed" : 1,
          "numinlets" : 1,
          "numoutlets" : 2,
          "patching_rect" : [ 20.0, 20.0, 150.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "transport-1",
          "maxclass" : "newobj",
          "text" : "transport @clocksource host",
          "numinlets" : 1,
          "numoutlets" : 9,
          "patching_rect" : [ 190.0, 20.0, 120.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-gettempo-1",
          "maxclass" : "message",
          "text" : "bang",
          "patching_rect" : [ 320.0, 20.0, 50.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "loadbang-1",
          "maxclass" : "loadbang",
          "patching_rect" : [ 320.0, 60.0, 30.0, 30.0 ]
        }
      },
      {
        "box" : {
          "id" : "qmetro-1",
          "maxclass" : "newobj",
          "text" : "qmetro 50",
          "numinlets" : 1,
          "numoutlets" : 1,
          "patching_rect" : [ 360.0, 60.0, 80.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "toggle-1",
          "maxclass" : "toggle",
          "patching_rect" : [ 450.0, 60.0, 20.0, 20.0 ]
        }
      },
      {
        "box" : {
          "id" : "expr-tps-1",
          "maxclass" : "expr",
          "text" : "($f1/60.)*192.",
          "numinlets" : 1,
          "numoutlets" : 1,
          "patching_rect" : [ 380.0, 20.0, 110.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "sig-tps-1",
          "maxclass" : "sig~",
          "numinlets" : 1,
          "numoutlets" : 1,
          "patching_rect" : [ 500.0, 20.0, 50.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "phasor-1",
          "maxclass" : "phasor~",
          "numinlets" : 2,
          "numoutlets" : 1,
          "patching_rect" : [ 560.0, 20.0, 70.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "gt-1",
          "maxclass" : ">~",
          "text" : ">~ 0.999",
          "numinlets" : 2,
          "numoutlets" : 1,
          "patching_rect" : [ 640.0, 20.0, 70.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "edge-1",
          "maxclass" : "edge~",
          "numinlets" : 1,
          "numoutlets" : 2,
          "patching_rect" : [ 720.0, 20.0, 60.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "gate-1",
          "maxclass" : "gate",
          "text" : "gate 1",
          "numinlets" : 2,
          "numoutlets" : 1,
          "patching_rect" : [ 720.0, 60.0, 60.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-tick-1",
          "maxclass" : "message",
          "text" : "tick",
          "patching_rect" : [ 720.0, 100.0, 45.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "print-tick-1",
          "maxclass" : "print",
          "text" : "octopus_tick",
          "patching_rect" : [ 720.0, 140.0, 120.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "js-data-1",
          "maxclass" : "js",
          "text" : "octopus_data.js",
          "numinlets" : 1,
          "numoutlets" : 1,
          "patching_rect" : [ 20.0, 60.0, 150.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "jsui-matrix-1",
          "maxclass" : "jsui",
          "filename" : "octopus_matrix_ui.js",
          "numinlets" : 1,
          "numoutlets" : 1,
          "patching_rect" : [ 20.0, 190.0, 520.0, 320.0 ],
          "presentation" : 1,
          "presentation_rect" : [ 0.0, 0.0, 560.0, 520.0 ]
        }
      },
      {
        "box" : {
          "id" : "js-ui-1",
          "maxclass" : "js",
          "text" : "octopus_ui.js",
          "numinlets" : 1,
          "numoutlets" : 2,
          "patching_rect" : [ 560.0, 190.0, 140.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "live-tab-mode-1",
          "maxclass" : "live.tab",
          "numinlets" : 1,
          "numoutlets" : 2,
          "patching_rect" : [ 560.0, 230.0, 220.0, 22.0 ],
          "presentation" : 1,
          "presentation_rect" : [ 580.0, 20.0, 200.0, 22.0 ],
          "parameter_enable" : 1,
          "items" : [ "PAGE", "TRACK", "STEP", "GRID" ]
        }
      },
      {
        "box" : {
          "id" : "live-tab-routing-1",
          "maxclass" : "live.tab",
          "numinlets" : 1,
          "numoutlets" : 2,
          "patching_rect" : [ 560.0, 260.0, 220.0, 22.0 ],
          "presentation" : 1,
          "presentation_rect" : [ 580.0, 50.0, 200.0, 22.0 ],
          "parameter_enable" : 1,
          "items" : [ "OCTOPUS_MCH", "FIXED_PER_TRACK" ]
        }
      },
      {
        "box" : {
          "id" : "live-tab-mix-1",
          "maxclass" : "live.tab",
          "numinlets" : 1,
          "numoutlets" : 2,
          "patching_rect" : [ 560.0, 350.0, 260.0, 22.0 ],
          "presentation" : 1,
          "presentation_rect" : [ 580.0, 80.0, 200.0, 22.0 ],
          "parameter_enable" : 1,
          "items" : [ "VEL", "PIT", "LEN", "STA", "POS", "DIR", "AMT", "GRV", "MCC", "MCH" ]
        }
      },
      {
        "box" : {
          "id" : "prepend-mix-index-1",
          "maxclass" : "newobj",
          "text" : "prepend mix_index",
          "numinlets" : 1,
          "numoutlets" : 1,
          "patching_rect" : [ 560.0, 380.0, 120.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "live-tab-scale-enable-1",
          "maxclass" : "live.tab",
          "numinlets" : 1,
          "numoutlets" : 2,
          "patching_rect" : [ 560.0, 410.0, 190.0, 22.0 ],
          "presentation" : 1,
          "presentation_rect" : [ 580.0, 110.0, 200.0, 22.0 ],
          "parameter_enable" : 1,
          "items" : [ "SCALE_OFF", "SCALE_ON" ]
        }
      },
      {
        "box" : {
          "id" : "sel-scale-enable-1",
          "maxclass" : "newobj",
          "text" : "sel 0 1",
          "numinlets" : 1,
          "numoutlets" : 3,
          "patching_rect" : [ 560.0, 440.0, 70.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-scale-off-1",
          "maxclass" : "message",
          "text" : "set_scale_enabled 0",
          "patching_rect" : [ 640.0, 440.0, 130.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-scale-on-1",
          "maxclass" : "message",
          "text" : "set_scale_enabled 1",
          "patching_rect" : [ 780.0, 440.0, 130.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "live-tab-scale-mode-1",
          "maxclass" : "live.tab",
          "numinlets" : 1,
          "numoutlets" : 2,
          "patching_rect" : [ 560.0, 470.0, 210.0, 22.0 ],
          "presentation" : 1,
          "presentation_rect" : [ 580.0, 140.0, 200.0, 22.0 ],
          "parameter_enable" : 1,
          "items" : [ "CHR", "MAJ", "MIN", "MOD" ]
        }
      },
      {
        "box" : {
          "id" : "sel-scale-mode-1",
          "maxclass" : "newobj",
          "text" : "sel 0 1 2 3",
          "numinlets" : 1,
          "numoutlets" : 5,
          "patching_rect" : [ 560.0, 500.0, 100.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-scale-chr-1",
          "maxclass" : "message",
          "text" : "set_scale chr",
          "patching_rect" : [ 670.0, 500.0, 95.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-scale-maj-1",
          "maxclass" : "message",
          "text" : "set_scale maj",
          "patching_rect" : [ 770.0, 500.0, 95.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-scale-min-1",
          "maxclass" : "message",
          "text" : "set_scale min",
          "patching_rect" : [ 870.0, 500.0, 95.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-scale-mod-1",
          "maxclass" : "message",
          "text" : "set_scale mod",
          "patching_rect" : [ 970.0, 500.0, 95.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "live-tab-transport-1",
          "maxclass" : "live.tab",
          "numinlets" : 1,
          "numoutlets" : 2,
          "patching_rect" : [ 560.0, 530.0, 120.0, 22.0 ],
          "presentation" : 1,
          "presentation_rect" : [ 580.0, 170.0, 200.0, 22.0 ],
          "parameter_enable" : 1,
          "items" : [ "STOP", "PLAY" ]
        }
      },
      {
        "box" : {
          "id" : "sel-transport-1",
          "maxclass" : "newobj",
          "text" : "sel 0 1",
          "numinlets" : 1,
          "numoutlets" : 3,
          "patching_rect" : [ 690.0, 530.0, 70.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "prepend-transport-1",
          "maxclass" : "newobj",
          "text" : "prepend transport",
          "numinlets" : 1,
          "numoutlets" : 1,
          "patching_rect" : [ 560.0, 560.0, 120.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-transport-stop-1",
          "maxclass" : "message",
          "text" : "transport_state 0",
          "patching_rect" : [ 770.0, 530.0, 120.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-transport-play-1",
          "maxclass" : "message",
          "text" : "transport_state 1",
          "patching_rect" : [ 900.0, 530.0, 120.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "sel-routing-1",
          "maxclass" : "newobj",
          "text" : "sel 0 1",
          "numinlets" : 1,
          "numoutlets" : 3,
          "patching_rect" : [ 560.0, 290.0, 70.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-routing-octo-1",
          "maxclass" : "message",
          "text" : "set_midi_routing_mode octopus",
          "patching_rect" : [ 640.0, 290.0, 200.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-routing-fixed-1",
          "maxclass" : "message",
          "text" : "set_midi_routing_mode fixed",
          "patching_rect" : [ 640.0, 320.0, 190.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-init-1",
          "maxclass" : "message",
          "text" : "ensure_state",
          "patching_rect" : [ 190.0, 60.0, 90.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-reset-1",
          "maxclass" : "message",
          "text" : "reset_state",
          "patching_rect" : [ 290.0, 60.0, 80.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "print-1",
          "maxclass" : "print",
          "text" : "octopus_data",
          "patching_rect" : [ 20.0, 100.0, 120.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "js-engine-1",
          "maxclass" : "js",
          "text" : "octopus_engine.js",
          "numinlets" : 1,
          "numoutlets" : 1,
          "patching_rect" : [ 190.0, 100.0, 150.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "print-engine-1",
          "maxclass" : "print",
          "text" : "octopus_engine",
          "patching_rect" : [ 190.0, 140.0, 140.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "prepend-ts-1",
          "maxclass" : "newobj",
          "text" : "prepend transport_state",
          "numinlets" : 1,
          "numoutlets" : 1,
          "patching_rect" : [ 350.0, 100.0, 160.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "midiin-1",
          "maxclass" : "newobj",
          "text" : "midiin",
          "numinlets" : 1,
          "numoutlets" : 1,
          "patching_rect" : [ 560.0, 20.0, 55.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "midiout-1",
          "maxclass" : "newobj",
          "text" : "midiout",
          "numinlets" : 1,
          "numoutlets" : 0,
          "patching_rect" : [ 560.0, 50.0, 60.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "route-engine-1",
          "maxclass" : "newobj",
          "text" : "route noteon noteoff cc",
          "numinlets" : 1,
          "numoutlets" : 4,
          "patching_rect" : [ 190.0, 170.0, 270.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "midiformat-noteon-1",
          "maxclass" : "newobj",
          "text" : "midiformat",
          "numinlets" : 6,
          "numoutlets" : 1,
          "patching_rect" : [ 190.0, 200.0, 80.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "midiformat-noteoff-1",
          "maxclass" : "newobj",
          "text" : "midiformat",
          "numinlets" : 6,
          "numoutlets" : 1,
          "patching_rect" : [ 280.0, 200.0, 80.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "midiformat-cc-1",
          "maxclass" : "newobj",
          "text" : "midiformat",
          "numinlets" : 6,
          "numoutlets" : 1,
          "patching_rect" : [ 370.0, 200.0, 80.0, 22.0 ]
        }
      },
      {
        "box" : {
          "id" : "msg-allnotesoff-1",
          "maxclass" : "message",
          "text" : "176 123 0",
          "patching_rect" : [ 460.0, 200.0, 70.0, 22.0 ]
        }
      }
    ],
    "lines" : [
      { "patchline" : { "source" : [ "loadbang-1", 0 ], "destination" : [ "msg-init-1", 0 ] } },
      { "patchline" : { "source" : [ "loadbang-1", 0 ], "destination" : [ "toggle-1", 0 ] } },
      { "patchline" : { "source" : [ "toggle-1", 0 ], "destination" : [ "qmetro-1", 0 ] } },
      { "patchline" : { "source" : [ "qmetro-1", 0 ], "destination" : [ "msg-gettempo-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-gettempo-1", 0 ], "destination" : [ "transport-1", 0 ] } },
      { "patchline" : { "source" : [ "transport-1", 6 ], "destination" : [ "gate-1", 1 ] } },
      { "patchline" : { "source" : [ "transport-1", 4 ], "destination" : [ "expr-tps-1", 0 ] } },
      { "patchline" : { "source" : [ "expr-tps-1", 0 ], "destination" : [ "sig-tps-1", 0 ] } },
      { "patchline" : { "source" : [ "sig-tps-1", 0 ], "destination" : [ "phasor-1", 0 ] } },
      { "patchline" : { "source" : [ "phasor-1", 0 ], "destination" : [ "gt-1", 0 ] } },
      { "patchline" : { "source" : [ "gt-1", 0 ], "destination" : [ "edge-1", 0 ] } },
      { "patchline" : { "source" : [ "edge-1", 0 ], "destination" : [ "gate-1", 0 ] } },
      { "patchline" : { "source" : [ "gate-1", 0 ], "destination" : [ "msg-tick-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-tick-1", 0 ], "destination" : [ "js-engine-1", 0 ] } },
      { "patchline" : { "source" : [ "transport-1", 6 ], "destination" : [ "prepend-ts-1", 0 ] } },
      { "patchline" : { "source" : [ "prepend-ts-1", 0 ], "destination" : [ "js-engine-1", 0 ] } },
      { "patchline" : { "source" : [ "js-engine-1", 0 ], "destination" : [ "route-engine-1", 0 ] } },
      { "patchline" : { "source" : [ "route-engine-1", 0 ], "destination" : [ "midiformat-noteon-1", 0 ] } },
      { "patchline" : { "source" : [ "route-engine-1", 1 ], "destination" : [ "midiformat-noteoff-1", 0 ] } },
      { "patchline" : { "source" : [ "route-engine-1", 2 ], "destination" : [ "midiformat-cc-1", 0 ] } },
      { "patchline" : { "source" : [ "midiformat-noteon-1", 0 ], "destination" : [ "midiout-1", 0 ] } },
      { "patchline" : { "source" : [ "midiformat-noteoff-1", 0 ], "destination" : [ "midiout-1", 0 ] } },
      { "patchline" : { "source" : [ "midiformat-cc-1", 0 ], "destination" : [ "midiout-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-init-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-reset-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "jsui-matrix-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "live-tab-mode-1", 0 ], "destination" : [ "js-ui-1", 0 ] } },
      { "patchline" : { "source" : [ "js-ui-1", 0 ], "destination" : [ "jsui-matrix-1", 0 ] } },
      { "patchline" : { "source" : [ "js-ui-1", 1 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "live-tab-mix-1", 0 ], "destination" : [ "prepend-mix-index-1", 0 ] } },
      { "patchline" : { "source" : [ "prepend-mix-index-1", 0 ], "destination" : [ "js-ui-1", 0 ] } },
      { "patchline" : { "source" : [ "live-tab-scale-enable-1", 0 ], "destination" : [ "sel-scale-enable-1", 0 ] } },
      { "patchline" : { "source" : [ "sel-scale-enable-1", 0 ], "destination" : [ "msg-scale-off-1", 0 ] } },
      { "patchline" : { "source" : [ "sel-scale-enable-1", 1 ], "destination" : [ "msg-scale-on-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-scale-off-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-scale-on-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "live-tab-scale-mode-1", 0 ], "destination" : [ "sel-scale-mode-1", 0 ] } },
      { "patchline" : { "source" : [ "sel-scale-mode-1", 0 ], "destination" : [ "msg-scale-chr-1", 0 ] } },
      { "patchline" : { "source" : [ "sel-scale-mode-1", 1 ], "destination" : [ "msg-scale-maj-1", 0 ] } },
      { "patchline" : { "source" : [ "sel-scale-mode-1", 2 ], "destination" : [ "msg-scale-min-1", 0 ] } },
      { "patchline" : { "source" : [ "sel-scale-mode-1", 3 ], "destination" : [ "msg-scale-mod-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-scale-chr-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-scale-maj-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-scale-min-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-scale-mod-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "live-tab-transport-1", 0 ], "destination" : [ "sel-transport-1", 0 ] } },
      { "patchline" : { "source" : [ "live-tab-transport-1", 0 ], "destination" : [ "prepend-transport-1", 0 ] } },
      { "patchline" : { "source" : [ "prepend-transport-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "sel-transport-1", 0 ], "destination" : [ "msg-transport-stop-1", 0 ] } },
      { "patchline" : { "source" : [ "sel-transport-1", 1 ], "destination" : [ "msg-transport-play-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-transport-stop-1", 0 ], "destination" : [ "js-engine-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-transport-play-1", 0 ], "destination" : [ "js-engine-1", 0 ] } },
      { "patchline" : { "source" : [ "live-tab-routing-1", 0 ], "destination" : [ "sel-routing-1", 0 ] } },
      { "patchline" : { "source" : [ "sel-routing-1", 0 ], "destination" : [ "msg-routing-octo-1", 0 ] } },
      { "patchline" : { "source" : [ "sel-routing-1", 1 ], "destination" : [ "msg-routing-fixed-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-routing-octo-1", 0 ], "destination" : [ "js-data-1", 0 ] } },
      { "patchline" : { "source" : [ "msg-routing-fixed-1", 0 ], "destination" : [ "js-data-1", 0 ] } }
    ]
  }
}

