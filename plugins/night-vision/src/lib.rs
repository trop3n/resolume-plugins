//! Night Vision: an FFGL effect for Resolume that turns the clip into an
//! amplified green phosphor image with noise, bloom and a tube vignette.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxNV",
    name: "Night Vision",
    short_name: c"NightV",
    description: c"Amplified green phosphor image with noise, bloom, scan lines and vignette",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::float("Gain", "gain", "Image", 0.5, 6.0, 2.0),
        Param::float("Bloom", "bloom", "Image", 0.0, 1.0, 0.4),
        Param::float("Noise", "noise", "Texture", 0.0, 1.0, 0.3),
        Param::float("Scan Lines", "scanLines", "Texture", 0.0, 1.0, 0.3),
        Param::float("Flicker", "flicker", "Texture", 0.0, 1.0, 0.2),
        Param::float("Vignette", "vignette", "Texture", 0.0, 1.0, 0.6),
        Param::colour("Phosphor", "phosphor", "Colour", [0.30, 1.0, 0.35]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
