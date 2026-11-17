//! Risograph: an FFGL effect for Resolume that turns the clip into a
//! two-ink riso print (separation, grain or halftone screen,
//! misregistration, paper). The work happens in `risograph.glsl`.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

/// Same order as `INKS` in risograph.glsl.
pub const INKS: &[&str] = &[
    "Medium Blue", "Bright Red", "Fluo Pink", "Fluo Orange", "Yellow", "Green",
    "Teal", "Blue", "Federal Blue", "Purple", "Burgundy", "Black",
];

/// Same order as `PAPERS` in risograph.glsl.
pub const PAPERS: &[&str] = &["Natural", "White", "Newsprint", "Kraft", "Grey"];

pub const SHADER: &str = include_str!("risograph.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxRi",
    name: "Risograph",
    short_name: c"Riso",
    description: c"Two-ink risograph print: separation, grain or halftone screen, misregistration and paper",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Ink 1", "ink1", "Inks", INKS, 0),
        Param::option("Ink 2", "ink2", "Inks", INKS, 1),
        Param::option("Paper", "paper", "Inks", PAPERS, 0),
        Param::option("Separation", "separation", "Tone", &["Duotone", "Best Fit", "Shadows + Mids"], 0),
        Param::float("Contrast", "contrast", "Tone", 0.25, 3.0, 1.3),
        Param::float("Brightness", "brightness", "Tone", -0.5, 0.5, 0.0),
        Param::float("Clean Highlights", "highlights", "Tone", 0.0, 0.8, 0.25),
        Param::float("Density", "density", "Tone", 0.0, 2.0, 1.0),
        Param::float("Ink 2 Strength", "ink2Strength", "Tone", 0.0, 1.0, 0.8),
        Param::option("Screen", "screenType", "Screen", &["Grain", "Halftone"], 0),
        Param::float("Dot Size", "dotSize", "Screen", 1.0, 12.0, 2.0),
        Param::float("Ink Grain", "grain", "Screen", 0.0, 1.0, 0.5),
        Param::float("Paper Texture", "paperTexture", "Screen", 0.0, 1.0, 0.5),
        Param::float("Offset X", "offsetX", "Registration", -40.0, 40.0, 8.0),
        Param::float("Offset Y", "offsetY", "Registration", -40.0, 40.0, 4.0),
        Param::float("Boil", "boil", "Registration", 0.0, 1.0, 0.0),
    ],
    passes: &[Pass::new(SHADER)],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_is_valid() {
        EFFECT.validate().unwrap();
    }

    #[test]
    fn shader_tables_match_option_lists() {
        assert!(SHADER.contains(&format!("vec3 INKS[{}]", INKS.len())));
        assert!(SHADER.contains(&format!("vec3 PAPERS[{}]", PAPERS.len())));
        assert!(SHADER.contains(&format!("clamp(ink1, 0, {})", INKS.len() - 1)));
        assert!(SHADER.contains(&format!("clamp(paper, 0, {})", PAPERS.len() - 1)));
        for name in INKS {
            assert!(SHADER.contains(&format!("// {name}")), "{name} missing from shader table");
        }
    }

    #[test]
    fn shader_uses_every_param() {
        for p in EFFECT.params {
            assert!(SHADER.contains(p.ident), "{} unused", p.ident);
        }
    }
}
