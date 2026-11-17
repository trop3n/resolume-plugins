//! Pixel Sort: an FFGL effect for Resolume that sorts runs of pixels along
//! each row or column, inside a brightness mask, on the GPU.
//!
//! Passes: `mask.glsl` marks sortable pixels, `sort.glsl` runs `Iterations`
//! odd-even transposition steps (four per draw), `output.glsl` mixes with
//! the original.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub const MASK: &str = concat!(include_str!("common.glsl"), include_str!("mask.glsl"));
pub const SORT: &str = concat!(include_str!("common.glsl"), include_str!("sort.glsl"));
pub const OUTPUT: &str = include_str!("output.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxPS",
    name: "Pixel Sort",
    short_name: c"PxSort",
    description: c"Sorts runs of pixels along rows or columns inside a brightness mask",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::option("Direction", "direction", "Sort", &["Down", "Up", "Right", "Left"], 0),
        Param::option("Sort By", "sortBy", "Sort", &["Brightness", "Hue", "Saturation"], 0),
        // Cost is linear in this: about 0.07 ms per step at 1080p on an RTX 3060 Ti.
        Param::int("Iterations", "iterations", "Sort", 0, 1024, 128),
        Param::float("Low", "lowThreshold", "Mask", 0.0, 1.0, 0.25),
        Param::float("High", "highThreshold", "Mask", 0.0, 1.0, 1.0),
        Param::bool("Invert Mask", "invertMask", "Mask", false),
        Param::float("Break Up", "breakUp", "Mask", 0.0, 1.0, 0.0),
        Param::bool("Show Mask", "showMask", "Mask", false),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    // sort.glsl does STEPS = 4 sort steps per run.
    passes: &[Pass::new(MASK), Pass::repeated_by(SORT, "iterations", 4), Pass::new(OUTPUT)],
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
    fn shaders_use_every_param() {
        let all = [MASK, SORT, OUTPUT].concat();
        for p in EFFECT.params {
            assert!(all.contains(p.ident), "{} unused", p.ident);
        }
    }

    #[test]
    fn steps_per_run_matches_shader() {
        let pass = &EFFECT.passes[1];
        assert!(SORT.contains(&format!("const int STEPS = {};", pass.steps_per_run)));
    }

    /// The sort shader's swap rule, on the CPU: a full run of iterations
    /// must sort a span and leave masked pixels where they are.
    #[test]
    fn odd_even_rule_sorts_spans() {
        // None = masked.
        let mut line: Vec<Option<f32>> =
            vec![Some(0.9), Some(0.1), Some(0.5), None, Some(0.7), Some(0.2), Some(0.8), Some(0.3)];
        let n = line.len();
        for ascending in [true, false] {
            for it in 0..n {
                let prev = line.clone();
                for i in 0..n {
                    let Some(km) = prev[i] else { continue };
                    let lower = (i + it) % 2 == 0;
                    let j = if lower { i + 1 } else { i.wrapping_sub(1) };
                    let Some(Some(ko)) = prev.get(j).copied() else { continue };
                    let out_of_order = if lower == ascending { km > ko } else { km < ko };
                    if out_of_order {
                        line[i] = Some(ko);
                    }
                }
            }
            let want: Vec<Option<f32>> = if ascending {
                vec![Some(0.1), Some(0.5), Some(0.9), None, Some(0.2), Some(0.3), Some(0.7), Some(0.8)]
            } else {
                vec![Some(0.9), Some(0.5), Some(0.1), None, Some(0.8), Some(0.7), Some(0.3), Some(0.2)]
            };
            assert_eq!(line, want, "ascending = {ascending}");
        }
    }
}
