//! Dither: an FFGL effect for Resolume that dithers the clip down to a few
//! levels with a Bayer or noise pattern, at a chosen pixel size.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxDi",
    name: "Dither",
    short_name: c"Dither",
    description: c"Ordered (Bayer) or noise dithering to a few levels, two-colour or per channel",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Pattern", "pattern", "Dither", &["Bayer 2x2", "Bayer 4x4", "Bayer 8x8", "Noise"], 1),
        Param::float("Pixel Size", "pixelSize", "Dither", 1.0, 16.0, 2.0),
        Param::int("Levels", "levels", "Dither", 2, 8, 2),
        Param::option("Colour", "colourMode", "Dither", &["Two Colour", "Per Channel"], 0),
        Param::float("Contrast", "contrast", "Tone", 0.25, 3.0, 1.2),
        Param::float("Brightness", "brightness", "Tone", -0.5, 0.5, 0.0),
        Param::colour("Dark", "darkColour", "Colours", [0.0, 0.0, 0.0]),
        Param::colour("Light", "lightColour", "Colours", [1.0, 1.0, 1.0]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
