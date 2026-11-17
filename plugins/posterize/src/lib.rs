//! Posterize: an FFGL effect for Resolume that reduces the clip to a few
//! tone levels, on a three-colour gradient or per channel.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxPo",
    name: "Posterize",
    short_name: c"Poster",
    description: c"Reduces the clip to a few tone levels, on a colour gradient or per channel",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Mode", "mode", "Tone", &["Gradient", "Per Channel"], 0),
        Param::int("Levels", "levels", "Tone", 2, 16, 4),
        Param::float("Dither", "dither", "Tone", 0.0, 1.0, 0.0),
        Param::float("Contrast", "contrast", "Tone", 0.25, 3.0, 1.2),
        Param::float("Brightness", "brightness", "Tone", -0.5, 0.5, 0.0),
        Param::colour("Shadows", "shadowColour", "Gradient", [0.04, 0.05, 0.40]),
        Param::colour("Midtones", "midColour", "Gradient", [0.10, 0.22, 0.95]),
        Param::colour("Highlights", "highlightColour", "Gradient", [0.95, 0.15, 0.20]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
