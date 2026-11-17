//! X-Ray: an FFGL effect for Resolume that inverts the clip into a cool,
//! high-contrast x-ray with glowing edges, grain and scan lines.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxXR",
    name: "X-Ray",
    short_name: c"XRay",
    description: c"Inverted, high-contrast x-ray look with glowing edges, grain and scan lines",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::float("Contrast", "contrast", "Tone", 0.5, 4.0, 1.6),
        Param::float("Brightness", "brightness", "Tone", -0.5, 0.5, -0.05),
        Param::float("Edge Glow", "edgeGlow", "Tone", 0.0, 2.0, 0.6),
        Param::float("Grain", "grain", "Texture", 0.0, 1.0, 0.2),
        Param::float("Scan Lines", "scanLines", "Texture", 0.0, 1.0, 0.3),
        Param::colour("Tint", "tint", "Colour", [0.78, 0.90, 1.0]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
