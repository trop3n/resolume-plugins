//! Effect parameters: what a plugin declares, and the flat list Resolume sees.
//!
//! Each [`Param`] becomes a GLSL uniform named by its `ident`. Resolume sends
//! sliders as 0..1; [`HostParam`] maps that onto the real range, so shaders
//! only see real values.

use std::ffi::CString;

use crate::ffi;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// Slider over `min..max`. GLSL `float`.
    Float { min: f32, max: f32, default: f32 },
    /// Slider over whole numbers. GLSL `int`.
    Int { min: i32, max: i32, default: i32 },
    /// Dropdown. GLSL `int` holding the element index.
    Option { elements: &'static [&'static str], default: usize },
    /// Toggle. GLSL `bool`.
    Bool { default: bool },
    /// Colour picker (three host params: R, G, B). GLSL `vec3`.
    Colour { default: [f32; 3] },
    /// Position in 0..1 (two host params of FFGL's X and Y position types,
    /// like ffgl-isf's `point2D`). GLSL `vec2`.
    Point { default: [f32; 2] },
}

#[derive(Clone, Copy, Debug)]
pub struct Param {
    /// Label in Resolume.
    pub name: &'static str,
    /// GLSL uniform name.
    pub ident: &'static str,
    /// Heading Resolume groups the param under.
    pub group: &'static str,
    pub kind: Kind,
}

impl Param {
    pub const fn float(name: &'static str, ident: &'static str, group: &'static str, min: f32, max: f32, default: f32) -> Self {
        Self { name, ident, group, kind: Kind::Float { min, max, default } }
    }

    pub const fn int(name: &'static str, ident: &'static str, group: &'static str, min: i32, max: i32, default: i32) -> Self {
        Self { name, ident, group, kind: Kind::Int { min, max, default } }
    }

    pub const fn option(
        name: &'static str,
        ident: &'static str,
        group: &'static str,
        elements: &'static [&'static str],
        default: usize,
    ) -> Self {
        Self { name, ident, group, kind: Kind::Option { elements, default } }
    }

    pub const fn bool(name: &'static str, ident: &'static str, group: &'static str, default: bool) -> Self {
        Self { name, ident, group, kind: Kind::Bool { default } }
    }

    pub const fn colour(name: &'static str, ident: &'static str, group: &'static str, default: [f32; 3]) -> Self {
        Self { name, ident, group, kind: Kind::Colour { default } }
    }

    pub const fn point(name: &'static str, ident: &'static str, group: &'static str, default: [f32; 2]) -> Self {
        Self { name, ident, group, kind: Kind::Point { default } }
    }

    pub fn glsl_type(&self) -> &'static str {
        match self.kind {
            Kind::Float { .. } => "float",
            Kind::Int { .. } | Kind::Option { .. } => "int",
            Kind::Bool { .. } => "bool",
            Kind::Colour { .. } => "vec3",
            Kind::Point { .. } => "vec2",
        }
    }
}

#[derive(Debug)]
pub enum HostKind {
    Float { min: f32, max: f32 },
    Int { min: i32, max: i32 },
    Option { elements: Vec<CString> },
    Bool,
    /// One 0..1 component of a multi-part control: a colour channel
    /// (Resolume groups consecutive R, G, B) or a position axis (X, Y).
    Channel { ffgl_type: u32 },
}

/// One parameter as Resolume sees it. A colour param becomes three of these.
#[derive(Debug)]
pub struct HostParam {
    pub name: CString,
    pub group: String,
    pub kind: HostKind,
    /// Default in real units (element index for options, 0/1 for bools).
    pub default: f32,
}

fn cstring(s: &str) -> CString {
    CString::new(s).expect("parameter strings must not contain nul bytes")
}

impl HostParam {
    pub fn ffgl_type(&self) -> u32 {
        match self.kind {
            HostKind::Float { .. } | HostKind::Int { .. } => ffi::FF_TYPE_STANDARD,
            HostKind::Option { .. } => ffi::FF_TYPE_OPTION,
            HostKind::Bool => ffi::FF_TYPE_BOOLEAN,
            HostKind::Channel { ffgl_type } => ffgl_type,
        }
    }

    pub fn host_default(&self) -> f32 {
        self.to_host(self.default)
    }

    pub fn to_host(&self, real: f32) -> f32 {
        match self.kind {
            HostKind::Float { min, max } => ((real - min) / (max - min)).clamp(0.0, 1.0),
            HostKind::Int { min, max } => ((real - min as f32) / (max - min) as f32).clamp(0.0, 1.0),
            HostKind::Option { .. } | HostKind::Bool | HostKind::Channel { .. } => real,
        }
    }

    pub fn from_host(&self, host: f32) -> f32 {
        let host = host.clamp(0.0, f32::MAX);
        match &self.kind {
            HostKind::Float { min, max } => min + host.min(1.0) * (max - min),
            HostKind::Int { min, max } => (*min as f32 + host.min(1.0) * (max - min) as f32).round(),
            HostKind::Option { elements } => host.round().min((elements.len() - 1) as f32),
            HostKind::Bool => if host >= 0.5 { 1.0 } else { 0.0 },
            HostKind::Channel { .. } => host.min(1.0),
        }
    }

    /// Upper end of the range the host sends (lower end is always 0).
    pub fn host_max(&self) -> f32 {
        match &self.kind {
            HostKind::Option { elements } => (elements.len() - 1) as f32,
            _ => 1.0,
        }
    }

    pub fn elements(&self) -> &[CString] {
        match &self.kind {
            HostKind::Option { elements } => elements,
            _ => &[],
        }
    }

    /// Text Resolume shows next to the control.
    pub fn display(&self, real: f32) -> String {
        match &self.kind {
            HostKind::Option { elements } => {
                elements.get(real as usize).map(|e| e.to_string_lossy().into_owned()).unwrap_or_default()
            }
            HostKind::Bool => (if real >= 0.5 { "On" } else { "Off" }).to_owned(),
            HostKind::Int { .. } => format!("{}", real as i32),
            HostKind::Float { .. } | HostKind::Channel { .. } => format!("{real:.2}"),
        }
    }
}

/// Flatten an effect's params into host params. Returns the list and, for
/// each effect param, the index of its first host param.
pub fn expand(params: &[Param]) -> (Vec<HostParam>, Vec<usize>) {
    let mut host = Vec::new();
    let mut first = Vec::with_capacity(params.len());
    for p in params {
        first.push(host.len());
        let group = p.group.to_owned();
        match p.kind {
            Kind::Float { min, max, default } => {
                host.push(HostParam { name: cstring(p.name), group, kind: HostKind::Float { min, max }, default })
            }
            Kind::Int { min, max, default } => host.push(HostParam {
                name: cstring(p.name),
                group,
                kind: HostKind::Int { min, max },
                default: default as f32,
            }),
            Kind::Option { elements, default } => host.push(HostParam {
                name: cstring(p.name),
                group,
                kind: HostKind::Option { elements: elements.iter().map(|e| cstring(e)).collect() },
                default: default as f32,
            }),
            Kind::Bool { default } => host.push(HostParam {
                name: cstring(p.name),
                group,
                kind: HostKind::Bool,
                default: if default { 1.0 } else { 0.0 },
            }),
            Kind::Colour { default } => {
                let channels = [
                    (p.name.to_owned(), ffi::FF_TYPE_RED),
                    (format!("{} G", p.name), ffi::FF_TYPE_GREEN),
                    (format!("{} B", p.name), ffi::FF_TYPE_BLUE),
                ];
                for (i, (name, ffgl_type)) in channels.into_iter().enumerate() {
                    host.push(HostParam {
                        name: cstring(&name),
                        group: group.clone(),
                        kind: HostKind::Channel { ffgl_type },
                        default: default[i],
                    });
                }
            }
            Kind::Point { default } => {
                let axes = [(format!("{} X", p.name), ffi::FF_TYPE_XPOS), (format!("{} Y", p.name), ffi::FF_TYPE_YPOS)];
                for (i, (name, ffgl_type)) in axes.into_iter().enumerate() {
                    host.push(HostParam {
                        name: cstring(&name),
                        group: group.clone(),
                        kind: HostKind::Channel { ffgl_type },
                        default: default[i],
                    });
                }
            }
        }
    }
    (host, first)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARAMS: &[Param] = &[
        Param::float("Amount", "amount", "Main", -1.0, 3.0, 1.0),
        Param::int("Steps", "steps", "Main", 0, 100, 25),
        Param::option("Mode", "mode", "Main", &["A", "B", "C"], 2),
        Param::bool("Flip", "flip", "Main", true),
        Param::colour("Tint", "tint", "Colour", [0.1, 0.2, 0.3]),
        Param::point("Center", "center", "Position", [0.25, 0.75]),
    ];

    #[test]
    fn point_expands_to_x_and_y_positions() {
        let (host, first) = expand(PARAMS);
        assert_eq!(first[5], 7);
        assert_eq!(host.len(), 9);
        assert_eq!(host[7].name.to_str().unwrap(), "Center X");
        assert_eq!((host[7].ffgl_type(), host[8].ffgl_type()), (ffi::FF_TYPE_XPOS, ffi::FF_TYPE_YPOS));
        assert_eq!((host[7].host_default(), host[8].host_default()), (0.25, 0.75));
        assert_eq!(PARAMS[5].glsl_type(), "vec2");
    }

    #[test]
    fn colour_expands_to_three_channels() {
        let (host, first) = expand(PARAMS);
        assert_eq!(host.len(), 9);
        assert_eq!(first, vec![0, 1, 2, 3, 4, 7]);
        let types: Vec<u32> = host[4..7].iter().map(HostParam::ffgl_type).collect();
        assert_eq!(types, vec![ffi::FF_TYPE_RED, ffi::FF_TYPE_GREEN, ffi::FF_TYPE_BLUE]);
        assert_eq!(host[5].name.to_str().unwrap(), "Tint G");
        assert_eq!(host[6].default, 0.3);
    }

    #[test]
    fn host_mapping_round_trips_defaults() {
        let (host, _) = expand(PARAMS);
        for p in &host {
            assert!((p.from_host(p.host_default()) - p.default).abs() < 1e-5, "{:?}", p.name);
        }
        assert_eq!(host[0].host_default(), 0.5);
        assert_eq!(host[1].host_default(), 0.25);
        assert_eq!(host[2].host_default(), 2.0);
    }

    #[test]
    fn from_host_clamps_and_rounds() {
        let (host, _) = expand(PARAMS);
        assert_eq!(host[0].from_host(2.0), 3.0);
        assert_eq!(host[1].from_host(0.333), 33.0);
        assert_eq!(host[2].from_host(9.0), 2.0);
        assert_eq!(host[2].from_host(-4.0), 0.0);
        assert_eq!(host[3].from_host(0.4), 0.0);
    }

    #[test]
    fn display_text() {
        let (host, _) = expand(PARAMS);
        assert_eq!(host[0].display(1.234), "1.23");
        assert_eq!(host[1].display(33.0), "33");
        assert_eq!(host[2].display(1.0), "B");
        assert_eq!(host[3].display(1.0), "On");
    }
}
