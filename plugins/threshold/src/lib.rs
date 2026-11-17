//! Threshold: an FFGL effect for Resolume that cuts the clip to two
//! colours at a brightness level, with optional photocopy grit.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxTh",
    name: "Threshold",
    short_name: c"Thresh",
    description: c"Hard two-colour cut at a brightness level, with optional grit",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::float("Level", "level", "Cut", 0.0, 1.0, 0.5),
        Param::float("Softness", "softness", "Cut", 0.0, 0.25, 0.0),
        Param::bool("Invert", "invert", "Cut", false),
        Param::float("Noise", "noise", "Grit", 0.0, 0.6, 0.1),
        Param::float("Grain Size", "grainSize", "Grit", 1.0, 8.0, 1.5),
        Param::float("Boil", "boil", "Grit", 0.0, 1.0, 0.0),
        Param::colour("Dark", "darkColour", "Colours", [0.0, 0.0, 0.0]),
        Param::colour("Light", "lightColour", "Colours", [1.0, 1.0, 1.0]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
