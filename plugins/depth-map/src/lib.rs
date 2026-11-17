//! Depth Map: an FFGL effect for Resolume that fakes a depth camera.
//!
//! A single camera image has no depth, so this assumes a fixed camera: it
//! learns the empty scene, treats whatever differs from it (people) as
//! near, and gives the rest of the scene a simple fake depth.
//!
//! Passes, with the persistent buffers they write:
//! 1. `background.glsl` → `background`: the learned empty scene
//! 2. `matte.glsl` → `matte`: soft foreground matte, smoothed over time
//! 3. `depth.glsl`: depth per pixel (scene depth, foreground pushed near)
//! 4. `output.glsl`: smoothing and display style

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxDp",
    name: "Depth Map",
    short_name: c"Depth",
    description: c"Fake depth camera for a fixed shot: learned background, people pushed near",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::bool("Learn Background", "learnBackground", "Background", false),
        Param::float("Adapt", "adapt", "Background", 0.0, 1.0, 0.1),
        Param::bool("Auto Relearn", "autoRelearn", "Background", true),
        Param::float("Threshold", "threshold", "Subject", 0.0, 1.0, 0.12),
        Param::float("Softness", "softness", "Subject", 0.01, 0.5, 0.15),
        Param::float("Hold", "hold", "Subject", 0.0, 1.0, 0.3),
        Param::option("Scene", "sceneMode", "Depth", &["Floor", "Brightness", "Flat"], 0),
        Param::float("Scene Amount", "sceneAmount", "Depth", 0.0, 1.0, 0.45),
        Param::float("Near", "nearLevel", "Depth", 0.0, 1.0, 0.95),
        Param::float("Far", "farLevel", "Depth", 0.0, 1.0, 0.08),
        Param::float("Relief", "relief", "Depth", 0.0, 1.0, 0.3),
        Param::float("Smoothing", "smoothing", "Output", 0.0, 1.0, 0.4),
        Param::option("Style", "style", "Output", &["Grayscale", "Inverted", "Turbo", "Contours", "Matte"], 0),
        Param::int("Bands", "bands", "Output", 4, 64, 16),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[
        Pass::new(include_str!("background.glsl")).storing("background"),
        Pass::new(include_str!("matte.glsl")).storing("matte"),
        Pass::new(include_str!("depth.glsl")),
        Pass::new(include_str!("output.glsl")),
    ],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
