//! Minimal FFGL host for testing effect plugins without Resolume.
//!
//! Loads a plugin, creates an OpenGL 4.1 core context, feeds it a PNG as
//! the input clip, drives it through the calls Resolume makes, checks the
//! GL state rules the FFGL SDK enforces after every call, and saves the
//! output as a PNG.
//!
//!     fx-host <plugin.dll|.bundle> <input.png> <output.png> [options] [Param=value ...]
//!
//! Options:
//!     --frames N    frames to render (default 3; the last one is saved)
//!     --time T      host time of the first frame in seconds (default 0)
//!
//! `Param=value` sets a parameter by its Resolume name before rendering.
//! The value is what the host sends: 0..1 for sliders, 0/1 for toggles, or
//! an option's element name (or index). Exits non-zero if a check fails.

#[cfg(not(any(windows, target_os = "macos")))]
fn main() {
    eprintln!("fx-host only runs on Windows and macOS.");
}

#[cfg(any(windows, target_os = "macos"))]
fn main() {
    std::process::exit(match host::run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("FAILED: {e}");
            1
        }
    });
}

#[cfg(any(windows, target_os = "macos"))]
mod host {
    use std::ffi::{c_char, c_void, CStr};
    use std::path::PathBuf;
    use std::time::Instant;

    use glow::HasContext;

    #[cfg(target_os = "macos")]
    use super::cgl::create_gl_context;
    #[cfg(windows)]
    use super::wgl::create_gl_context;

    // FFGL values, repeated here so this tests the DLL boundary.
    const FF_GET_INFO: u32 = 0;
    const FF_DEINITIALISE: u32 = 2;
    const FF_GET_NUM_PARAMETERS: u32 = 4;
    const FF_GET_PARAMETER_NAME: u32 = 5;
    const FF_GET_PARAMETER_DEFAULT: u32 = 6;
    const FF_GET_PARAMETER_DISPLAY: u32 = 7;
    const FF_SET_PARAMETER: u32 = 8;
    const FF_GET_PLUGIN_CAPS: u32 = 10;
    const FF_GET_PARAMETER_TYPE: u32 = 15;
    const FF_PROCESS_OPENGL: u32 = 17;
    const FF_INSTANTIATE_GL: u32 = 18;
    const FF_DEINSTANTIATE_GL: u32 = 19;
    const FF_SET_TIME: u32 = 20;
    const FF_CONNECT: u32 = 21;
    const FF_GET_NUM_PARAMETER_ELEMENTS: u32 = 31;
    const FF_INITIALISE_V2: u32 = 34;
    const FF_GET_PARAMETER_ELEMENT_NAME: u32 = 35;
    const FF_CAP_MINIMUM_INPUT_FRAMES: u32 = 10;
    const FF_TYPE_OPTION: u32 = 11;
    const FF_FAIL: u32 = 0xFFFF_FFFF;
    const FF_SUCCESS: u32 = 0;
    const FF_EFFECT: u32 = 0;

    #[repr(C)]
    #[derive(Clone, Copy)]
    union FFMixed {
        u: u32,
        p: *mut c_void,
    }
    fn uint(v: u32) -> FFMixed {
        let mut m = FFMixed { p: std::ptr::null_mut() };
        m.u = v;
        m
    }
    fn ptr<T>(p: *const T) -> FFMixed {
        FFMixed { p: p as *mut c_void }
    }

    #[repr(C)]
    struct PluginInfo {
        api_major: u32,
        api_minor: u32,
        id: [u8; 4],
        name: [u8; 16],
        kind: u32,
    }
    #[repr(C)]
    struct Viewport {
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    }
    #[repr(C)]
    struct SetParam {
        index: u32,
        value: FFMixed,
    }
    #[repr(C)]
    struct Texture {
        width: u32,
        height: u32,
        hardware_width: u32,
        hardware_height: u32,
        handle: u32,
    }
    #[repr(C)]
    struct ProcessGl {
        num_inputs: u32,
        inputs: *mut *mut Texture,
        host_fbo: u32,
    }
    #[repr(C)]
    struct ElementName {
        param: u32,
        element: u32,
    }

    type PlugMain = unsafe extern "system" fn(u32, FFMixed, *mut c_void) -> FFMixed;
    type SetLogCallback = unsafe extern "system" fn(Option<unsafe extern "system" fn(*mut c_char)>);

    /// The library to load: a dll/dylib as given, or a bundle's executable.
    fn library_path(arg: PathBuf) -> PathBuf {
        if arg.extension().is_some_and(|e| e == "bundle") {
            let name = arg.file_stem().unwrap_or_default().to_owned();
            arg.join("Contents").join("MacOS").join(name)
        } else {
            arg
        }
    }

    unsafe extern "system" fn print_log(msg: *mut c_char) {
        println!("  plugin log: {}", CStr::from_ptr(msg).to_string_lossy());
    }

    struct Args {
        plugin: PathBuf,
        input: PathBuf,
        output: PathBuf,
        frames: u32,
        time: f64,
        sets: Vec<(String, String)>,
    }

    fn parse_args() -> Result<Args, String> {
        let usage = "usage: fx-host <plugin> <input.png> <output.png> [--frames N] [--time T] [Param=value ...]";
        let mut positional = Vec::new();
        let mut frames = 3;
        let mut time = 0.0;
        let mut sets = Vec::new();
        let mut it = std::env::args().skip(1);
        while let Some(a) = it.next() {
            match a.as_str() {
                "--frames" => frames = it.next().and_then(|v| v.parse().ok()).ok_or(usage)?,
                "--time" => time = it.next().and_then(|v| v.parse().ok()).ok_or(usage)?,
                _ => match a.split_once('=') {
                    Some((k, v)) => sets.push((k.to_owned(), v.to_owned())),
                    None => positional.push(PathBuf::from(a)),
                },
            }
        }
        let [plugin, input, output]: [PathBuf; 3] = positional.try_into().map_err(|_| usage)?;
        Ok(Args { plugin: library_path(plugin), input, output, frames: frames.max(1), time, sets })
    }

    pub fn run() -> Result<(), String> {
        let args = parse_args()?;
        let img = image::open(&args.input).map_err(|e| format!("read {}: {e}", args.input.display()))?.to_rgba8();
        let (w, h) = img.dimensions();

        unsafe {
            let gl = create_gl_context()?;
            println!("GL: {} on {}", gl.get_parameter_string(glow::VERSION), gl.get_parameter_string(glow::RENDERER));

            let lib = libloading::Library::new(&args.plugin).map_err(|e| format!("load {}: {e}", args.plugin.display()))?;
            let plug_main: libloading::Symbol<PlugMain> = lib.get(b"plugMain\0").map_err(|e| format!("plugMain: {e}"))?;
            let set_log: libloading::Symbol<SetLogCallback> =
                lib.get(b"SetLogCallback\0").map_err(|e| format!("SetLogCallback: {e}"))?;
            set_log(Some(print_log));
            let call = |code, input, inst| plug_main(code, input, inst);
            let null = std::ptr::null_mut();
            let cstr = |m: FFMixed| CStr::from_ptr(m.p as *const c_char).to_string_lossy().into_owned();

            // ── Info and parameters, as Resolume reads them at scan time ──
            let info = &*(call(FF_GET_INFO, uint(0), null).p as *const PluginInfo);
            let name = String::from_utf8_lossy(&info.name).trim_end_matches('\0').to_owned();
            println!(
                "Plugin: {name:?} id {:?} API {}.{}",
                String::from_utf8_lossy(&info.id),
                info.api_major,
                info.api_minor
            );
            check(info.kind == FF_EFFECT, "plugin is an effect")?;
            check(call(FF_GET_PLUGIN_CAPS, uint(FF_CAP_MINIMUM_INPUT_FRAMES), null).u == 1, "takes one input")?;
            check(call(FF_INITIALISE_V2, uint(0), null).u == FF_SUCCESS, "initialise")?;
            let num = call(FF_GET_NUM_PARAMETERS, uint(0), null).u;
            let mut names = Vec::new();
            for i in 0..num {
                let pname = cstr(call(FF_GET_PARAMETER_NAME, uint(i), null));
                let ty = call(FF_GET_PARAMETER_TYPE, uint(i), null).u;
                let def = f32::from_bits(call(FF_GET_PARAMETER_DEFAULT, uint(i), null).u);
                println!("  param {i:2}: {pname:<14} type {ty:3} default {def:.3}");
                names.push((pname, ty));
            }

            // ── Input clip texture and host framebuffer ──
            let make_tex = |data: Option<&[u8]>| -> Result<glow::Texture, String> {
                let tex = gl.create_texture()?;
                gl.bind_texture(glow::TEXTURE_2D, Some(tex));
                gl.tex_image_2d(
                    glow::TEXTURE_2D, 0, glow::RGBA8 as i32, w as i32, h as i32, 0,
                    glow::RGBA, glow::UNSIGNED_BYTE, glow::PixelUnpackData::Slice(data),
                );
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
                gl.bind_texture(glow::TEXTURE_2D, None);
                Ok(tex)
            };
            // GL rows start at the bottom.
            let input_tex = make_tex(Some(&flip_rows(img.as_raw(), w, h)))?;
            let out_tex = make_tex(None)?;
            let fbo = gl.create_framebuffer()?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(out_tex), 0);
            check(gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE, "host fbo complete")?;
            gl.viewport(0, 0, w as i32, h as i32);
            validate_state(&gl, fbo, "before instantiate")?;

            // ── Instance ──
            let vp = Viewport { x: 0, y: 0, w, h };
            let inst = call(FF_INSTANTIATE_GL, ptr(&vp), null).p;
            check(!inst.is_null() && inst as usize != FF_FAIL as usize, "instantiate (see plugin log)")?;
            validate_state(&gl, fbo, "after instantiate")?;
            check(call(FF_CONNECT, uint(0), inst).u == FF_SUCCESS, "connect")?;

            for (key, value) in &args.sets {
                let index = names.iter().position(|(n, _)| n == key).ok_or(format!("no parameter {key:?}"))? as u32;
                let host_value = if names[index as usize].1 == FF_TYPE_OPTION {
                    let count = call(FF_GET_NUM_PARAMETER_ELEMENTS, uint(index), null).u;
                    let by_name = (0..count).find(|&e| {
                        let args = ElementName { param: index, element: e };
                        cstr(call(FF_GET_PARAMETER_ELEMENT_NAME, ptr(&args), null)).eq_ignore_ascii_case(value)
                    });
                    match by_name {
                        Some(e) => e as f32,
                        None => value.parse().map_err(|_| format!("{key}: no element {value:?}"))?,
                    }
                } else {
                    value.parse().map_err(|_| format!("{key}: {value:?} isn't a number"))?
                };
                let set = SetParam { index, value: uint(host_value.to_bits()) };
                check(call(FF_SET_PARAMETER, ptr(&set), inst).u == FF_SUCCESS, &format!("set {key}"))?;
                println!("  set {key} = {}", cstr(call(FF_GET_PARAMETER_DISPLAY, uint(index), inst)));
            }

            let mut tex = Texture { width: w, height: h, hardware_width: w, hardware_height: h, handle: input_tex.0.get() };
            let mut inputs = [&mut tex as *mut Texture];
            let mut times = Vec::new();
            for frame in 0..args.frames {
                let t = args.time + frame as f64 / 60.0;
                call(FF_SET_TIME, ptr(&t), inst);
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
                gl.finish();
                let start = Instant::now();
                let pgl = ProcessGl { num_inputs: 1, inputs: inputs.as_mut_ptr(), host_fbo: fbo.0.get() };
                check(call(FF_PROCESS_OPENGL, ptr(&pgl), inst).u == FF_SUCCESS, "process")?;
                gl.finish();
                times.push(start.elapsed().as_secs_f64() * 1000.0);
                validate_state(&gl, fbo, &format!("after process (frame {frame})"))?;
            }
            // The first frame includes driver warm-up.
            let steady = if times.len() > 1 { &times[1..] } else { &times[..] };
            println!("Frame time at {w}x{h}: {:.2} ms (first frame {:.2} ms)", steady.iter().sum::<f64>() / steady.len() as f64, times[0]);

            let mut pixels = vec![0u8; (w * h * 4) as usize];
            gl.read_pixels(
                0, 0, w as i32, h as i32, glow::RGBA, glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut pixels)),
            );
            let out = image::RgbaImage::from_raw(w, h, flip_rows(&pixels, w, h)).ok_or("image buffer size")?;
            out.save(&args.output).map_err(|e| format!("save {}: {e}", args.output.display()))?;
            println!("Wrote {}", args.output.display());

            check(call(FF_DEINSTANTIATE_GL, uint(0), inst).u == FF_SUCCESS, "deinstantiate")?;
            validate_state(&gl, fbo, "after deinstantiate")?;
            check(call(FF_DEINITIALISE, uint(0), null).u == FF_SUCCESS, "deinitialise")?;
            check(gl.get_error() == glow::NO_ERROR, "no GL errors")?;
            println!("All checks passed.");
            Ok(())
        }
    }

    fn flip_rows(pixels: &[u8], w: u32, h: u32) -> Vec<u8> {
        let row = (w * 4) as usize;
        (0..h as usize).rev().flat_map(|y| &pixels[y * row..(y + 1) * row]).copied().collect()
    }

    fn check(ok: bool, what: &str) -> Result<(), String> {
        if ok { Ok(()) } else { Err(format!("{what} failed")) }
    }

    /// Port of the FFGL SDK's `ValidateContextState`, plus the host FBO.
    unsafe fn validate_state(gl: &glow::Context, fbo: glow::Framebuffer, when: &str) -> Result<(), String> {
        let int = |p| gl.get_parameter_i32(p);
        let mut bad = Vec::new();
        let mut expect = |name: &str, got: i32, want: i32| {
            if got != want {
                bad.push(format!("{name}={got:#x} (want {want:#x})"));
            }
        };
        expect("CURRENT_PROGRAM", int(glow::CURRENT_PROGRAM), 0);
        expect("ACTIVE_TEXTURE", int(glow::ACTIVE_TEXTURE), glow::TEXTURE0 as i32);
        for unit in 0..int(glow::MAX_TEXTURE_IMAGE_UNITS) as u32 {
            gl.active_texture(glow::TEXTURE0 + unit);
            let b = int(glow::TEXTURE_BINDING_2D);
            if b != 0 {
                expect(&format!("TEXTURE_BINDING_2D[{unit}]"), b, 0);
            }
        }
        gl.active_texture(glow::TEXTURE0);
        expect("ARRAY_BUFFER_BINDING", int(glow::ARRAY_BUFFER_BINDING), 0);
        expect("ELEMENT_ARRAY_BUFFER_BINDING", int(glow::ELEMENT_ARRAY_BUFFER_BINDING), 0);
        expect("UNIFORM_BUFFER_BINDING", int(glow::UNIFORM_BUFFER_BINDING), 0);
        expect("PIXEL_PACK_BUFFER_BINDING", int(glow::PIXEL_PACK_BUFFER_BINDING), 0);
        expect("PIXEL_UNPACK_BUFFER_BINDING", int(glow::PIXEL_UNPACK_BUFFER_BINDING), 0);
        expect("VERTEX_ARRAY_BINDING", int(glow::VERTEX_ARRAY_BINDING), 0);
        expect("BLEND", gl.is_enabled(glow::BLEND) as i32, 0);
        expect("CULL_FACE", gl.is_enabled(glow::CULL_FACE) as i32, 0);
        expect("DEPTH_TEST", gl.is_enabled(glow::DEPTH_TEST) as i32, 0);
        expect("FRAMEBUFFER_BINDING (host fbo)", int(glow::FRAMEBUFFER_BINDING), fbo.0.get() as i32);
        let err = gl.get_error();
        expect("glGetError", err as i32, glow::NO_ERROR as i32);
        if bad.is_empty() { Ok(()) } else { Err(format!("GL state {when}: {}", bad.join(", "))) }
    }
}

#[cfg(windows)]
mod wgl {
    use std::ffi::c_void;

    use windows_sys::Win32::Graphics::Gdi::{GetDC, HDC};
    use windows_sys::Win32::Graphics::OpenGL::*;
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    type CreateContextAttribs = unsafe extern "system" fn(HDC, HGLRC, *const i32) -> HGLRC;

    fn check(ok: bool, what: &str) -> Result<(), String> {
        if ok { Ok(()) } else { Err(format!("{what} failed")) }
    }

    /// Hidden window + OpenGL 4.1 core context (what Resolume gives plugins).
    pub unsafe fn create_gl_context() -> Result<glow::Context, String> {
        let instance = GetModuleHandleA(std::ptr::null());
        let class_name = c"FfglFxHost".as_ptr().cast();
        let class = WNDCLASSA {
            style: CS_OWNDC,
            lpfnWndProc: Some(DefWindowProcA),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
        };
        RegisterClassA(&class);
        let hwnd = CreateWindowExA(
            0, class_name, class_name, WS_OVERLAPPEDWINDOW, 0, 0, 64, 64,
            std::ptr::null_mut(), std::ptr::null_mut(), instance, std::ptr::null(),
        );
        check(!hwnd.is_null(), "CreateWindowExA")?;
        let hdc = GetDC(hwnd);

        let pfd = PIXELFORMATDESCRIPTOR {
            nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
            nVersion: 1,
            dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            iPixelType: PFD_TYPE_RGBA,
            cColorBits: 32,
            cDepthBits: 24,
            cStencilBits: 8,
            ..std::mem::zeroed()
        };
        let format = ChoosePixelFormat(hdc, &pfd);
        check(format != 0 && SetPixelFormat(hdc, format, &pfd) != 0, "pixel format")?;

        // A legacy context is needed to look up wglCreateContextAttribsARB.
        let legacy = wglCreateContext(hdc);
        check(!legacy.is_null() && wglMakeCurrent(hdc, legacy) != 0, "legacy GL context")?;
        let create_attribs = wglGetProcAddress(c"wglCreateContextAttribsARB".as_ptr().cast())
            .ok_or("wglCreateContextAttribsARB unavailable")?;
        let create_attribs: CreateContextAttribs = std::mem::transmute(create_attribs);

        const WGL_CONTEXT_MAJOR_VERSION_ARB: i32 = 0x2091;
        const WGL_CONTEXT_MINOR_VERSION_ARB: i32 = 0x2092;
        const WGL_CONTEXT_PROFILE_MASK_ARB: i32 = 0x9126;
        const WGL_CONTEXT_CORE_PROFILE_BIT_ARB: i32 = 0x1;
        let attribs = [
            WGL_CONTEXT_MAJOR_VERSION_ARB, 4,
            WGL_CONTEXT_MINOR_VERSION_ARB, 1,
            WGL_CONTEXT_PROFILE_MASK_ARB, WGL_CONTEXT_CORE_PROFILE_BIT_ARB,
            0,
        ];
        let core = create_attribs(hdc, std::ptr::null_mut(), attribs.as_ptr());
        check(!core.is_null(), "OpenGL 4.1 core context")?;
        wglMakeCurrent(hdc, core);
        wglDeleteContext(legacy);

        Ok(glow::Context::from_loader_function_cstr(|name| {
            let p = wglGetProcAddress(name.as_ptr().cast()).map_or(0, |f| f as usize);
            if !matches!(p, 0 | 1 | 2 | 3 | usize::MAX) {
                return p as *const c_void;
            }
            let gl32 = GetModuleHandleA(c"opengl32.dll".as_ptr().cast());
            GetProcAddress(gl32, name.as_ptr().cast()).map_or(std::ptr::null(), |f| f as *const c_void)
        }))
    }
}

#[cfg(target_os = "macos")]
mod cgl {
    use std::ffi::{c_char, c_int, c_void};

    type CGLPixelFormatObj = *mut c_void;
    type CGLContextObj = *mut c_void;
    type CGLError = c_int;

    #[link(name = "OpenGL", kind = "framework")]
    extern "C" {
        fn CGLChoosePixelFormat(attribs: *const c_int, pix: *mut CGLPixelFormatObj, npix: *mut c_int) -> CGLError;
        fn CGLCreateContext(pix: CGLPixelFormatObj, share: CGLContextObj, ctx: *mut CGLContextObj) -> CGLError;
        fn CGLDestroyPixelFormat(pix: CGLPixelFormatObj) -> CGLError;
        fn CGLSetCurrentContext(ctx: CGLContextObj) -> CGLError;
    }
    extern "C" {
        fn dlopen(path: *const c_char, mode: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }

    // Values from <OpenGL/CGLTypes.h>.
    const KCGLPFA_ALLOW_OFFLINE_RENDERERS: c_int = 96;
    const KCGLPFA_OPENGL_PROFILE: c_int = 99;
    const KCGL_OGLP_VERSION_GL4_CORE: c_int = 0x4100;
    const KCGLPFA_RENDERER_ID: c_int = 70;
    const KCGL_RENDERER_GENERIC_FLOAT_ID: c_int = 0x0002_0400;

    /// Offscreen OpenGL 4.1 core context; falls back to Apple's software
    /// renderer on machines without a GPU, such as CI runners.
    pub unsafe fn create_gl_context() -> Result<glow::Context, String> {
        let hardware = [KCGLPFA_OPENGL_PROFILE, KCGL_OGLP_VERSION_GL4_CORE, KCGLPFA_ALLOW_OFFLINE_RENDERERS, 0];
        let software = [
            KCGLPFA_OPENGL_PROFILE, KCGL_OGLP_VERSION_GL4_CORE,
            KCGLPFA_RENDERER_ID, KCGL_RENDERER_GENERIC_FLOAT_ID,
            0,
        ];
        let mut pix: CGLPixelFormatObj = std::ptr::null_mut();
        let mut npix = 0;
        for attribs in [&hardware[..], &software[..]] {
            if CGLChoosePixelFormat(attribs.as_ptr(), &mut pix, &mut npix) == 0 && !pix.is_null() {
                break;
            }
        }
        if pix.is_null() {
            return Err("no OpenGL 4.1 core pixel format".into());
        }
        let mut ctx: CGLContextObj = std::ptr::null_mut();
        let err = CGLCreateContext(pix, std::ptr::null_mut(), &mut ctx);
        CGLDestroyPixelFormat(pix);
        if err != 0 || ctx.is_null() {
            return Err(format!("CGLCreateContext failed ({err})"));
        }
        if CGLSetCurrentContext(ctx) != 0 {
            return Err("CGLSetCurrentContext failed".into());
        }

        const RTLD_LAZY: c_int = 0x1;
        let opengl = dlopen(c"/System/Library/Frameworks/OpenGL.framework/OpenGL".as_ptr(), RTLD_LAZY);
        if opengl.is_null() {
            return Err("could not open the OpenGL framework".into());
        }
        Ok(glow::Context::from_loader_function_cstr(|name| dlsym(opengl, name.as_ptr()) as *const c_void))
    }
}
