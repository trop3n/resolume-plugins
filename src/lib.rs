//! Rusci Scope: an FFGL 2.1 source plugin for Resolume that draws
//! oscilloscope-style vector graphics with the rusci-render visualiser.
//!
//! Resolume loads the DLL and talks to it only through `plugMain`.

pub mod ffi;
mod gl_loader;
pub mod instance;
pub mod params;
pub mod scene;

use std::ffi::{c_char, c_void, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};

use ffi::*;
use instance::Instance;
use params::{Kind, PARAMS};

// ── Plugin identity ─────────────────────────────────────────────────

const fn pad16(name: &[u8]) -> [u8; 16] {
    let mut out = [0u8; 16];
    let mut i = 0;
    while i < name.len() && i < 16 {
        out[i] = name[i];
        i += 1;
    }
    out
}

static PLUGIN_INFO: PluginInfoStruct = PluginInfoStruct {
    APIMajorVersion: 2,
    APIMinorVersion: 1,
    PluginUniqueID: *b"RsSc",
    PluginName: pad16(b"Rusci Scope"),
    PluginType: FF_SOURCE,
};

/// Raw pointers aren't `Sync`; these only point at static strings.
struct ExtendedInfo(PluginExtendedInfoStruct);
unsafe impl Sync for ExtendedInfo {}

static EXTENDED_INFO: ExtendedInfo = ExtendedInfo(PluginExtendedInfoStruct {
    PluginMajorVersion: 0,
    PluginMinorVersion: 1,
    Description: c"Oscilloscope vector graphics, from rusci-render".as_ptr(),
    About: c"rusci-render (GPL-3.0), after osci-render by James Ball".as_ptr(),
    FreeFrameExtendedDataSize: 0,
    FreeFrameExtendedDataBlock: std::ptr::null_mut(),
});

static SHORT_NAME: &CStr = c"RScope";

// ── Host logging ────────────────────────────────────────────────────

static LOG_CALLBACK: AtomicUsize = AtomicUsize::new(0);

/// Send a line to Resolume's log, if it gave us a callback.
fn log(msg: &str) {
    let f = LOG_CALLBACK.load(Ordering::Acquire);
    if f == 0 {
        return;
    }
    if let Ok(line) = CString::new(format!("[Rusci Scope] {msg}")) {
        let f: unsafe extern "system" fn(*mut c_char) = unsafe { std::mem::transmute(f) };
        unsafe { f(line.as_ptr() as *mut c_char) };
    }
}

#[no_mangle]
pub extern "system" fn SetLogCallback(callback: PFNLog) {
    LOG_CALLBACK.store(callback.map_or(0, |f| f as usize), Ordering::Release);
}

// ── Entry point ─────────────────────────────────────────────────────

#[no_mangle]
pub extern "system" fn plugMain(code: u32, input: FFMixed, instance: *mut c_void) -> FFMixed {
    // A panic must never unwind into Resolume.
    catch_unwind(AssertUnwindSafe(|| unsafe { dispatch(code, input, instance.cast()) })).unwrap_or_else(|e| {
        let msg = e.downcast_ref::<&str>().copied().or_else(|| e.downcast_ref::<String>().map(|s| s.as_str()));
        log(&format!("panic in function {code}: {}", msg.unwrap_or("unknown")));
        FFMixed::fail()
    })
}

/// # Safety
/// `instance` is null or a pointer returned by `FF_INSTANTIATE_GL`, and
/// `input` matches what the FFGL spec says `code` takes.
pub unsafe fn dispatch(code: u32, input: FFMixed, instance: *mut Instance) -> FFMixed {
    let inst = instance.as_mut();
    let index = || input.as_u32() as usize;
    let ok = |b: bool| FFMixed::uint(if b { FF_SUCCESS } else { FF_FAIL });

    match code {
        FF_GET_INFO => FFMixed::ptr(&PLUGIN_INFO),
        FF_GET_EXTENDED_INFO => FFMixed::ptr(&EXTENDED_INFO.0),
        FF_GET_PLUGIN_SHORT_NAME => FFMixed::ptr(SHORT_NAME.as_ptr()),
        FF_INITIALISE_V2 | FF_DEINITIALISE => ok(true),

        FF_GET_PLUGIN_CAPS => FFMixed::uint(match input.as_u32() {
            FF_CAP_SET_TIME => FF_TRUE,
            // Source plugin: no input clips.
            FF_CAP_MINIMUM_INPUT_FRAMES | FF_CAP_MAXIMUM_INPUT_FRAMES => 0,
            _ => FF_FALSE,
        }),
        FF_ENABLE_PLUGIN_CAP => FFMixed::fail(),

        // ── Parameter metadata (no instance needed) ──
        FF_GET_NUM_PARAMETERS => FFMixed::uint(PARAMS.len() as u32),
        FF_GET_PARAMETER_NAME => match PARAMS.get(index()) {
            Some(p) => FFMixed::ptr(p.name.as_ptr()),
            None => FFMixed::ptr(std::ptr::null::<c_char>()),
        },
        FF_GET_PARAMETER_DEFAULT => PARAMS.get(index()).map_or(FFMixed::fail(), |p| FFMixed::float(p.host_default())),
        FF_GET_PARAMETER_TYPE => PARAMS.get(index()).map_or(FFMixed::fail(), |p| FFMixed::uint(p.ffgl_type())),
        FF_GET_PARAMETER_USAGE => FFMixed::uint(FF_USAGE_STANDARD),
        FF_GET_PRAMETER_VISIBILITY => FFMixed::uint(FF_TRUE),
        FF_GET_NUM_PARAMETER_ELEMENTS => {
            PARAMS.get(index()).map_or(FFMixed::fail(), |p| FFMixed::uint(p.elements().len() as u32))
        }
        FF_GET_PARAMETER_ELEMENT_NAME => {
            let args = &*(input.as_ptr() as *const GetParameterElementNameStruct);
            match PARAMS.get(args.ParameterNumber as usize).and_then(|p| p.elements().get(args.ElementNumber as usize)) {
                Some(name) => FFMixed::ptr(name.as_ptr()),
                None => FFMixed::ptr(std::ptr::null::<c_char>()),
            }
        }
        FF_GET_PARAMETER_ELEMENT_VALUE => {
            let args = &*(input.as_ptr() as *const GetParameterElementValueStruct);
            match PARAMS.get(args.ParameterNumber as usize) {
                Some(p) if (args.ElementNumber as usize) < p.elements().len() => {
                    FFMixed::float(args.ElementNumber as f32)
                }
                _ => FFMixed::fail(),
            }
        }
        FF_GET_NUM_ELEMENT_SEPARATORS | FF_GET_NUM_FILE_PARAMETER_EXTENSIONS => FFMixed::uint(0),
        FF_GET_RANGE => {
            let args = &mut *(input.as_ptr() as *mut GetRangeStruct);
            let Some(p) = PARAMS.get(args.parameterNumber as usize) else {
                return FFMixed::fail();
            };
            let max = match p.kind {
                Kind::Option { elements } => (elements.len() - 1) as f32,
                _ => 1.0,
            };
            args.range = RangeStruct { min: 0.0, max };
            ok(true)
        }
        FF_GET_PARAM_GROUP | FF_GET_PARAM_DISPLAY_NAME => {
            let args = &mut *(input.as_ptr() as *mut GetStringStruct);
            let Some(p) = PARAMS.get(args.parameterNumber as usize) else {
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
            Some(&vp) => instantiate(vp),
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
        FF_SET_PARAMETER => match inst {
            Some(i) => {
                let args = &*(input.as_ptr() as *const SetParameterStruct);
                let value = f32::from_bits(args.NewParameterValue.as_u32());
                ok(i.params.set_host(args.ParameterNumber as usize, value))
            }
            None => FFMixed::fail(),
        },
        FF_GET_PARAMETER => {
            inst.and_then(|i| i.params.get_host(index())).map_or(FFMixed::fail(), FFMixed::float)
        }
        FF_GET_PARAMETER_DISPLAY => match inst.and_then(|i| Some((i.params.display(index())?, i))) {
            Some((text, i)) => {
                i.display = CString::new(text).unwrap_or_default();
                FFMixed::ptr(i.display.as_ptr())
            }
            None => FFMixed::fail(),
        },
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
            (Some(i), Some(gl)) => ok(i.process(gl.HostFBO)),
            _ => FFMixed::fail(),
        },

        // Source plugin has no inputs; buffer params aren't used.
        FF_GET_INPUT_STATUS | FF_SET_PARAMETER_ELEMENT_VALUE => FFMixed::fail(),
        _ => FFMixed::fail(),
    }
}

unsafe fn instantiate(viewport: FFGLViewportStruct) -> FFMixed {
    let mut inst = Box::new(Instance::new(viewport));
    // Catch here, not just in plugMain, so a failed init doesn't leak the instance.
    match catch_unwind(AssertUnwindSafe(|| unsafe { inst.init_gl() })) {
        Ok(()) => FFMixed::ptr(Box::into_raw(inst)),
        Err(_) => {
            log("failed to create GL resources (needs an OpenGL 3.3+ core context)");
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

    fn call(code: u32, input: FFMixed, inst: *mut Instance) -> FFMixed {
        unsafe { dispatch(code, input, inst) }
    }

    /// An instance without GL, the way `FF_INSTANTIATE_GL` would hand one out.
    fn new_instance() -> *mut Instance {
        Box::into_raw(Box::new(Instance::new(FFGLViewportStruct { x: 0, y: 0, width: 640, height: 360 })))
    }

    fn free(inst: *mut Instance) {
        drop(unsafe { Box::from_raw(inst) });
    }

    fn cstr(m: FFMixed) -> String {
        unsafe { CStr::from_ptr(m.as_ptr() as *const c_char) }.to_string_lossy().into_owned()
    }

    #[test]
    fn info_identifies_a_source_plugin() {
        let info = unsafe { &*(call(FF_GET_INFO, FFMixed::uint(0), std::ptr::null_mut()).as_ptr() as *const PluginInfoStruct) };
        assert_eq!((info.APIMajorVersion, info.APIMinorVersion), (2, 1));
        assert_eq!(&info.PluginUniqueID, b"RsSc");
        assert_eq!(&info.PluginName[..11], b"Rusci Scope");
        assert_eq!(info.PluginType, FF_SOURCE);
        assert_eq!(cstr(call(FF_GET_PLUGIN_SHORT_NAME, FFMixed::uint(0), std::ptr::null_mut())), "RScope");
    }

    #[test]
    fn caps_say_source_with_time() {
        let cap = |c| call(FF_GET_PLUGIN_CAPS, FFMixed::uint(c), std::ptr::null_mut()).as_u32();
        assert_eq!(cap(FF_CAP_SET_TIME), FF_TRUE);
        assert_eq!(cap(FF_CAP_MINIMUM_INPUT_FRAMES), 0);
        assert_eq!(cap(FF_CAP_MAXIMUM_INPUT_FRAMES), 0);
        assert_eq!(cap(FF_CAP_TOP_LEFT_TEXTURE_ORIENTATION), FF_FALSE);
    }

    #[test]
    fn parameter_metadata() {
        let null = std::ptr::null_mut();
        assert_eq!(call(FF_GET_NUM_PARAMETERS, FFMixed::uint(0), null).as_u32(), 12);
        assert_eq!(cstr(call(FF_GET_PARAMETER_NAME, FFMixed::uint(0), null)), "Shape");
        assert_eq!(call(FF_GET_PARAMETER_TYPE, FFMixed::uint(0), null).as_u32(), FF_TYPE_OPTION);
        assert_eq!(call(FF_GET_PARAMETER_TYPE, FFMixed::uint(1), null).as_u32(), FF_TYPE_STANDARD);
        assert!(call(FF_GET_PARAMETER_NAME, FFMixed::uint(99), null).as_ptr().is_null());
        assert_eq!(call(FF_GET_PARAMETER_TYPE, FFMixed::uint(99), null).as_u32(), FF_FAIL);

        let default = f32::from_bits(call(FF_GET_PARAMETER_DEFAULT, FFMixed::uint(params::PERSISTENCE as u32), null).as_u32());
        assert_eq!(default, 0.5);
    }

    #[test]
    fn option_elements() {
        let null = std::ptr::null_mut();
        assert_eq!(call(FF_GET_NUM_PARAMETER_ELEMENTS, FFMixed::uint(0), null).as_u32(), 5);
        let args = GetParameterElementNameStruct { ParameterNumber: 0, ElementNumber: 3 };
        assert_eq!(cstr(call(FF_GET_PARAMETER_ELEMENT_NAME, FFMixed::ptr(&args), null)), "Star");
        let args = GetParameterElementValueStruct { ParameterNumber: 0, ElementNumber: 3 };
        assert_eq!(f32::from_bits(call(FF_GET_PARAMETER_ELEMENT_VALUE, FFMixed::ptr(&args), null).as_u32()), 3.0);
        let args = GetParameterElementValueStruct { ParameterNumber: 0, ElementNumber: 9 };
        assert_eq!(call(FF_GET_PARAMETER_ELEMENT_VALUE, FFMixed::ptr(&args), null).as_u32(), FF_FAIL);
    }

    #[test]
    fn group_string_is_written_without_overflow() {
        let mut buf = [b'#'; 8];
        let mut args = GetStringStruct {
            parameterNumber: params::GLOW as u32,
            stringBuffer: StringBufferStruct { address: buf.as_mut_ptr().cast(), maxToWrite: 3 },
        };
        assert_eq!(call(FF_GET_PARAM_GROUP, FFMixed::ptr_mut(&mut args), std::ptr::null_mut()).as_u32(), FF_SUCCESS);
        assert_eq!(&buf, b"Bea#####");
    }

    #[test]
    fn range_for_option_covers_elements() {
        let mut args = GetRangeStruct { parameterNumber: 0, range: RangeStruct { min: -1.0, max: -1.0 } };
        call(FF_GET_RANGE, FFMixed::ptr_mut(&mut args), std::ptr::null_mut());
        assert_eq!((args.range.min, args.range.max), (0.0, 4.0));
    }

    #[test]
    fn set_get_and_display_round_trip() {
        let inst = new_instance();
        let set = SetParameterStruct { ParameterNumber: params::SHAPE as u32, NewParameterValue: FFMixed::float(3.0) };
        assert_eq!(call(FF_SET_PARAMETER, FFMixed::ptr(&set), inst).as_u32(), FF_SUCCESS);
        assert_eq!(f32::from_bits(call(FF_GET_PARAMETER, FFMixed::uint(0), inst).as_u32()), 3.0);
        assert_eq!(cstr(call(FF_GET_PARAMETER_DISPLAY, FFMixed::uint(0), inst)), "Star");

        let set = SetParameterStruct { ParameterNumber: params::FOCUS as u32, NewParameterValue: FFMixed::float(1.0) };
        call(FF_SET_PARAMETER, FFMixed::ptr(&set), inst);
        assert_eq!(cstr(call(FF_GET_PARAMETER_DISPLAY, FFMixed::uint(params::FOCUS as u32), inst)), "0.0200");
        free(inst);
    }

    #[test]
    fn per_instance_calls_fail_without_instance() {
        let null = std::ptr::null_mut();
        let set = SetParameterStruct { ParameterNumber: 0, NewParameterValue: FFMixed::float(1.0) };
        assert_eq!(call(FF_SET_PARAMETER, FFMixed::ptr(&set), null).as_u32(), FF_FAIL);
        assert_eq!(call(FF_GET_PARAMETER, FFMixed::uint(0), null).as_u32(), FF_FAIL);
        let gl = ProcessOpenGLStruct { numInputTextures: 0, inputTextures: std::ptr::null_mut(), HostFBO: 0 };
        assert_eq!(call(FF_PROCESS_OPENGL, FFMixed::ptr(&gl), null).as_u32(), FF_FAIL);
    }

    #[test]
    fn process_without_gl_fails_cleanly() {
        let inst = new_instance();
        let gl = ProcessOpenGLStruct { numInputTextures: 0, inputTextures: std::ptr::null_mut(), HostFBO: 0 };
        assert_eq!(call(FF_PROCESS_OPENGL, FFMixed::ptr(&gl), inst).as_u32(), FF_FAIL);
        free(inst);
    }

    #[test]
    fn unknown_code_fails() {
        assert_eq!(call(9999, FFMixed::uint(0), std::ptr::null_mut()).as_u32(), FF_FAIL);
    }
}
