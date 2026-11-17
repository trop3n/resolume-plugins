//! Ripple Prism: Ripple distortion spreading from a movable center (circular, linear waves, droplet, standing wave).
//!
//! Ported from the Prism ISF shader `RipplePrism.fs` (prism-ffgl, Prism Lab). The
//! shader body is in `effect.glsl`; parameters keep their ISF names.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub const SHADER: &str = include_str!("effect.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"PrRi",
    name: "Ripple Prism",
    short_name: c"Ripple",
    description: c"Ripple distortion spreading from a movable center (circular, linear waves, droplet, standing wave)",
    about: c"Prism Lab / resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Pattern", "Pattern", "Ripple", &["Circular", "Linear Waves", "Droplet", "Standing Wave"], 0),
        Param::float("Amplitude", "Amplitude", "Ripple", 0.0, 0.2, 0.02),
        Param::float("Wavelength", "Wavelength", "Ripple", 0.01, 1.0, 0.25),
        Param::float("Speed", "Speed", "Ripple", -2.0, 2.0, 0.3),
        Param::float("Phase", "Phase", "Ripple", 0.0, 360.0, 0.0),
        Param::float("Falloff", "Falloff", "Ripple", 0.0, 10.0, 0.5),
        Param::float("Radius", "Radius", "Ripple", 0.05, 3.0, 1.0),
        Param::float("Direction", "Direction", "Ripple", 0.0, 360.0, 0.0),
        Param::float("Noise", "Noise", "Ripple", 0.0, 2.0, 0.0),
        Param::point("Center", "Center", "Ripple", [0.5, 0.5]),
        Param::float("Shading", "Shading", "Ripple", 0.0, 1.0, 0.3),
        Param::float("Mix", "Mix", "Ripple", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(SHADER)],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
