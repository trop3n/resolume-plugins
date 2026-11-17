//! Retro LCD: an FFGL effect for Resolume that redraws the clip on a
//! low-res, two-tone phone LCD with dither and a pixel grid.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxLC",
    name: "Retro LCD",
    short_name: c"RetLCD",
    description: c"Low-res two-tone phone LCD with ordered dither and a pixel grid",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::float("Pixel Size", "pixelSize", "Screen", 2.0, 24.0, 6.0),
        Param::int("Levels", "levels", "Screen", 2, 4, 2),
        Param::bool("Dither", "dither", "Screen", true),
        Param::float("Grid", "grid", "Screen", 0.0, 1.0, 0.5),
        Param::float("Contrast", "contrast", "Tone", 0.25, 3.0, 1.3),
        Param::float("Brightness", "brightness", "Tone", -0.5, 0.5, 0.0),
        Param::colour("Ink", "inkColour", "Colours", [0.09, 0.16, 0.27]),
        Param::colour("Screen", "screenColour", "Colours", [0.20, 0.46, 0.38]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
