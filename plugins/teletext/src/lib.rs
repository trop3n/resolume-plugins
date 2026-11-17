//! Teletext: an FFGL effect for Resolume that redraws the clip in
//! Ceefax-style 2x3 mosaic blocks and the eight teletext colours.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxTx",
    name: "Teletext",
    short_name: c"Ttext",
    description: c"Ceefax-style mosaic blocks in the eight teletext colours",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Mode", "mode", "Mosaic", &["Free", "One Colour per Cell"], 0),
        Param::int("Columns", "columns", "Mosaic", 20, 120, 40),
        Param::bool("Separated", "separated", "Mosaic", false),
        Param::float("Dither", "dither", "Mosaic", 0.0, 1.0, 0.3),
        Param::float("Contrast", "contrast", "Tone", 0.25, 3.0, 1.2),
        Param::float("Saturation", "saturation", "Tone", 0.0, 3.0, 1.6),
        Param::float("Scan Lines", "scanLines", "Tone", 0.0, 1.0, 0.3),
        Param::colour("Background", "backgroundColour", "Colours", [0.0, 0.0, 0.0]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
