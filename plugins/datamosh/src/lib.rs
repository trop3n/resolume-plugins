//! Datamosh: an FFGL effect for Resolume that fakes a video codec losing
//! its keyframes. Block motion from the live clip moves the old picture
//! around, so previous content smears and bleeds into the new motion.
//!
//! Passes, with the persistent buffers they write:
//! 1. `motion.glsl` → `motion`: block motion vectors, previous → current input
//! 2. `mosh.glsl` → `mosh`: last mosh moved by the motion, plus residual
//! 3. `remember.glsl` → `prevInput`: this frame's clip, for next frame
//! 4. `output.glsl`: shows the mosh

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxDa",
    name: "Datamosh",
    short_name: c"Mosh",
    description: c"Fake codec datamosh: old frames smeared along the clip's block motion",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::bool("Keyframe", "keyframe", "Keyframes", false),
        Param::float("Key Interval", "keyInterval", "Keyframes", 0.0, 10.0, 0.0),
        Param::float("Refresh", "refresh", "Keyframes", 0.0, 1.0, 0.05),
        Param::float("Block Size", "blockSize", "Motion", 4.0, 64.0, 16.0),
        Param::float("Motion Gain", "motionGain", "Motion", 0.0, 4.0, 1.0),
        Param::float("Residual", "residualAmount", "Motion", 0.0, 1.0, 0.85),
        Param::float("Cut Threshold", "cutThreshold", "Motion", 0.0, 1.0, 0.15),
        Param::bool("Freeze Motion", "freezeMotion", "Motion", false),
        Param::bool("Show Motion", "showMotion", "Output", false),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[
        Pass::new(include_str!("motion.glsl")).storing("motion"),
        Pass::new(include_str!("mosh.glsl")).storing("mosh"),
        Pass::new(include_str!("remember.glsl")).storing("prevInput"),
        Pass::new(include_str!("output.glsl")),
    ],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
