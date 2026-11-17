//! Solarize: an FFGL effect for Resolume that folds tones past a
//! threshold (the Sabattier effect), optionally through a rainbow.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxSo",
    name: "Solarize",
    short_name: c"Solar",
    description: c"Folds tones past a threshold, in rainbow, colour or mono",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Palette", "palette", "Solarize", &["Rainbow", "Colour", "Mono"], 0),
        Param::float("Threshold", "threshold", "Solarize", 0.05, 0.95, 0.5),
        Param::float("Amount", "amount", "Solarize", 0.0, 1.0, 1.0),
        Param::float("Cycles", "cycles", "Rainbow", 0.25, 4.0, 1.0),
        Param::float("Hue Shift", "hueShift", "Rainbow", 0.0, 1.0, 0.0),
        Param::float("Speed", "speed", "Rainbow", -1.0, 1.0, 0.0),
        Param::float("Saturation", "saturation", "Rainbow", 0.0, 1.0, 0.9),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
