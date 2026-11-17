//! Data Prism: FUI-style HUD overlay (brackets, lock frame, measure arcs, labels, circle chain, anchors) around a box you place on screen.
//!
//! Ported from the Prism ISF shader `DataPrism.fs` (prism-ffgl, Prism Lab). The
//! shader body is in `effect.glsl`; parameters keep their ISF names.

use ffgl_fx::{Cache, EffectDef, Param, Pass};

pub const SHADER: &str = include_str!("effect.glsl");

pub static EFFECT: EffectDef = EffectDef {
    id: *b"PrDa",
    name: "Data Prism",
    short_name: c"Data",
    description: c"FUI-style HUD overlay (brackets, lock frame, measure arcs, labels, circle chain, anchors) around a box you place on screen",
    about: c"Prism Lab / resolume-plugins (GPL-3.0)",
    version: (0, 1),
    params: &[
        Param::point("Center", "Center", "Box", [0.5, 0.5]),
        Param::float("Width", "Width", "Box", 0.02, 1.0, 0.3),
        Param::float("Height", "Height", "Box", 0.02, 1.0, 0.45),
        Param::colour("Color", "Color", "Box", [0.55, 0.85, 1.0]),
        Param::colour("Accent", "Accent", "Box", [1.0, 0.45, 0.15]),
        Param::float("Line Weight", "LineWeight", "Box", 0.5, 8.0, 2.0),
        Param::float("Text Size", "TextSize", "Box", 0.0, 3.0, 1.0),
        Param::float("Speed", "Speed", "Box", -4.0, 4.0, 1.0),
        Param::float("Seed", "Seed", "Box", 0.0, 100.0, 0.0),
        Param::float("Opacity", "Opacity", "Box", 0.0, 1.0, 1.0),
        Param::bool("Brackets", "Brackets", "Brackets", true),
        Param::float("Brackets Opacity", "BracketsOpacity", "Brackets", 0.0, 1.0, 1.0),
        Param::float("Bracket Length", "BracketLength", "Brackets", 0.0, 0.5, 0.18),
        Param::float("Bracket Gap", "BracketGap", "Brackets", 0.0, 0.5, 0.08),
        Param::bool("Labels", "Labels", "Labels", true),
        Param::float("Labels Opacity", "LabelsOpacity", "Labels", 0.0, 1.0, 1.0),
        Param::float("Label Count", "LabelCount", "Labels", 0.0, 8.0, 4.0),
        Param::float("Label Offset", "LabelOffset", "Labels", 0.0, 1.0, 0.35),
        Param::bool("Anchors", "Anchors", "Anchors", false),
        Param::float("Anchors Opacity", "AnchorsOpacity", "Anchors", 0.0, 1.0, 1.0),
        Param::float("Anchor Count", "AnchorCount", "Anchors", 0.0, 32.0, 10.0),
        Param::float("Anchor Size", "AnchorSize", "Anchors", 0.0, 4.0, 1.0),
        Param::bool("Arcs", "Arcs", "Arcs", false),
        Param::float("Arcs Opacity", "ArcsOpacity", "Arcs", 0.0, 1.0, 1.0),
        Param::float("Arc Radius", "ArcRadius", "Arcs", 0.0, 3.0, 1.4),
        Param::float("Arc Sweep", "ArcSweep", "Arcs", 0.0, 360.0, 240.0),
        Param::float("Arc Ticks", "ArcTicks", "Arcs", 0.0, 200.0, 48.0),
        Param::bool("Lock", "Lock", "Lock", false),
        Param::float("Lock Opacity", "LockOpacity", "Lock", 0.0, 1.0, 1.0),
        Param::float("Lock Size", "LockSize", "Lock", 0.0, 2.0, 0.75),
        Param::bool("Chain", "Chain", "Chain", false),
        Param::float("Chain Opacity", "ChainOpacity", "Chain", 0.0, 1.0, 1.0),
        Param::float("Chain Count", "ChainCount", "Chain", 3.0, 120.0, 28.0),
        Param::float("Chain Radius", "ChainRadius", "Chain", 0.0, 3.0, 1.25),
    ],
    passes: &[Pass::new(SHADER)],
    cache: Cache::new(),
};

ffgl_fx::export_effect!(EFFECT);
ffgl_fx::effect_tests!(EFFECT);
