//! Edge Detect: an FFGL effect for Resolume that draws the clip's edges as
//! outlines, scan-line stripes or neon.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxED",
    name: "Edge Detect",
    short_name: c"Edges",
    description: c"Sobel edges as outlines, scan-line stripes or neon",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Mode", "mode", "Edges", &["Lines", "Scan Lines", "Neon"], 0),
        Param::float("Strength", "strength", "Edges", 0.5, 8.0, 2.5),
        Param::float("Threshold", "threshold", "Edges", 0.0, 1.0, 0.15),
        Param::float("Line Width", "lineWidth", "Edges", 0.5, 4.0, 1.0),
        Param::float("Line Spacing", "lineSpacing", "Scan Lines", 2.0, 20.0, 6.0),
        Param::colour("Ink", "inkColour", "Colours", [0.05, 0.05, 0.05]),
        Param::colour("Paper", "paperColour", "Colours", [0.94, 0.94, 0.92]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
