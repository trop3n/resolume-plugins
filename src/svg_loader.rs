//! Loads SVG files off the render thread.
//!
//! Parsing a large SVG can take longer than a video frame, so each load runs
//! on its own thread and the result is picked up by the next `poll`. Only the
//! most recent request counts: results from superseded loads are dropped.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

use osci_core::Shape;

pub type Shapes = Vec<Box<dyn Shape>>;

pub struct Loaded {
    pub path: PathBuf,
    pub result: Result<Shapes, String>,
}

struct Message {
    generation: u64,
    loaded: Loaded,
}

pub struct SvgLoader {
    tx: Sender<Message>,
    rx: Receiver<Message>,
    /// Bumped on every request; results tagged with an older value are stale.
    generation: u64,
}

impl Default for SvgLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SvgLoader {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx, generation: 0 }
    }

    /// Start loading `path`, replacing any load in progress.
    pub fn load(&mut self, path: PathBuf) {
        self.generation += 1;
        let generation = self.generation;
        let tx = self.tx.clone();
        let spawned = std::thread::Builder::new().name("rusci-svg-load".into()).spawn(move || {
            let result = load_svg(&path);
            // The instance may be gone by now; that's fine.
            let _ = tx.send(Message { generation, loaded: Loaded { path, result } });
        });
        if let Err(e) = spawned {
            // Report through poll like any other failure.
            let _ = self.tx.send(Message {
                generation,
                loaded: Loaded { path: PathBuf::new(), result: Err(format!("could not start loader thread: {e}")) },
            });
        }
    }

    /// Forget any load in progress (e.g. the file was cleared).
    pub fn cancel(&mut self) {
        self.generation += 1;
    }

    /// The finished load for the current request, if it has arrived.
    pub fn poll(&mut self) -> Option<Loaded> {
        let mut latest = None;
        while let Ok(msg) = self.rx.try_recv() {
            if msg.generation == self.generation {
                latest = Some(msg.loaded);
            }
        }
        latest
    }
}

/// Read and parse an SVG. `parse_svg` returns shapes Y-up, centred and
/// fitted to -0.9..0.9.
pub fn load_svg(path: &Path) -> Result<Shapes, String> {
    let data = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let shapes = osci_parsers::svg::parse_svg(&data).map_err(|e| format!("{}: {e}", path.display()))?;
    if shapes.is_empty() {
        return Err(format!("{}: no drawable paths found", path.display()));
    }
    Ok(shapes)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    pub const SQUARE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
        <path d="M 10 10 L 90 10 L 90 90 L 10 90 Z" fill="none" stroke="black"/>
    </svg>"#;

    /// A fresh file in a per-test temp directory.
    pub fn temp_file(test: &str, name: &str, contents: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("resolume-rusci-{}-{test}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, contents).unwrap();
        path
    }

    pub fn wait_for(loader: &mut SvgLoader) -> Loaded {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(l) = loader.poll() {
                return l;
            }
            assert!(Instant::now() < deadline, "load timed out");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn corners(shapes: &Shapes) -> Vec<(f32, f32)> {
        shapes.iter().map(|s| s.next_vector(0.0)).map(|p| ((p.x * 1e3).round() / 1e3, (p.y * 1e3).round() / 1e3)).collect()
    }

    #[test]
    fn loads_an_svg_centred_and_upright() {
        // The square sits at 10..90 in a 0..100 canvas: a margin on every side.
        let path = temp_file("loads", "square.svg", SQUARE_SVG);
        let shapes = load_svg(&path).unwrap();
        assert_eq!(shapes.len(), 4);
        // Starts at the SVG's top-left (10,10), which is (-, +) once y points up.
        assert_eq!(corners(&shapes), vec![(-0.9, 0.9), (0.9, 0.9), (0.9, -0.9), (-0.9, -0.9)]);
    }

    #[test]
    fn wide_svg_keeps_its_proportions() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 100">
            <path d="M 50 25 L 250 25 L 250 75 L 50 75 Z" fill="none" stroke="black"/></svg>"#;
        let shapes = load_svg(&temp_file("wide", "wide.svg", svg)).unwrap();
        // 200 x 50 box: x spans -0.9..0.9, y a quarter of that, centred.
        assert_eq!(corners(&shapes), vec![(-0.9, 0.225), (0.9, 0.225), (0.9, -0.225), (-0.9, -0.225)]);
    }

    #[test]
    fn reports_missing_file() {
        let err = load_svg(Path::new("definitely/not/here.svg")).err().unwrap();
        assert!(err.contains("here.svg"), "{err}");
    }

    #[test]
    fn reports_bad_svg() {
        let path = temp_file("bad", "bad.svg", "this is not xml");
        assert!(load_svg(&path).is_err());
    }

    #[test]
    fn reports_svg_without_paths() {
        let path = temp_file("empty", "empty.svg", r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"/>"#);
        let err = load_svg(&path).err().unwrap();
        assert!(err.contains("no drawable paths"), "{err}");
    }

    #[test]
    fn background_load_arrives_via_poll() {
        let path = temp_file("bg", "square.svg", SQUARE_SVG);
        let mut loader = SvgLoader::new();
        loader.load(path.clone());
        let loaded = wait_for(&mut loader);
        assert_eq!(loaded.path, path);
        assert_eq!(loaded.result.unwrap().len(), 4);
    }

    #[test]
    fn newer_request_wins() {
        let good = temp_file("newer", "square.svg", SQUARE_SVG);
        let mut loader = SvgLoader::new();
        loader.load(PathBuf::from("missing.svg"));
        loader.load(good.clone());
        let loaded = wait_for(&mut loader);
        assert_eq!(loaded.path, good);
        assert!(loaded.result.is_ok());
        // The superseded result never shows up later either.
        std::thread::sleep(Duration::from_millis(50));
        assert!(loader.poll().is_none());
    }

    #[test]
    fn cancel_drops_pending_result() {
        let path = temp_file("cancel", "square.svg", SQUARE_SVG);
        let mut loader = SvgLoader::new();
        loader.load(path);
        loader.cancel();
        std::thread::sleep(Duration::from_millis(100));
        assert!(loader.poll().is_none());
    }
}
