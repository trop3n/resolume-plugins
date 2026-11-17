//! Tracking: an FFGL effect for Resolume that draws a surveillance-style
//! tracking HUD around whatever is moving.
//!
//! There's no face or body detection: the box follows motion. Passes, with
//! the persistent buffers they write:
//! 1. `energy.glsl` → `energy`: motion, held for a while
//! 2. `remember.glsl` → `prevInput`: this frame, for next frame's motion
//! 3. `tracker.glsl` → `track`: box and confidence, in two texels
//! 4. `output.glsl`: desaturated picture with the HUD

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub const OUTPUT: &str = include_str!("output.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxTr",
    name: "Tracking",
    short_name: c"Track",
    description: c"Surveillance tracking HUD that follows whatever is moving",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::float("Sensitivity", "sensitivity", "Tracker", 0.0, 1.0, 0.7),
        Param::float("Memory", "memory", "Tracker", 0.0, 0.99, 0.85),
        Param::float("Smoothing", "smoothing", "Tracker", 0.0, 1.0, 0.7),
        Param::float("Padding", "padding", "Tracker", 0.0, 0.3, 0.04),
        Param::option("Box Style", "boxStyle", "HUD", &["Box", "Corners"], 0),
        Param::bool("Head Box", "headBox", "HUD", true),
        Param::bool("Crosshair", "crosshair", "HUD", true),
        Param::bool("Labels", "labels", "HUD", true),
        Param::float("Line Width", "lineWidth", "HUD", 1.0, 6.0, 2.0),
        Param::colour("Colour", "boxColour", "HUD", [0.95, 0.12, 0.10]),
        Param::float("Desaturate", "desaturate", "Picture", 0.0, 1.0, 0.85),
        Param::bool("Show Motion", "showMotion", "Picture", false),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[
        Pass::new(include_str!("energy.glsl")).storing("energy"),
        Pass::new(include_str!("remember.glsl")).storing("prevInput"),
        Pass::new(include_str!("tracker.glsl")).storing("track"),
        Pass::new(OUTPUT),
    ],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);

#[cfg(test)]
mod tests {
    use super::OUTPUT;

    #[test]
    fn font_table_has_every_glyph() {
        let start = OUTPUT.find("int[119](").expect("font table") + 9;
        let body = &OUTPUT[start..start + OUTPUT[start..].find(");").unwrap()];
        let rows: Vec<u8> = body
            .lines()
            .flat_map(|l| l.split("//").next().unwrap().split(','))
            .filter_map(|n| n.trim().parse().ok())
            .collect();
        assert_eq!(rows.len(), 17 * 7);
        assert!(rows.iter().all(|&r| r < 32), "rows are 5 bits wide");
        // Every glyph but the space draws something.
        for (i, g) in rows.chunks(7).enumerate().take(16) {
            assert!(g.iter().any(|&r| r != 0), "glyph {i} is empty");
        }
    }
}
