//! Halftone: an FFGL effect for Resolume that redraws the clip as a
//! rotated screen of dots, lines or squares, in one ink or CMYK.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxHt",
    name: "Halftone",
    short_name: c"Halftn",
    description: c"Rotated halftone screen of dots, lines or squares, mono or CMYK",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Mode", "mode", "Screen", &["Mono", "CMYK"], 0),
        Param::option("Shape", "shape", "Screen", &["Circle", "Line", "Square"], 0),
        Param::float("Cell Size", "cellSize", "Screen", 3.0, 60.0, 10.0),
        Param::float("Angle", "angle", "Screen", 0.0, 90.0, 45.0),
        Param::float("Contrast", "contrast", "Tone", 0.25, 3.0, 1.1),
        Param::float("Brightness", "brightness", "Tone", -0.5, 0.5, 0.0),
        Param::colour("Ink", "inkColour", "Colours", [0.05, 0.05, 0.05]),
        Param::colour("Paper", "paperColour", "Colours", [0.96, 0.95, 0.92]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
