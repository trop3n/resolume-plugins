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

/// Largest SVG we'll read. Real line art is far smaller; this stops a huge or
/// growing file from filling memory.
pub const MAX_SVG_BYTES: u64 = 32 * 1024 * 1024;

/// Read and parse an SVG. `parse_svg` returns shapes Y-up, centred and
/// fitted to -0.9..0.9.
pub fn load_svg(path: &Path) -> Result<Shapes, String> {
    load_svg_limited(path, MAX_SVG_BYTES)
}

fn load_svg_limited(path: &Path, max_bytes: u64) -> Result<Shapes, String> {
    let data = read_limited(path, max_bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let shapes = osci_parsers::svg::parse_svg(&data).map_err(|e| format!("{}: {e}", path.display()))?;
    if shapes.is_empty() {
        return Err(format!("{}: no drawable paths found", path.display()));
    }
    Ok(shapes)
}

/// Read a regular file of at most `max_bytes`.
fn read_limited(path: &Path, max_bytes: u64) -> Result<Vec<u8>, String> {
    use std::io::Read;

    if is_device_path(path) {
        // e.g. \\.\pipe\x or \\.\COM1: opening these can block forever.
        return Err("device paths are not supported".into());
    }
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("not a regular file".into());
    }
    if meta.len() > max_bytes {
        return Err(format!("file is {} bytes; the limit is {max_bytes}", meta.len()));
    }
    // Read through a limit too, in case the file grows after the size check.
    let mut data = Vec::with_capacity(meta.len() as usize);
    file.take(max_bytes + 1).read_to_end(&mut data).map_err(|e| e.to_string())?;
    if data.len() as u64 > max_bytes {
        return Err(format!("file is over the {max_bytes}-byte limit"));
    }
    Ok(data)
}

/// Windows device-namespace paths (`\\.\...`, and `\\?\...` unless it is just
/// the long-path form of a drive path like `\\?\C:\...`). Network shares
/// (`\\server\share`) are allowed, like elsewhere in Resolume.
fn is_device_path(path: &Path) -> bool {
    let s = path.as_os_str().to_string_lossy().replace('/', "\\");
    if s.starts_with("\\\\.\\") {
        return true;
    }
    if let Some(rest) = s.strip_prefix("\\\\?\\") {
        let b = rest.as_bytes();
        let is_drive = b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':';
        return !is_drive;
    }
    false
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
    fn rejects_device_paths() {
        for p in [r"\\.\pipe\x.svg", r"\\.\COM1", "//./pipe/x", r"\\?\GLOBALROOT\Device\x.svg", r"\\?\UNC\srv\x.svg"] {
            let err = load_svg(Path::new(p)).err().unwrap();
            assert!(err.contains("device paths"), "{p}: {err}");
        }
    }

    #[test]
    fn allows_drive_and_network_paths_through_the_device_check() {
        for p in [r"C:\art\x.svg", r"\\?\C:\art\x.svg", r"\\server\share\x.svg", "relative/x.svg"] {
            assert!(!is_device_path(Path::new(p)), "{p}");
        }
    }

    #[test]
    fn rejects_directories() {
        let dir = temp_file("dir", "placeholder.svg", SQUARE_SVG);
        let err = load_svg(dir.parent().unwrap()).err().unwrap();
        // Windows can't open a directory as a file at all; Unix opens it and we reject it.
        assert!(err.contains("not a regular file") || err.contains("denied") || err.contains("directory"), "{err}");
    }

    #[test]
    fn rejects_oversized_files() {
        let path = temp_file("big", "square.svg", SQUARE_SVG);
        let err = load_svg_limited(&path, 16).err().unwrap();
        assert!(err.contains("limit"), "{err}");
        assert!(load_svg_limited(&path, SQUARE_SVG.len() as u64).is_ok());
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
