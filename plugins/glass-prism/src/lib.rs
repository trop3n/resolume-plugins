//! Glass Prism: View the video through patterned glass (reeded, fluted, frosted, liquid, bubble, ripple, hammered, lenticular).
//!
//! Ported from the Prism ISF shader `GlassPrism.fs` (prism-ffgl, Prism Lab). The
//! shader body is in `effect.glsl`; parameters keep their ISF names.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub const SHADER: &str = include_str!("effect.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"PrGl",
    name: "Glass Prism",
    short_name: c"Glass",
    description: c"View the video through patterned glass (reeded, fluted, frosted, liquid, bubble, ripple, hammered, lenticular)",
    about: c"Prism Lab / resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Pattern", "Pattern", "Glass", &["Reeded", "Fluted", "Frost", "Liquid", "Bubble", "Ripple", "Hammered", "Lenticular"], 0),
        Param::float("Scale", "Scale", "Glass", 2.0, 120.0, 24.0),
        Param::float("Depth", "Depth", "Glass", 0.0, 1.0, 0.35),
        Param::float("Rotation", "Rotation", "Glass", 0.0, 360.0, 0.0),
        Param::float("Dispersion", "Dispersion", "Glass", 0.0, 1.0, 0.0),
        Param::float("Frost", "Frost", "Glass", 0.0, 1.0, 0.0),
        Param::float("Sheen", "Sheen", "Glass", 0.0, 1.0, 0.15),
        Param::colour("Tint", "Tint", "Glass", [1.0, 1.0, 1.0]),
        Param::option("Animate", "Animate", "Glass", &["Off", "Ripples", "Waves"], 0),
        Param::float("Anim Amount", "AnimAmount", "Glass", 0.0, 2.0, 0.5),
        Param::float("Anim Speed", "AnimSpeed", "Glass", -4.0, 4.0, 1.0),
        Param::float("Anim Freq", "AnimFreq", "Glass", 0.5, 30.0, 4.0),
        Param::float("Anim Dir", "AnimDir", "Glass", 0.0, 360.0, 0.0),
        Param::point("Center", "Center", "Glass", [0.5, 0.5]),
        Param::float("Mix", "Mix", "Glass", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(SHADER)],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
