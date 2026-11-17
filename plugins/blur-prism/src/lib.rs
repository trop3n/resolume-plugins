//! Blur Prism: Directional motion-style blur inside a box/ellipse area or a luma/alpha/color key.
//!
//! Ported from the Prism ISF shader `BlurPrism.fs` (prism-ffgl, Prism Lab). The
//! shader body is in `effect.glsl`; parameters keep their ISF names.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub const SHADER: &str = include_str!("effect.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"PrBl",
    name: "Blur Prism",
    short_name: c"Blur",
    description: c"Directional motion-style blur inside a box/ellipse area or a luma/alpha/color key",
    about: c"Prism Lab / resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Mode", "Mode", "Blur", &["Area", "Key"], 0),
        Param::option("Shape", "Shape", "Blur", &["Box", "Ellipse"], 1),
        Param::point("Center", "Center", "Blur", [0.5, 0.5]),
        Param::float("Width", "Width", "Blur", 0.0, 1.5, 0.4),
        Param::float("Height", "Height", "Blur", 0.0, 1.5, 0.3),
        Param::float("Rotation", "Rotation", "Blur", 0.0, 360.0, 0.0),
        Param::option("Key", "Key", "Blur", &["Luma", "Alpha", "Color"], 0),
        Param::colour("Key Color", "KeyColor", "Blur", [0.0, 1.0, 0.0]),
        Param::float("Threshold", "Threshold", "Blur", 0.0, 1.0, 0.5),
        Param::bool("Invert", "Invert", "Blur", false),
        Param::float("Softness", "Softness", "Blur", 0.0, 0.5, 0.03),
        Param::float("Amount", "Amount", "Blur", 0.0, 1.0, 1.0),
        Param::float("Length", "Length", "Blur", 0.0, 6.0, 1.0),
        Param::float("Direction", "Direction", "Blur", 0.0, 360.0, 0.0),
        Param::float("Quality", "Quality", "Blur", 2.0, 64.0, 16.0),
        Param::bool("Trail", "Trail", "Blur", true),
        Param::bool("Animate", "Animate", "Blur", false),
        Param::float("Pulse Speed", "PulseSpeed", "Blur", 0.0, 10.0, 1.0),
        Param::float("Pulse Depth", "PulseDepth", "Blur", 0.0, 1.0, 0.6),
        Param::float("Spin", "Spin", "Blur", -5.0, 5.0, 0.0),
        Param::float("Mix", "Mix", "Blur", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(SHADER)],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
