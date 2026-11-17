//! Fluid Prism: Smoke-like fluid distortion that swirls the video out of an emitter, with chromatic dispersion (in the style of the Hiss effect from Control).
//!
//! Ported from the Prism ISF shader `FluidPrism.fs` (prism-ffgl, Prism Lab). The
//! shader body is in `effect.glsl`; parameters keep their ISF names.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub const SHADER: &str = include_str!("effect.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"PrFl",
    name: "Fluid Prism",
    short_name: c"Fluid",
    description: c"Smoke-like fluid distortion that swirls the video out of an emitter, with chromatic dispersion (in the style of the Hiss effect from Control)",
    about: c"Prism Lab / resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::point("Center", "Center", "Fluid", [0.5, 0.6]),
        Param::float("Radius", "Radius", "Fluid", 0.0, 0.5, 0.08),
        Param::float("Emit", "Emit", "Fluid", 0.0, 1.0, 0.6),
        Param::float("Direction", "Direction", "Fluid", 0.0, 360.0, 90.0),
        Param::float("Buoyancy", "Buoyancy", "Fluid", 0.0, 1.0, 0.25),
        Param::float("Turbulence", "Turbulence", "Fluid", 0.0, 2.0, 0.6),
        Param::float("Scale", "Scale", "Fluid", 0.5, 20.0, 4.0),
        Param::float("Speed", "Speed", "Fluid", 0.0, 4.0, 1.0),
        Param::float("Strength", "Strength", "Fluid", 0.0, 2.0, 1.0),
        Param::float("Fade", "Fade", "Fluid", 0.0, 4.0, 0.6),
        Param::float("Recover", "Recover", "Fluid", 0.0, 4.0, 0.3),
        Param::float("Dispersion", "Dispersion", "Fluid", 0.0, 1.0, 0.35),
        Param::float("Haze", "Haze", "Fluid", 0.0, 1.0, 0.15),
        Param::float("Mix", "Mix", "Fluid", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(SHADER).storing("field"), Pass::new(SHADER)],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
