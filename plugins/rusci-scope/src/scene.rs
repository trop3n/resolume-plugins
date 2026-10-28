//! Turns the selected shape into one frame of X/Y beam samples.
//!
//! Shapes are traced with `osci_synth::ShapeRenderer`, the same walker the
//! synth uses for audio.

use std::f32::consts::{PI, TAU};

use osci_core::{CircleArc, Line, Point, Shape};
use osci_synth::ShapeRenderer;

use crate::params::{self, ParamValues};

/// Samples per video frame. Plus the closing sample, this is 2048 segments,
/// the most `OsciRenderer` draws; detailed SVGs need all of them.
pub const SAMPLES_PER_FRAME: usize = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeKind {
    Circle,
    Square,
    Triangle,
    Star,
    Lissajous,
    /// Shapes loaded from the SVG File parameter.
    File,
}

impl ShapeKind {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Circle,
            1 => Self::Square,
            2 => Self::Triangle,
            3 => Self::Star,
            4 => Self::Lissajous,
            _ => Self::File,
        }
    }

    /// Unit-size outline in -1..1, or `None` for Lissajous (generated per
    /// sample) and File (shapes come from the scene).
    fn shapes(self) -> Option<Vec<Box<dyn Shape>>> {
        Some(match self {
            Self::Circle => vec![Box::new(CircleArc::new(0.0, 0.0, 1.0, 1.0, 0.0, TAU))],
            Self::Square => polygon(&[(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]),
            Self::Triangle => polygon(&regular(3, |_| 1.0)),
            Self::Star => polygon(&regular(10, |i| if i % 2 == 0 { 1.0 } else { 0.4 })),
            Self::Lissajous | Self::File => return None,
        })
    }
}

/// Closed outline through `points`.
fn polygon(points: &[(f32, f32)]) -> Vec<Box<dyn Shape>> {
    (0..points.len())
        .map(|i| {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % points.len()];
            Box::new(Line::new_2d(x1, y1, x2, y2)) as Box<dyn Shape>
        })
        .collect()
}

/// `n` points round a circle, starting at the top, with radius per point.
fn regular(n: usize, radius: impl Fn(usize) -> f32) -> Vec<(f32, f32)> {
    (0..n)
        .map(|i| {
            let a = PI / 2.0 + TAU * i as f32 / n as f32;
            (radius(i) * a.cos(), radius(i) * a.sin())
        })
        .collect()
}

pub struct Scene {
    renderer: ShapeRenderer,
    /// Shapes from the SVG file, already fitted to -1..1. Empty if none.
    file_shapes: Vec<Box<dyn Shape>>,
    xs: Vec<f32>,
    ys: Vec<f32>,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        // frequency = rate / N makes one trip round the shape every N samples.
        let rate = 48_000.0;
        Self {
            renderer: ShapeRenderer::new(rate, rate / SAMPLES_PER_FRAME as f64),
            file_shapes: Vec::new(),
            xs: Vec::with_capacity(SAMPLES_PER_FRAME + 1),
            ys: Vec::with_capacity(SAMPLES_PER_FRAME + 1),
        }
    }

    /// Replace the shapes drawn for `ShapeKind::File` (empty to clear).
    pub fn set_file_shapes(&mut self, shapes: Vec<Box<dyn Shape>>) {
        self.file_shapes = shapes;
    }

    pub fn has_file_shapes(&self) -> bool {
        !self.file_shapes.is_empty()
    }

    /// Build this frame's samples. `time` is host time in seconds.
    /// Returns empty slices when there is nothing to draw.
    pub fn update(&mut self, params: &ParamValues, time: f64) -> (&[f32], &[f32]) {
        let kind = params.shape();
        self.xs.clear();
        self.ys.clear();

        // Restart the walker every video frame so each frame is exactly one
        // trip round the shape, starting at the same point. Cloning shapes
        // once per frame (not per sample) is cheap.
        match kind {
            ShapeKind::Lissajous => {}
            ShapeKind::File if self.file_shapes.is_empty() => return (&self.xs, &self.ys),
            ShapeKind::File => {
                let shapes = self.file_shapes.iter().map(|s| s.clone_shape()).collect();
                self.renderer.set_shapes(shapes);
            }
            _ => self.renderer.set_shapes(kind.shapes().unwrap_or_default()),
        }

        let size = params.get(params::SIZE);
        // Spin is in turns per second; wrap so precision holds over long sets.
        let angle = ((params.get(params::SPIN) as f64 * time).fract() * std::f64::consts::TAU) as f32;
        let (sin, cos) = angle.sin_cos();

        for i in 0..SAMPLES_PER_FRAME {
            let p = match kind {
                ShapeKind::Lissajous => {
                    let t = TAU * i as f32 / SAMPLES_PER_FRAME as f32;
                    let phase = (time * 0.25).fract() as f32 * TAU;
                    Point::xy((3.0 * t + phase).sin(), (2.0 * t).sin())
                }
                _ => self.renderer.next_vector(),
            };
            self.xs.push(size * (p.x * cos - p.y * sin));
            self.ys.push(size * (p.x * sin + p.y * cos));
        }
        // Repeat the first sample so the last segment closes the loop.
        self.xs.push(self.xs[0]);
        self.ys.push(self.ys[0]);
        (&self.xs, &self.ys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params_for(kind: usize) -> ParamValues {
        let mut p = ParamValues::default();
        p.set_host(params::SHAPE, kind as f32);
        p.set_host(params::SIZE, 1.0); // real size 1.0
        p
    }

    const FILE: usize = 5;

    /// A diagonal line from (-1,-1) to (1,1), standing in for parsed SVG shapes.
    fn diagonal() -> Vec<Box<dyn Shape>> {
        vec![Box::new(Line::new_2d(-1.0, -1.0, 1.0, 1.0))]
    }

    #[test]
    fn every_shape_fills_a_frame_in_bounds() {
        let mut scene = Scene::new();
        scene.set_file_shapes(diagonal());
        for kind in 0..params::SHAPE_NAMES.len() {
            let (xs, ys) = scene.update(&params_for(kind), 0.0);
            assert_eq!(xs.len(), SAMPLES_PER_FRAME + 1);
            assert_eq!(ys.len(), SAMPLES_PER_FRAME + 1);
            for (&x, &y) in xs.iter().zip(ys) {
                assert!(x.is_finite() && y.is_finite());
                assert!(x.abs() <= 1.0 + 1e-4 && y.abs() <= 1.0 + 1e-4, "kind {kind}: ({x}, {y})");
            }
        }
    }

    #[test]
    fn file_without_shapes_draws_nothing() {
        let mut scene = Scene::new();
        let (xs, ys) = scene.update(&params_for(FILE), 0.0);
        assert!(xs.is_empty() && ys.is_empty());
        assert!(!scene.has_file_shapes());
    }

    #[test]
    fn file_shapes_are_traced() {
        let mut scene = Scene::new();
        scene.set_file_shapes(diagonal());
        let (xs, ys) = scene.update(&params_for(FILE), 0.0);
        assert_eq!((xs[0], ys[0]), (-1.0, -1.0));
        // Every sample lies on the line y = x.
        assert!(xs.iter().zip(ys).all(|(x, y)| (x - y).abs() < 1e-4));
        // Frames repeat while the file stays loaded (shapes are cloned, not consumed).
        let first = xs.to_vec();
        assert_eq!(scene.update(&params_for(FILE), 0.0).0, &first[..]);
    }

    #[test]
    fn clearing_file_shapes_stops_drawing() {
        let mut scene = Scene::new();
        scene.set_file_shapes(diagonal());
        scene.set_file_shapes(Vec::new());
        assert!(scene.update(&params_for(FILE), 0.0).0.is_empty());
    }

    #[test]
    fn circle_samples_lie_on_the_circle() {
        let mut scene = Scene::new();
        let (xs, ys) = scene.update(&params_for(0), 0.0);
        for (&x, &y) in xs.iter().zip(ys) {
            assert!(((x * x + y * y).sqrt() - 1.0).abs() < 1e-3);
        }
    }

    #[test]
    fn frame_closes_on_itself() {
        let mut scene = Scene::new();
        let (xs, ys) = scene.update(&params_for(1), 0.0);
        assert_eq!((xs[0], ys[0]), (xs[SAMPLES_PER_FRAME], ys[SAMPLES_PER_FRAME]));
    }

    #[test]
    fn square_traces_all_four_edges_in_order() {
        let mut scene = Scene::new();
        let (xs, ys) = scene.update(&params_for(1), 0.0);
        // A quarter of the samples per edge: check the corners.
        let at = |i: usize| (xs[i], ys[i]);
        let close = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01;
        let q = SAMPLES_PER_FRAME / 4;
        assert!(close(at(0), (-1.0, -1.0)), "{:?}", at(0));
        assert!(close(at(q), (1.0, -1.0)), "{:?}", at(q));
        assert!(close(at(2 * q), (1.0, 1.0)), "{:?}", at(2 * q));
        assert!(close(at(3 * q), (-1.0, 1.0)), "{:?}", at(3 * q));
    }

    #[test]
    fn consecutive_frames_are_identical_without_motion() {
        let mut scene = Scene::new();
        let p = params_for(3);
        let a = scene.update(&p, 0.0).0.to_vec();
        let b = scene.update(&p, 0.0).0.to_vec();
        assert_eq!(a, b);
    }

    #[test]
    fn spin_rotates_over_time() {
        let mut p = params_for(1);
        p.set_host(params::SPIN, 0.75); // +0.5 turns/s
        let mut scene = Scene::new();
        let a = scene.update(&p, 0.0).0[0];
        let mut scene = Scene::new();
        let b = scene.update(&p, 1.0).0[0]; // half a turn later
        assert!((a + b).abs() < 1e-3, "{a} vs {b}");
    }

    #[test]
    fn size_scales_output() {
        let mut p = params_for(0);
        p.set_host(params::SIZE, 0.0); // real size 0.05
        let mut scene = Scene::new();
        let (xs, _) = scene.update(&p, 0.0);
        assert!(xs.iter().all(|x| x.abs() <= 0.05 + 1e-4));
    }
}
