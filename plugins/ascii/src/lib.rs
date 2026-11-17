//! ASCII: an FFGL effect for Resolume that redraws the clip as text, from
//! a built-in 5x7 bitmap font.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub const SHADER: &str = include_str!("effect.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxAs",
    name: "ASCII",
    short_name: c"ASCII",
    description: c"Redraws the clip as text characters or shade blocks",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Charset", "charset", "Text", &["Ramp", "Blocks"], 0),
        Param::float("Char Size", "charSize", "Text", 6.0, 64.0, 16.0),
        Param::bool("Invert", "invert", "Text", false),
        Param::float("Contrast", "contrast", "Tone", 0.25, 3.0, 1.3),
        Param::float("Brightness", "brightness", "Tone", -0.5, 0.5, 0.0),
        Param::bool("Image Colour", "imageColour", "Colours", false),
        Param::colour("Text", "textColour", "Colours", [0.08, 0.08, 0.08]),
        Param::colour("Background", "backgroundColour", "Colours", [0.93, 0.93, 0.91]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(SHADER)],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);

#[cfg(test)]
mod tests {
    use super::SHADER;

    /// The glyph table, parsed back out of the shader.
    fn glyphs() -> Vec<[u8; 7]> {
        let start = SHADER.find("int[70](").expect("glyph table") + 8;
        let body = &SHADER[start..start + SHADER[start..].find(");").unwrap()];
        let nums: Vec<u8> = body
            .lines()
            .flat_map(|l| l.split("//").next().unwrap().split(','))
            .filter_map(|n| n.trim().parse().ok())
            .collect();
        assert_eq!(nums.len(), 70);
        nums.chunks(7).map(|c| c.try_into().unwrap()).collect()
    }

    #[test]
    fn ramp_runs_sparse_to_dense() {
        let counts: Vec<u32> = glyphs().iter().map(|g| g.iter().map(|r| r.count_ones()).sum()).collect();
        assert!(counts.windows(2).all(|w| w[0] <= w[1]), "{counts:?}");
        assert!(glyphs().iter().flatten().all(|&r| r < 32), "rows are 5 bits wide");
    }
}
