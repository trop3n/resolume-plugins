//! Dot Matrix: an FFGL effect for Resolume that redraws the clip as a grid
//! of dots sized by brightness, printer or LED style.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxDM",
    name: "Dot Matrix",
    short_name: c"DotMtx",
    description: c"Grid of dots sized by brightness, printer or LED style",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Style", "style", "Dots", &["Print", "LED"], 0),
        Param::option("Shape", "shape", "Dots", &["Round", "Square"], 0),
        Param::float("Cell Size", "cellSize", "Dots", 3.0, 48.0, 12.0),
        Param::float("Min Size", "minSize", "Dots", 0.0, 1.0, 0.15),
        Param::float("Glow", "glow", "Dots", 0.0, 1.0, 0.5),
        Param::float("Contrast", "contrast", "Tone", 0.25, 3.0, 1.2),
        Param::float("Brightness", "brightness", "Tone", -0.5, 0.5, 0.0),
        Param::bool("Image Colour", "imageColour", "Colours", false),
        Param::colour("Dot", "dotColour", "Colours", [0.15, 0.15, 0.17]),
        Param::colour("Background", "backgroundColour", "Colours", [0.88, 0.88, 0.87]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
