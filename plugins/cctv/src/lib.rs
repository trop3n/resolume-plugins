//! CCTV: an FFGL effect for Resolume that makes the clip look like a cheap
//! security camera feed.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxCC",
    name: "CCTV",
    short_name: c"CCTV",
    description: c"Cheap security camera: soft, tinted, noisy, with jitter, scan lines and a REC dot",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::float("Softness", "softness", "Image", 0.0, 3.0, 1.0),
        Param::float("Chroma Shift", "chromaShift", "Image", 0.0, 10.0, 2.0),
        Param::float("Saturation", "saturation", "Grade", 0.0, 1.5, 0.6),
        Param::float("Contrast", "contrast", "Grade", 0.5, 2.0, 1.15),
        Param::colour("Cast", "castColour", "Grade", [1.0, 0.92, 0.72]),
        Param::float("Noise", "noise", "Signal", 0.0, 1.0, 0.25),
        Param::float("Scan Lines", "scanLines", "Signal", 0.0, 1.0, 0.35),
        Param::float("Jitter", "jitter", "Signal", 0.0, 1.0, 0.2),
        Param::float("Vignette", "vignette", "Signal", 0.0, 1.0, 0.5),
        Param::bool("REC Dot", "recDot", "Signal", true),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
