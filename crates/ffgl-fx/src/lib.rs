//! Shared plumbing for the GLSL effect plugins in this workspace.
//!
//! A plugin describes itself with a `static` [`EffectDef`] (identity,
//! parameters, shader passes) and calls [`export_effect!`]. This crate does
//! the FFGL 2.1 ABI, parameter handling and GL rendering.
//!
//! Shaders are GLSL 4.10 fragment bodies (they define `main`). Each one gets
//! a prelude with these, plus a uniform per parameter named by its `ident`:
//!
//! | Name | Type | Meaning |
//! |---|---|---|
//! | `uv` | `vec2` | 0..1 across the output |
//! | `fragColor` | `vec4` | output |
//! | `inputImage` | `sampler2D` | the clip; read with `IMG_INPUT(uv)` or `FETCH_INPUT(ivec2)` |
//! | `passIn` | `sampler2D` | previous pass's output; `IMG_PASS` / `FETCH_PASS` |
//! | `RENDERSIZE` | `vec2` | output size in pixels |
//! | `TIME` | `float` | seconds |
//! | `FRAMEINDEX`, `PASSINDEX`, `ITERATION` | `int` | counters |
//! | `pixelCoord()` | `ivec2` | integer pixel, same in every pass |
//! | `sizeScale()` | `float` | multiply sizes given in pixels at 1080p by this |
//! | *buffer name* | `sampler2D` | a persistent buffer (see [`Pass::store`]) |
//!
//! Helpers also in the prelude: `hash12`, `hash22`, `valueNoise`, `fbm`
//! (0..1), `luma` (Rec. 709), `rgb2hsv`, `hsv2rgb`, and `bayer2/4/8`
//! (ordered-dither thresholds for integer cell coordinates).
//!
//! Plugins add `ffgl_fx::effect_tests!(EFFECT);` for the standard checks.

pub mod ffi;
mod gl_loader;
mod instance;
mod params;

use std::ffi::{c_char, c_void, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use ffi::*;
pub use instance::{fragment_source, Instance};
pub use params::{HostKind, HostParam, Kind, Param};

/// One shader pass.
pub struct Pass {
    pub shader: &'static str,
    /// `ident` of an `Int` param giving how many times this pass runs each
    /// frame (ping-ponging between iterations). `None` runs it once. The last
    /// pass always runs once.
    pub repeat: Option<&'static str>,
    /// For a repeated pass: how many of the param's steps one run does. The
    /// pass runs `ceil(value / steps_per_run)` times; the shader gets the
    /// run number in `ITERATION` and works out which steps are its own.
    pub steps_per_run: u32,
    /// Render into the persistent buffer of this name instead of the
    /// ping-pong chain. Every pass can sample the buffer as a `sampler2D` of
    /// that name; it keeps its contents across frames, so a pass sees the
    /// last value written, which may be from the previous frame. Buffers
    /// start out transparent black and are cleared when the size changes.
    pub store: Option<&'static str>,
}

impl Pass {
    pub const fn new(shader: &'static str) -> Self {
        Self { shader, repeat: None, steps_per_run: 1, store: None }
    }

    pub const fn repeated(shader: &'static str, ident: &'static str) -> Self {
        Self { shader, repeat: Some(ident), steps_per_run: 1, store: None }
    }

    /// Repeated pass where each run does `steps_per_run` steps.
    pub const fn repeated_by(shader: &'static str, ident: &'static str, steps_per_run: u32) -> Self {
        Self { shader, repeat: Some(ident), steps_per_run, store: None }
    }

    /// Write this pass into the persistent buffer `name` (see [`Pass::store`]).
    pub const fn storing(self, name: &'static str) -> Self {
        Self { store: Some(name), ..self }
    }
}

pub struct EffectDef {
    /// Four-character plugin ID, unique among installed plugins.
    pub id: [u8; 4],
    /// Name in Resolume's effect list, at most 16 bytes.
    pub name: &'static str,
    pub short_name: &'static CStr,
    pub description: &'static CStr,
    pub about: &'static CStr,
    pub version: (u32, u32),
    pub params: &'static [Param],
    pub passes: &'static [Pass],
    /// Always `Cache::new()`.
    pub cache: Cache,
}

/// Lazily built FFGL structs and host params for an [`EffectDef`].
pub struct Cache(OnceLock<Statics>);

impl Cache {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }
}

struct Statics {
    info: PluginInfoStruct,
    extended: PluginExtendedInfoStruct,
    host: Vec<HostParam>,
    first_host: Vec<usize>,
}

// The raw pointers in the FFGL structs point at 'static strings.
unsafe impl Send for Statics {}
unsafe impl Sync for Statics {}

impl EffectDef {
    fn statics(&self) -> &Statics {
        self.cache.0.get_or_init(|| {
            let mut name = [0u8; 16];
            let n = self.name.len().min(16);
            name[..n].copy_from_slice(&self.name.as_bytes()[..n]);
            let (host, first_host) = params::expand(self.params);
            Statics {
                info: PluginInfoStruct {
                    APIMajorVersion: 2,
                    APIMinorVersion: 1,
                    PluginUniqueID: self.id,
                    PluginName: name,
                    PluginType: FF_EFFECT,
                },
                extended: PluginExtendedInfoStruct {
                    PluginMajorVersion: self.version.0,
                    PluginMinorVersion: self.version.1,
                    Description: self.description.as_ptr(),
                    About: self.about.as_ptr(),
                    FreeFrameExtendedDataSize: 0,
                    FreeFrameExtendedDataBlock: std::ptr::null_mut(),
                },
                host,
                first_host,
            }
        })
    }

    /// Parameters as Resolume sees them (colours split into channels).
    pub fn host_params(&self) -> &[HostParam] {
        &self.statics().host
    }

    /// Index of the first host param of effect param `param`.
    pub fn first_host(&self, param: usize) -> usize {
        self.statics().first_host[param]
    }

    /// Persistent buffer names, in order of first use.
    pub fn buffers(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        for name in self.passes.iter().filter_map(|p| p.store) {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names
    }

    /// Check the definition for mistakes the compiler can't catch. Plugins
    /// call this from a test.
    pub fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        if self.name.is_empty() || self.name.len() > 16 {
            errors.push(format!("name {:?} must be 1..=16 bytes", self.name));
        }
        if !self.id.iter().all(u8::is_ascii_alphanumeric) {
            errors.push("id must be 4 ASCII letters or digits".to_owned());
        }
        match self.passes.last() {
            None => errors.push("needs at least one pass".to_owned()),
            Some(p) if p.repeat.is_some() => errors.push("the last pass can't repeat".to_owned()),
            Some(p) if p.store.is_some() => errors.push("the last pass can't store into a buffer".to_owned()),
            _ => {}
        }
        for name in self.buffers() {
            let valid = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !name.starts_with("gl_");
            if !valid || instance::BUILTINS.contains(&name) || self.params.iter().any(|p| p.ident == name) {
                errors.push(format!("buffer name {name:?} isn't a free GLSL name"));
            }
        }
        for (i, p) in self.params.iter().enumerate() {
            let valid_ident = p.ident.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && p.ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !p.ident.starts_with("gl_");
            if !valid_ident || instance::BUILTINS.contains(&p.ident) {
                errors.push(format!("param {:?}: ident {:?} isn't a free GLSL name", p.name, p.ident));
            }
            if self.params[..i].iter().any(|q| q.ident == p.ident) {
                errors.push(format!("ident {:?} used twice", p.ident));
            }
            let ok = match p.kind {
                Kind::Float { min, max, default } => min < max && (min..=max).contains(&default),
                Kind::Int { min, max, default } => min < max && (min..=max).contains(&default),
                Kind::Option { elements, default } => default < elements.len(),
                Kind::Bool { .. } => true,
                Kind::Colour { default } => default.iter().all(|c| (0.0..=1.0).contains(c)),
                Kind::Point { default } => default.iter().all(|c| (0.0..=1.0).contains(c)),
            };
            if !ok {
                errors.push(format!("param {:?}: bad range or default", p.name));
            }
        }
        for p in self.params {
            let used = self.passes.iter().any(|pass| uses_ident(pass.shader, p.ident))
                || self.passes.iter().any(|pass| pass.repeat == Some(p.ident));
            if !used {
                errors.push(format!("param {:?} ({}) isn't used by any pass", p.name, p.ident));
            }
        }
        for (i, pass) in self.passes.iter().enumerate() {
            if pass.steps_per_run == 0 {
                errors.push(format!("pass {i}: steps_per_run must be at least 1"));
            }
            if let Some(ident) = pass.repeat {
                if !self.params.iter().any(|p| p.ident == ident && matches!(p.kind, Kind::Int { .. })) {
                    errors.push(format!("pass {i} repeats by {ident:?}, which isn't an Int param"));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }
}

/// True if `ident` appears in `src` as a whole word.
fn uses_ident(src: &str, ident: &str) -> bool {
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    src.match_indices(ident).any(|(i, _)| {
        let before = src[..i].chars().next_back();
        let after = src[i + ident.len()..].chars().next();
        !before.is_some_and(word) && !after.is_some_and(word)
    })
}

/// Standard tests for a plugin: `ffgl_fx::effect_tests!(EFFECT);`
#[macro_export]
macro_rules! effect_tests {
    ($def:path) => {
        #[cfg(test)]
        mod effect_tests {
            #[allow(unused_imports)]
            use super::*;

            #[test]
            fn definition_is_valid() {
                if let Err(e) = $def.validate() {
                    panic!("{e}");
                }
            }
        }
    };
}

/// Export `plugMain` and `SetLogCallback` for a `static` [`EffectDef`].
#[macro_export]
macro_rules! export_effect {
    ($def:path) => {
        #[no_mangle]
        pub extern "system" fn plugMain(
            code: u32,
            input: $crate::ffi::FFMixed,
            instance: *mut ::std::ffi::c_void,
        ) -> $crate::ffi::FFMixed {
            $crate::plug_main(&$def, code, input, instance)
        }

        #[no_mangle]
        pub extern "system" fn SetLogCallback(callback: $crate::ffi::PFNLog) {
            $crate::set_log_callback(callback)
        }
    };
}

// ── Host logging ────────────────────────────────────────────────────

static LOG_CALLBACK: AtomicUsize = AtomicUsize::new(0);
static LOG_PREFIX: OnceLock<String> = OnceLock::new();

pub fn set_log_callback(callback: PFNLog) {
    LOG_CALLBACK.store(callback.map_or(0, |f| f as usize), Ordering::Release);
}

/// Send a line to Resolume's log, if it gave us a callback.
pub fn log(msg: &str) {
    let f = LOG_CALLBACK.load(Ordering::Acquire);
    if f == 0 {
        return;
    }
    let prefix = LOG_PREFIX.get().map_or("ffgl-fx", String::as_str);
    if let Ok(line) = CString::new(format!("[{prefix}] {msg}")) {
        let f: unsafe extern "system" fn(*mut c_char) = unsafe { std::mem::transmute(f) };
        unsafe { f(line.as_ptr() as *mut c_char) };
    }
}

// ── Entry point ─────────────────────────────────────────────────────

pub fn plug_main(def: &'static EffectDef, code: u32, input: FFMixed, instance: *mut c_void) -> FFMixed {
    LOG_PREFIX.get_or_init(|| def.name.to_owned());
    // A panic must never unwind into Resolume.
    catch_unwind(AssertUnwindSafe(|| unsafe { dispatch(def, code, input, instance.cast()) })).unwrap_or_else(|e| {
        let msg = e.downcast_ref::<&str>().copied().or_else(|| e.downcast_ref::<String>().map(|s| s.as_str()));
        log(&format!("panic in function {code}: {}", msg.unwrap_or("unknown")));
        FFMixed::fail()
    })
}

/// # Safety
/// `instance` is null or a pointer returned by `FF_INSTANTIATE_GL` for this
/// `def`, and `input` matches what the FFGL spec says `code` takes.
pub unsafe fn dispatch(def: &'static EffectDef, code: u32, input: FFMixed, instance: *mut Instance) -> FFMixed {
    let inst = instance.as_mut();
    let params = def.host_params();
    let index = || input.as_u32() as usize;
    let ok = |b: bool| FFMixed::uint(if b { FF_SUCCESS } else { FF_FAIL });

    match code {
        FF_GET_INFO => FFMixed::ptr(&def.statics().info),
        FF_GET_EXTENDED_INFO => FFMixed::ptr(&def.statics().extended),
        FF_GET_PLUGIN_SHORT_NAME => FFMixed::ptr(def.short_name.as_ptr()),
        FF_INITIALISE_V2 | FF_DEINITIALISE => ok(true),

        FF_GET_PLUGIN_CAPS => FFMixed::uint(match input.as_u32() {
            FF_CAP_SET_TIME => FF_TRUE,
            // An effect: exactly one input clip.
            FF_CAP_MINIMUM_INPUT_FRAMES | FF_CAP_MAXIMUM_INPUT_FRAMES => 1,
            _ => FF_FALSE,
        }),
        FF_ENABLE_PLUGIN_CAP => FFMixed::fail(),

        // ── Parameter metadata (no instance needed) ──
        FF_GET_NUM_PARAMETERS => FFMixed::uint(params.len() as u32),
        FF_GET_PARAMETER_NAME => match params.get(index()) {
            Some(p) => FFMixed::ptr(p.name.as_ptr()),
            None => FFMixed::ptr(std::ptr::null::<c_char>()),
        },
        FF_GET_PARAMETER_DEFAULT => params.get(index()).map_or(FFMixed::fail(), |p| FFMixed::float(p.host_default())),
        FF_GET_PARAMETER_TYPE => params.get(index()).map_or(FFMixed::fail(), |p| FFMixed::uint(p.ffgl_type())),
        FF_GET_PARAMETER_USAGE => FFMixed::uint(FF_USAGE_STANDARD),
        FF_GET_PRAMETER_VISIBILITY => FFMixed::uint(FF_TRUE),
        FF_GET_NUM_PARAMETER_ELEMENTS => {
            params.get(index()).map_or(FFMixed::fail(), |p| FFMixed::uint(p.elements().len() as u32))
        }
        FF_GET_PARAMETER_ELEMENT_NAME => {
            let args = &*(input.as_ptr() as *const GetParameterElementNameStruct);
            match params.get(args.ParameterNumber as usize).and_then(|p| p.elements().get(args.ElementNumber as usize)) {
                Some(name) => FFMixed::ptr(name.as_ptr()),
                None => FFMixed::ptr(std::ptr::null::<c_char>()),
            }
        }
        FF_GET_PARAMETER_ELEMENT_VALUE => {
            let args = &*(input.as_ptr() as *const GetParameterElementValueStruct);
            match params.get(args.ParameterNumber as usize) {
                Some(p) if (args.ElementNumber as usize) < p.elements().len() => FFMixed::float(args.ElementNumber as f32),
                _ => FFMixed::fail(),
            }
        }
        FF_GET_NUM_ELEMENT_SEPARATORS => FFMixed::uint(0),
        FF_GET_NUM_FILE_PARAMETER_EXTENSIONS => FFMixed::uint(0),
        FF_GET_RANGE => {
            let args = &mut *(input.as_ptr() as *mut GetRangeStruct);
            let Some(p) = params.get(args.parameterNumber as usize) else {
                return FFMixed::fail();
            };
            args.range = RangeStruct { min: 0.0, max: p.host_max() };
            ok(true)
        }
        FF_GET_PARAM_GROUP | FF_GET_PARAM_DISPLAY_NAME => {
            let args = &mut *(input.as_ptr() as *mut GetStringStruct);
            let Some(p) = params.get(args.parameterNumber as usize) else {
                return FFMixed::fail();
            };
            let text = if code == FF_GET_PARAM_GROUP { p.group.as_bytes() } else { p.name.to_bytes() };
            ok(write_host_string(&mut args.stringBuffer, text))
        }
        FF_GET_THUMBNAIL => {
            if let Some(t) = (input.as_ptr() as *mut GetThumbnailStruct).as_mut() {
                t.width = 0;
                t.height = 0;
            }
            FFMixed::fail()
        }

        // ── Instance lifecycle ──
        FF_INSTANTIATE_GL => match (input.as_ptr() as *const FFGLViewportStruct).as_ref() {
            Some(&vp) => instantiate(def, vp),
            None => FFMixed::fail(),
        },
        FF_DEINSTANTIATE_GL => {
            if instance.is_null() {
                return FFMixed::fail();
            }
            let mut boxed = Box::from_raw(instance);
            boxed.deinit_gl();
            ok(true)
        }
        FF_CONNECT | FF_DISCONNECT => ok(inst.is_some()),
        FF_SET_BEATINFO | FF_SET_HOSTINFO | FF_SET_SAMPLERATE => ok(true),

        // ── Per-instance calls ──
        FF_GET_INPUT_STATUS => match inst {
            Some(_) if index() == 0 => FFMixed::uint(FF_INPUT_INUSE),
            _ => FFMixed::fail(),
        },
        FF_SET_PARAMETER => match inst {
            Some(i) => {
                let args = &*(input.as_ptr() as *const SetParameterStruct);
                ok(i.set_host(args.ParameterNumber as usize, f32::from_bits(args.NewParameterValue.as_u32())))
            }
            None => FFMixed::fail(),
        },
        FF_GET_PARAMETER => match inst {
            Some(i) => i.get_host(index()).map_or(FFMixed::fail(), FFMixed::float),
            None => FFMixed::fail(),
        },
        FF_GET_PARAMETER_DISPLAY => {
            let Some(i) = inst else {
                return FFMixed::fail();
            };
            match i.display_text(index()) {
                Some(text) => {
                    i.display = CString::new(text).unwrap_or_default();
                    FFMixed::ptr(i.display.as_ptr())
                }
                None => FFMixed::fail(),
            }
        }
        FF_GET_PARAMETER_EVENTS => match (inst, (input.as_ptr() as *mut GetParamEventsStruct).as_mut()) {
            (Some(_), Some(events)) => {
                events.numEvents = 0;
                ok(true)
            }
            _ => FFMixed::fail(),
        },
        FF_SET_TIME => match (inst, (input.as_ptr() as *const f64).as_ref()) {
            (Some(i), Some(&t)) => {
                i.set_time(t);
                ok(true)
            }
            _ => FFMixed::fail(),
        },
        FF_RESIZE => match (inst, (input.as_ptr() as *const FFGLViewportStruct).as_ref()) {
            (Some(i), Some(&vp)) => {
                i.set_viewport(vp);
                ok(true)
            }
            _ => FFMixed::fail(),
        },
        FF_PROCESS_OPENGL => match (inst, (input.as_ptr() as *const ProcessOpenGLStruct).as_ref()) {
            (Some(i), Some(gl)) => {
                let tex = if gl.numInputTextures >= 1 && !gl.inputTextures.is_null() {
                    (*gl.inputTextures).as_ref()
                } else {
                    None
                };
                ok(i.process(tex, gl.HostFBO))
            }
            _ => FFMixed::fail(),
        },

        FF_SET_PARAMETER_ELEMENT_VALUE => FFMixed::fail(),
        _ => FFMixed::fail(),
    }
}

unsafe fn instantiate(def: &'static EffectDef, viewport: FFGLViewportStruct) -> FFMixed {
    let mut inst = Box::new(Instance::new(def, viewport));
    // Catch here, not just in plug_main, so a failed init doesn't leak the instance.
    match catch_unwind(AssertUnwindSafe(|| unsafe { inst.init_gl() })) {
        Ok(Ok(())) => FFMixed::ptr(Box::into_raw(inst)),
        Ok(Err(e)) => {
            log(&format!("GL setup failed: {e}"));
            FFMixed::fail()
        }
        Err(_) => {
            log("GL setup panicked (needs an OpenGL 4.1 core context)");
            FFMixed::fail()
        }
    }
}

/// Copy into a host buffer the FFGL way: at most `maxToWrite` bytes, no nul.
unsafe fn write_host_string(buf: &mut StringBufferStruct, text: &[u8]) -> bool {
    if buf.address.is_null() || buf.maxToWrite == 0 {
        return false;
    }
    let n = text.len().min(buf.maxToWrite as usize);
    std::ptr::copy_nonoverlapping(text.as_ptr(), buf.address.cast(), n);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_FX: EffectDef = EffectDef {
        id: *b"TsFx",
        name: "Test Effect",
        short_name: c"TestFx",
        description: c"test",
        about: c"test",
        version: (0, 1),
        params: &[
            Param::float("Amount", "amount", "Main", 0.0, 2.0, 1.0),
            Param::option("Mode", "mode", "Main", &["One", "Two", "Three"], 1),
            Param::int("Steps", "steps", "Main", 0, 10, 4),
            Param::colour("Tint", "tint", "Colour", [1.0, 0.5, 0.0]),
        ],
        passes: &[
            Pass::repeated("void main(){}", "steps"),
            Pass::repeated_by("void main(){}", "steps", 3).storing("history"),
            Pass::new("void main(){ fragColor = vec4(tint * amount, float(mode)); }"),
        ],
        cache: Cache::new(),
    };

    fn call(code: u32, input: FFMixed, inst: *mut Instance) -> FFMixed {
        unsafe { dispatch(&TEST_FX, code, input, inst) }
    }

    fn new_instance() -> *mut Instance {
        Box::into_raw(Box::new(Instance::new(&TEST_FX, FFGLViewportStruct { x: 0, y: 0, width: 64, height: 32 })))
    }

    fn free(inst: *mut Instance) {
        drop(unsafe { Box::from_raw(inst) });
    }

    fn cstr(m: FFMixed) -> String {
        unsafe { CStr::from_ptr(m.as_ptr() as *const c_char) }.to_string_lossy().into_owned()
    }

    #[test]
    fn test_def_is_valid() {
        TEST_FX.validate().unwrap();
    }

    #[test]
    fn validate_catches_mistakes() {
        static BAD: EffectDef = EffectDef {
            id: *b"B d!",
            name: "A name that is far too long",
            short_name: c"Bad",
            description: c"",
            about: c"",
            version: (0, 1),
            params: &[
                Param::float("X", "TIME", "", 0.0, 1.0, 2.0),
                Param::bool("Y", "y", "", false),
                Param::bool("Y2", "y", "", false),
            ],
            passes: &[Pass::new("").storing("y"), Pass::repeated("", "y")],
            cache: Cache::new(),
        };
        let err = BAD.validate().unwrap_err();
        for needle in ["name", "id", "last pass", "TIME", "bad range", "used twice", "isn't an Int", "isn't used", "buffer name \"y\""] {
            assert!(err.contains(needle), "missing {needle:?} in:\n{err}");
        }
    }

    #[test]
    fn info_identifies_an_effect() {
        let info = unsafe { &*(call(FF_GET_INFO, FFMixed::uint(0), std::ptr::null_mut()).as_ptr() as *const PluginInfoStruct) };
        assert_eq!((info.APIMajorVersion, info.APIMinorVersion), (2, 1));
        assert_eq!(&info.PluginUniqueID, b"TsFx");
        assert_eq!(&info.PluginName[..12], b"Test Effect\0");
        assert_eq!(info.PluginType, FF_EFFECT);
        assert_eq!(cstr(call(FF_GET_PLUGIN_SHORT_NAME, FFMixed::uint(0), std::ptr::null_mut())), "TestFx");
    }

    #[test]
    fn caps_say_one_input_with_time() {
        let cap = |c| call(FF_GET_PLUGIN_CAPS, FFMixed::uint(c), std::ptr::null_mut()).as_u32();
        assert_eq!(cap(FF_CAP_SET_TIME), FF_TRUE);
        assert_eq!(cap(FF_CAP_MINIMUM_INPUT_FRAMES), 1);
        assert_eq!(cap(FF_CAP_MAXIMUM_INPUT_FRAMES), 1);
        assert_eq!(cap(FF_CAP_TOP_LEFT_TEXTURE_ORIENTATION), FF_FALSE);
    }

    #[test]
    fn parameter_metadata() {
        let null = std::ptr::null_mut();
        assert_eq!(call(FF_GET_NUM_PARAMETERS, FFMixed::uint(0), null).as_u32(), 6);
        assert_eq!(cstr(call(FF_GET_PARAMETER_NAME, FFMixed::uint(1), null)), "Mode");
        assert_eq!(cstr(call(FF_GET_PARAMETER_NAME, FFMixed::uint(5), null)), "Tint B");
        assert_eq!(call(FF_GET_PARAMETER_TYPE, FFMixed::uint(0), null).as_u32(), FF_TYPE_STANDARD);
        assert_eq!(call(FF_GET_PARAMETER_TYPE, FFMixed::uint(1), null).as_u32(), FF_TYPE_OPTION);
        assert_eq!(call(FF_GET_PARAMETER_TYPE, FFMixed::uint(3), null).as_u32(), FF_TYPE_RED);
        assert_eq!(call(FF_GET_PARAMETER_TYPE, FFMixed::uint(9), null).as_u32(), FF_FAIL);
        let default = |i| f32::from_bits(call(FF_GET_PARAMETER_DEFAULT, FFMixed::uint(i), null).as_u32());
        assert_eq!((default(0), default(1), default(2), default(4)), (0.5, 1.0, 0.4, 0.5));
        assert_eq!(call(FF_GET_NUM_PARAMETER_ELEMENTS, FFMixed::uint(1), null).as_u32(), 3);
        let args = GetParameterElementNameStruct { ParameterNumber: 1, ElementNumber: 2 };
        assert_eq!(cstr(call(FF_GET_PARAMETER_ELEMENT_NAME, FFMixed::ptr(&args), null)), "Three");
    }

    #[test]
    fn ranges() {
        let range = |i| {
            let mut args = GetRangeStruct { parameterNumber: i, range: RangeStruct { min: -1.0, max: -1.0 } };
            call(FF_GET_RANGE, FFMixed::ptr_mut(&mut args), std::ptr::null_mut());
            (args.range.min, args.range.max)
        };
        assert_eq!(range(0), (0.0, 1.0));
        assert_eq!(range(1), (0.0, 2.0));
    }

    #[test]
    fn group_string_is_written_without_overflow() {
        let mut buf = [b'#'; 8];
        let mut args = GetStringStruct {
            parameterNumber: 3,
            stringBuffer: StringBufferStruct { address: buf.as_mut_ptr().cast(), maxToWrite: 3 },
        };
        assert_eq!(call(FF_GET_PARAM_GROUP, FFMixed::ptr_mut(&mut args), std::ptr::null_mut()).as_u32(), FF_SUCCESS);
        assert_eq!(&buf, b"Col#####");
    }

    #[test]
    fn set_get_display_and_iterations() {
        let inst = new_instance();
        let set = |i, v: f32| {
            let s = SetParameterStruct { ParameterNumber: i, NewParameterValue: FFMixed::float(v) };
            call(FF_SET_PARAMETER, FFMixed::ptr(&s), inst).as_u32()
        };
        assert_eq!(set(0, 0.25), FF_SUCCESS);
        assert_eq!(cstr(call(FF_GET_PARAMETER_DISPLAY, FFMixed::uint(0), inst)), "0.50");
        assert_eq!(f32::from_bits(call(FF_GET_PARAMETER, FFMixed::uint(0), inst).as_u32()), 0.25);
        set(1, 2.0);
        assert_eq!(cstr(call(FF_GET_PARAMETER_DISPLAY, FFMixed::uint(1), inst)), "Three");
        set(2, 0.7);
        let i = unsafe { &*inst };
        // 7 steps: 7 runs of one step, ceil(7 / 3) = 3 runs of three; the last pass runs once.
        assert_eq!((i.iterations(0), i.iterations(1), i.iterations(2)), (7, 3, 1));
        assert_eq!(set(0, f32::NAN), FF_FAIL);
        assert_eq!(set(9, 0.0), FF_FAIL);
        free(inst);
    }

    #[test]
    fn input_status_and_no_gl() {
        let inst = new_instance();
        assert_eq!(call(FF_GET_INPUT_STATUS, FFMixed::uint(0), inst).as_u32(), FF_INPUT_INUSE);
        assert_eq!(call(FF_GET_INPUT_STATUS, FFMixed::uint(1), inst).as_u32(), FF_FAIL);
        let gl = ProcessOpenGLStruct { numInputTextures: 0, inputTextures: std::ptr::null_mut(), HostFBO: 0 };
        assert_eq!(call(FF_PROCESS_OPENGL, FFMixed::ptr(&gl), inst).as_u32(), FF_FAIL);
        free(inst);
        assert_eq!(call(FF_PROCESS_OPENGL, FFMixed::ptr(&gl), std::ptr::null_mut()).as_u32(), FF_FAIL);
    }

    #[test]
    fn ident_matching_is_whole_word() {
        assert!(uses_ident("x = amount * 2.0;", "amount"));
        assert!(!uses_ident("x = amountB + myamount;", "amount"));
        assert!(uses_ident("f(amount)", "amount"));
    }

    #[test]
    fn fragment_source_declares_params() {
        let src = fragment_source(&TEST_FX, 0);
        for line in ["uniform float amount;", "uniform int mode;", "uniform int steps;", "uniform vec3 tint;", "uniform sampler2D history;", "#line 1"] {
            assert!(src.contains(line), "{line}");
        }
    }

    #[test]
    fn buffers_are_listed_once_and_last_pass_cannot_store() {
        assert_eq!(TEST_FX.buffers(), vec!["history"]);
        static LAST_STORES: EffectDef = EffectDef {
            id: *b"LsSt",
            name: "Last Stores",
            short_name: c"Last",
            description: c"",
            about: c"",
            version: (0, 1),
            params: &[],
            passes: &[Pass::new("").storing("buf")],
            cache: Cache::new(),
        };
        assert!(LAST_STORES.validate().unwrap_err().contains("can't store"));
    }
}
