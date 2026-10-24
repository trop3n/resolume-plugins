//! Parameters exposed to Resolume.
//!
//! Resolume sends standard sliders as 0..1. Each spec maps that onto a
//! real range, so the rest of the code only sees real values.

use std::ffi::CStr;

use osci_visualizer::VisualiserSettings;

use crate::ffi;
use crate::scene::ShapeKind;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// Slider; `min..max` is the real range the 0..1 slider maps onto.
    Standard { min: f32, max: f32 },
    /// Dropdown. Element `i` has value `i as f32`.
    Option { elements: &'static [&'static CStr] },
    /// One channel of a colour picker (Resolume groups R, G, B params).
    Colour { ffgl_type: u32 },
}

pub struct ParamSpec {
    pub name: &'static CStr,
    pub group: &'static str,
    pub kind: Kind,
    /// Default in the parameter's real units (or element index for options).
    pub default: f32,
}

impl ParamSpec {
    pub fn ffgl_type(&self) -> u32 {
        match self.kind {
            Kind::Standard { .. } => ffi::FF_TYPE_STANDARD,
            Kind::Option { .. } => ffi::FF_TYPE_OPTION,
            Kind::Colour { ffgl_type } => ffgl_type,
        }
    }

    /// Default as the host sees it (0..1 for sliders, element value for options).
    pub fn host_default(&self) -> f32 {
        self.to_host(self.default)
    }

    pub fn to_host(&self, real: f32) -> f32 {
        match self.kind {
            Kind::Standard { min, max } => ((real - min) / (max - min)).clamp(0.0, 1.0),
            Kind::Option { .. } | Kind::Colour { .. } => real,
        }
    }

    pub fn from_host(&self, host: f32) -> f32 {
        match self.kind {
            Kind::Standard { min, max } => min + host.clamp(0.0, 1.0) * (max - min),
            Kind::Option { elements } => host.round().clamp(0.0, (elements.len() - 1) as f32),
            Kind::Colour { .. } => host.clamp(0.0, 1.0),
        }
    }

    pub fn elements(&self) -> &'static [&'static CStr] {
        match self.kind {
            Kind::Option { elements } => elements,
            _ => &[],
        }
    }
}

// Indices into PARAMS. Keep in the same order as the table.
pub const SHAPE: usize = 0;
pub const SIZE: usize = 1;
pub const SPIN: usize = 2;
pub const FOCUS: usize = 3;
pub const INTENSITY: usize = 4;
pub const PERSISTENCE: usize = 5;
pub const GLOW: usize = 6;
pub const SCATTER: usize = 7;
pub const EXPOSURE: usize = 8;
pub const RED: usize = 9;
pub const GREEN: usize = 10;
pub const BLUE: usize = 11;

pub const SHAPE_NAMES: &[&CStr] = &[c"Circle", c"Square", c"Triangle", c"Star", c"Lissajous"];

/// Visualiser ranges/defaults follow `osci_visualizer::VisualiserSettings`.
pub static PARAMS: [ParamSpec; 12] = [
    ParamSpec { name: c"Shape", group: "Shape", kind: Kind::Option { elements: SHAPE_NAMES }, default: 0.0 },
    ParamSpec { name: c"Size", group: "Shape", kind: Kind::Standard { min: 0.05, max: 1.0 }, default: 0.7 },
    ParamSpec { name: c"Spin", group: "Shape", kind: Kind::Standard { min: -1.0, max: 1.0 }, default: 0.0 },
    ParamSpec { name: c"Focus", group: "Beam", kind: Kind::Standard { min: 0.001, max: 0.02 }, default: 0.004 },
    ParamSpec { name: c"Intensity", group: "Beam", kind: Kind::Standard { min: 0.1, max: 5.0 }, default: 1.0 },
    ParamSpec { name: c"Persistence", group: "Beam", kind: Kind::Standard { min: 0.0, max: 1.0 }, default: 0.5 },
    ParamSpec { name: c"Glow", group: "Beam", kind: Kind::Standard { min: 0.0, max: 2.0 }, default: 0.6 },
    ParamSpec { name: c"Scatter", group: "Beam", kind: Kind::Standard { min: 0.0, max: 2.0 }, default: 0.4 },
    ParamSpec { name: c"Exposure", group: "Beam", kind: Kind::Standard { min: 0.5, max: 5.0 }, default: 1.5 },
    ParamSpec { name: c"Colour", group: "Colour", kind: Kind::Colour { ffgl_type: ffi::FF_TYPE_RED }, default: 0.2 },
    ParamSpec { name: c"Green", group: "Colour", kind: Kind::Colour { ffgl_type: ffi::FF_TYPE_GREEN }, default: 1.0 },
    ParamSpec { name: c"Blue", group: "Colour", kind: Kind::Colour { ffgl_type: ffi::FF_TYPE_BLUE }, default: 0.3 },
];

/// Current values of one plugin instance, in real units.
#[derive(Clone, Debug, PartialEq)]
pub struct ParamValues {
    values: [f32; PARAMS.len()],
}

impl Default for ParamValues {
    fn default() -> Self {
        let mut values = [0.0; PARAMS.len()];
        for (v, spec) in values.iter_mut().zip(PARAMS.iter()) {
            *v = spec.default;
        }
        Self { values }
    }
}

impl ParamValues {
    pub fn get(&self, index: usize) -> f32 {
        self.values[index]
    }

    /// Value as the host sees it. `None` for an unknown index.
    pub fn get_host(&self, index: usize) -> Option<f32> {
        Some(PARAMS.get(index)?.to_host(self.values[index]))
    }

    /// Returns false for an unknown index or a non-finite value.
    pub fn set_host(&mut self, index: usize, host: f32) -> bool {
        match PARAMS.get(index) {
            Some(spec) if host.is_finite() => {
                self.values[index] = spec.from_host(host);
                true
            }
            _ => false,
        }
    }

    /// Text Resolume shows next to the slider.
    pub fn display(&self, index: usize) -> Option<String> {
        let spec = PARAMS.get(index)?;
        let v = self.values[index];
        Some(match spec.kind {
            Kind::Option { elements } => elements[v as usize].to_string_lossy().into_owned(),
            _ if index == FOCUS => format!("{v:.4}"),
            _ => format!("{v:.2}"),
        })
    }

    pub fn shape(&self) -> ShapeKind {
        ShapeKind::from_index(self.values[SHAPE] as usize)
    }

    pub fn visualiser_settings(&self) -> VisualiserSettings {
        let colour = [self.values[RED], self.values[GREEN], self.values[BLUE]];
        VisualiserSettings {
            focus: self.values[FOCUS],
            intensity: self.values[INTENSITY],
            persistence: self.values[PERSISTENCE],
            glow_amount: self.values[GLOW],
            scatter_amount: self.values[SCATTER],
            exposure: self.values[EXPOSURE],
            color: colour,
            afterglow_color: colour,
            ..VisualiserSettings::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_constants_match_table() {
        let expect = [
            (SHAPE, "Shape"), (SIZE, "Size"), (SPIN, "Spin"), (FOCUS, "Focus"),
            (INTENSITY, "Intensity"), (PERSISTENCE, "Persistence"), (GLOW, "Glow"),
            (SCATTER, "Scatter"), (EXPOSURE, "Exposure"), (RED, "Colour"),
            (GREEN, "Green"), (BLUE, "Blue"),
        ];
        for (i, name) in expect {
            assert_eq!(PARAMS[i].name.to_str().unwrap(), name);
        }
    }

    #[test]
    fn defaults_fit_their_ranges() {
        for spec in &PARAMS {
            let h = spec.host_default();
            assert!((0.0..=1.0).contains(&h) || matches!(spec.kind, Kind::Option { .. }), "{:?}", spec.name);
            assert!((spec.from_host(h) - spec.default).abs() < 1e-5, "{:?}", spec.name);
        }
    }

    #[test]
    fn colour_params_are_consecutive_rgb() {
        assert_eq!(PARAMS[RED].ffgl_type(), ffi::FF_TYPE_RED);
        assert_eq!(PARAMS[GREEN].ffgl_type(), ffi::FF_TYPE_GREEN);
        assert_eq!(PARAMS[BLUE].ffgl_type(), ffi::FF_TYPE_BLUE);
    }

    #[test]
    fn slider_maps_to_real_range() {
        let mut p = ParamValues::default();
        assert!(p.set_host(FOCUS, 1.0));
        assert!((p.get(FOCUS) - 0.02).abs() < 1e-6);
        assert!(p.set_host(SPIN, 0.5));
        assert!(p.get(SPIN).abs() < 1e-6);
        assert!(p.set_host(SIZE, 7.0)); // out of range is clamped
        assert_eq!(p.get(SIZE), 1.0);
    }

    #[test]
    fn option_rounds_and_clamps() {
        let mut p = ParamValues::default();
        p.set_host(SHAPE, 2.4);
        assert_eq!(p.shape(), ShapeKind::Triangle);
        p.set_host(SHAPE, 99.0);
        assert_eq!(p.shape(), ShapeKind::Lissajous);
        assert_eq!(p.display(SHAPE).unwrap(), "Lissajous");
    }

    #[test]
    fn rejects_bad_input() {
        let mut p = ParamValues::default();
        assert!(!p.set_host(PARAMS.len(), 0.5));
        assert!(!p.set_host(SIZE, f32::NAN));
        assert_eq!(p.get_host(PARAMS.len()), None);
    }

    #[test]
    fn settings_use_param_values() {
        let mut p = ParamValues::default();
        p.set_host(GLOW, 1.0);
        let s = p.visualiser_settings();
        assert_eq!(s.glow_amount, 2.0);
        assert_eq!(s.color, [0.2, 1.0, 0.3]);
    }
}
