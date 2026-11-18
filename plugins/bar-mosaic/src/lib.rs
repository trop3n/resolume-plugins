//! Bar Mosaic: an FFGL effect for Resolume that redraws the clip as rows of
//! rounded, lit bars. Each bar is one colour (the clip's average under it).
//! The bar edges sit on a regular grid in a warped coordinate, so they line
//! up into columns that bend with the image's brightness and with an
//! optional lens curve, and stretch into long streaks where the brightness
//! falls away.
//!
//! Passes: blur the luma across, blur it down, build the warp field, take a
//! running max of it along the bars (Streak Length doubling steps, so folds
//! in the warp become streaks), then draw the bars.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub const SHADER: &str = include_str!("effect.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxBM",
    name: "Bar Mosaic",
    short_name: c"BarMosaic",
    description: c"Rows of lit bars whose edges bend with the image",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::float("Bar Width", "barWidth", "Bars", 2.0, 80.0, 12.0),
        Param::float("Row Height", "rowHeight", "Bars", 2.0, 60.0, 13.0),
        Param::float("Gap", "gap", "Bars", 0.0, 8.0, 2.0),
        Param::float("Roundness", "roundness", "Bars", 0.0, 1.0, 0.5),
        Param::option(
            "Orientation",
            "orientation",
            "Bars",
            &["Horizontal", "Vertical"],
            0,
        ),
        Param::float("Luma Warp", "lumaWarp", "Warp", -60.0, 60.0, 12.0),
        Param::float("Warp Blur", "warpBlur", "Warp", 0.0, 80.0, 14.0),
        Param::int("Streak Length", "streakSteps", "Warp", 0, 10, 8),
        Param::float("Lens", "lens", "Warp", -1.0, 1.0, 0.0),
        Param::float("Row Wave", "rowWave", "Warp", 0.0, 40.0, 0.0),
        Param::float("Wave Freq", "waveFreq", "Warp", 0.0, 20.0, 3.0),
        Param::float("Wave Speed", "waveSpeed", "Warp", -5.0, 5.0, 0.3),
        Param::float("Scroll", "scroll", "Warp", -5.0, 5.0, 0.0),
        Param::float("Row Jitter", "rowJitter", "Warp", 0.0, 1.0, 0.0),
        Param::float("Ramp", "ramp", "Look", 0.0, 1.0, 0.5),
        Param::float("Cap Glow", "capGlow", "Look", 0.0, 2.0, 0.8),
        Param::float("Cap Floor", "capFloor", "Look", 0.0, 1.0, 0.3),
        Param::float("Gamma", "gamma", "Look", 0.3, 3.0, 1.0),
        Param::colour("Tint", "tint", "Look", [1.0, 1.0, 1.0]),
        Param::colour("Gap Colour", "gapColour", "Look", [0.01, 0.01, 0.03]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[
        Pass::new(SHADER),
        Pass::new(SHADER),
        Pass::new(SHADER),
        Pass::repeated(SHADER, "streakSteps"),
        Pass::new(SHADER),
    ],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
