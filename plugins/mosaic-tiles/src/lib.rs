//! Mosaic Tiles: an FFGL effect for Resolume that splits the clip into a
//! grid of tiles which keep flipping between the image and a solid colour.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub static EFFECT: EffectDef = EffectDef {
    id: *b"FxMT",
    name: "Mosaic Tiles",
    short_name: c"Mosaic",
    description: c"Grid of tiles flipping between the image and a solid colour",
    about: c"resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::int("Columns", "columns", "Tiles", 2, 40, 8),
        Param::float("Solid Amount", "solidAmount", "Tiles", 0.0, 1.0, 0.4),
        Param::float("Flip Speed", "flipSpeed", "Tiles", 0.0, 10.0, 1.0),
        Param::float("Seed", "seed", "Tiles", 0.0, 100.0, 0.0),
        Param::option("Image", "imageStyle", "Image", &["Original", "Threshold", "Mono"], 1),
        Param::float("Level", "level", "Image", 0.0, 1.0, 0.5),
        Param::colour("Tile", "tileColour", "Colours", [0.86, 0.08, 0.08]),
        Param::float("Grout", "grout", "Colours", 0.0, 0.3, 0.04),
        Param::colour("Grout Colour", "groutColour", "Colours", [0.05, 0.05, 0.05]),
        Param::float("Mix", "mixAmount", "Output", 0.0, 1.0, 1.0),
    ],
    passes: &[Pass::new(include_str!("effect.glsl"))],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
