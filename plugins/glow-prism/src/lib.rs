//! Glow Prism: Bright parts of the video bloom into a soft glow.
//!
//! Ported from the Prism ISF shader `GlowPrism.fs` (prism-ffgl, Prism Lab). The
//! shader body is in `effect.glsl`; parameters keep their ISF names.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub const SHADER: &str = include_str!("effect.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"PrGw",
    name: "Glow Prism",
    short_name: c"Glow",
    description: c"Bright parts of the video bloom into a soft glow",
    about: c"Prism Lab / resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::float("Threshold", "Threshold", "Glow", 0.0, 1.0, 0.7),
        Param::float("Softness", "Softness", "Glow", 0.0, 0.5, 0.1),
        Param::float("Size", "Size", "Glow", 0.0, 1.0, 0.6),
        Param::float("Strength", "Strength", "Glow", 0.0, 5.0, 1.0),
        Param::colour("Tint", "Tint", "Glow", [1.0, 1.0, 1.0]),
        Param::float("Mix", "Mix", "Glow", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(SHADER), Pass::new(SHADER), Pass::new(SHADER)],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
