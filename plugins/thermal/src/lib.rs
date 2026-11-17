//! Thermal: an FFGL effect for Resolume that maps brightness or warmth
//! through a thermal-camera false-colour palette.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxTM",
    name: "Thermal",
    short_name: c"Therm",
    description: c"Thermal-camera false colour: iron, rainbow, white hot or black hot",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Palette", "palette", "Palette", &["Iron", "Rainbow", "White Hot", "Black Hot"], 0),
        Param::option("Source", "source", "Palette", &["Brightness", "Warmth"], 0),
        Param::float("Low", "low", "Range", 0.0, 1.0, 0.05),
        Param::float("High", "high", "Range", 0.0, 1.0, 0.95),
        Param::float("Contrast", "contrast", "Range", 0.5, 3.0, 1.2),
        Param::float("Blur", "blur", "Camera", 0.0, 1.0, 0.3),
        Param::float("Noise", "noise", "Camera", 0.0, 1.0, 0.3),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
