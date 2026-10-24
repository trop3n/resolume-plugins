//! Minimal FFGL host for testing rusci_scope.dll without Resolume.
//!
//! It loads the DLL, creates an OpenGL 4.1 core context on a hidden window,
//! drives the plugin through the calls Resolume makes, checks the GL state
//! rules the FFGL SDK enforces, and saves one PNG per shape.
//!
//!     cargo run --release -p osci-ffgl --example ffgl_host [path\to\rusci_scope.dll]
//!
//! Output goes to target\ffgl-host\. Exits non-zero if any check fails.

#[cfg(not(windows))]
fn main() {
    eprintln!("ffgl_host only runs on Windows.");
}

#[cfg(windows)]
fn main() {
    std::process::exit(match windows_host::run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("FAILED: {e}");
            1
        }
    });
}

#[cfg(windows)]
mod windows_host {
    use std::ffi::{c_char, c_void, CStr};
    use std::path::PathBuf;

    use glow::HasContext;
    use windows_sys::Win32::Graphics::Gdi::{GetDC, HDC};
    use windows_sys::Win32::Graphics::OpenGL::*;
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    // FFGL values (same as the plugin's ffi.rs, repeated so this tests the DLL boundary).
    const FF_GET_INFO: u32 = 0;
    const FF_GET_NUM_PARAMETERS: u32 = 4;
    const FF_GET_PARAMETER_NAME: u32 = 5;
    const FF_GET_PARAMETER_DEFAULT: u32 = 6;
    const FF_GET_PARAMETER_DISPLAY: u32 = 7;
    const FF_SET_PARAMETER: u32 = 8;
    const FF_GET_PARAMETER_TYPE: u32 = 15;
    const FF_PROCESS_OPENGL: u32 = 17;
    const FF_INSTANTIATE_GL: u32 = 18;
    const FF_DEINSTANTIATE_GL: u32 = 19;
    const FF_SET_TIME: u32 = 20;
    const FF_CONNECT: u32 = 21;
    const FF_DEINITIALISE: u32 = 2;
    const FF_INITIALISE_V2: u32 = 34;
    const FF_FAIL: u32 = 0xFFFF_FFFF;
    const FF_SUCCESS: u32 = 0;

    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 360;
    const FRAMES_PER_SHAPE: u32 = 30;
    const SHAPES: [&str; 5] = ["circle", "square", "triangle", "star", "lissajous"];

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
    struct ProcessGl {
        num_inputs: u32,
        inputs: *mut c_void,
        host_fbo: u32,
    }

    type PlugMain = unsafe extern "system" fn(u32, FFMixed, *mut c_void) -> FFMixed;
    type SetLogCallback = unsafe extern "system" fn(Option<unsafe extern "system" fn(*mut c_char)>);
    type CreateContextAttribs = unsafe extern "system" fn(HDC, HGLRC, *const i32) -> HGLRC;

    unsafe extern "system" fn print_log(msg: *mut c_char) {
        println!("  plugin log: {}", CStr::from_ptr(msg).to_string_lossy());
    }

    pub fn run() -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        // exe is target\<profile>\examples\ffgl_host.exe
        let profile_dir = exe.parent().and_then(|p| p.parent()).ok_or("unexpected exe location")?;
        let dll = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| profile_dir.join("rusci_scope.dll"));
        let out_dir = profile_dir.parent().ok_or("unexpected exe location")?.join("ffgl-host");
        std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

        unsafe {
            let gl = create_gl_context()?;
            println!("GL: {}", gl.get_parameter_string(glow::VERSION));
            println!("GPU: {}", gl.get_parameter_string(glow::RENDERER));

            println!("Loading {}", dll.display());
            let lib = libloading::Library::new(&dll).map_err(|e| format!("load dll: {e}"))?;
            let plug_main: libloading::Symbol<PlugMain> = lib.get(b"plugMain\0").map_err(|e| format!("plugMain: {e}"))?;
            let set_log: libloading::Symbol<SetLogCallback> =
                lib.get(b"SetLogCallback\0").map_err(|e| format!("SetLogCallback: {e}"))?;
            set_log(Some(print_log));
            let call = |code, input, inst| plug_main(code, input, inst);

            // ── Info and parameters, as Resolume reads them at scan time ──
            let info = &*(call(FF_GET_INFO, uint(0), std::ptr::null_mut()).p as *const PluginInfo);
            println!(
                "Plugin: {:?} id {:?} API {}.{} type {}",
                String::from_utf8_lossy(&info.name).trim_end_matches('\0'),
                String::from_utf8_lossy(&info.id),
                info.api_major,
                info.api_minor,
                if info.kind == 1 { "source" } else { "other" }
            );
            check(call(FF_INITIALISE_V2, uint(0), std::ptr::null_mut()).u == FF_SUCCESS, "initialise")?;
            let num = call(FF_GET_NUM_PARAMETERS, uint(0), std::ptr::null_mut()).u;
            for i in 0..num {
                let name = CStr::from_ptr(call(FF_GET_PARAMETER_NAME, uint(i), std::ptr::null_mut()).p as *const c_char);
                let ty = call(FF_GET_PARAMETER_TYPE, uint(i), std::ptr::null_mut()).u;
                let def = f32::from_bits(call(FF_GET_PARAMETER_DEFAULT, uint(i), std::ptr::null_mut()).u);
                println!("  param {i:2}: {:<12} type {ty:3} default {def:.3}", name.to_string_lossy());
            }

            // ── Host framebuffer the plugin renders into ──
            let tex = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0, glow::RGBA8 as i32, WIDTH as i32, HEIGHT as i32, 0,
                glow::RGBA, glow::UNSIGNED_BYTE, glow::PixelUnpackData::Slice(None),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
            let fbo = gl.create_framebuffer()?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(tex), 0);
            check(gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE, "host fbo complete")?;
            gl.viewport(0, 0, WIDTH as i32, HEIGHT as i32);
            validate_state(&gl, fbo, "before instantiate")?;

            // ── Instance ──
            let vp = Viewport { x: 0, y: 0, w: WIDTH, h: HEIGHT };
            let inst = call(FF_INSTANTIATE_GL, ptr(&vp), std::ptr::null_mut()).p;
            check(!inst.is_null() && inst as usize != FF_FAIL as usize, "instantiate")?;
            validate_state(&gl, fbo, "after instantiate")?;
            check(call(FF_CONNECT, uint(0), inst).u == FF_SUCCESS, "connect")?;

            let mut failures = Vec::new();
            for (shape_idx, shape) in SHAPES.iter().enumerate() {
                let set = SetParam { index: 0, value: uint((shape_idx as f32).to_bits()) };
                check(call(FF_SET_PARAMETER, ptr(&set), inst).u == FF_SUCCESS, "set shape")?;
                let shown = CStr::from_ptr(call(FF_GET_PARAMETER_DISPLAY, uint(0), inst).p as *const c_char);

                for frame in 0..FRAMES_PER_SHAPE {
                    let t = (shape_idx as u32 * FRAMES_PER_SHAPE + frame) as f64 / 60.0;
                    call(FF_SET_TIME, ptr(&t), inst);
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
                    gl.clear_color(0.0, 0.0, 0.0, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                    let pgl = ProcessGl { num_inputs: 0, inputs: std::ptr::null_mut(), host_fbo: fbo.0.get() };
                    check(call(FF_PROCESS_OPENGL, ptr(&pgl), inst).u == FF_SUCCESS, "process")?;
                    validate_state(&gl, fbo, &format!("after process ({shape} frame {frame})"))?;
                }

                let pixels = read_pixels(&gl);
                let lit = pixels.chunks(4).filter(|p| p[0].max(p[1]).max(p[2]) > 40).count();
                let frac = lit as f32 / (WIDTH * HEIGHT) as f32;
                let path = out_dir.join(format!("{shape}.png"));
                image::RgbaImage::from_raw(WIDTH, HEIGHT, pixels)
                    .ok_or("image buffer size")?
                    .save(&path)
                    .map_err(|e| e.to_string())?;
                println!("  {:<10} (shown as {:?}): {:5.2}% lit -> {}", shape, shown.to_string_lossy(), frac * 100.0, path.display());
                // A visible beam lights some of the frame but not most of it.
                if !(0.002..0.6).contains(&frac) {
                    failures.push(format!("{shape}: {:.2}% of pixels lit", frac * 100.0));
                }
            }

            check(call(FF_DEINSTANTIATE_GL, uint(0), inst).u == FF_SUCCESS, "deinstantiate")?;
            validate_state(&gl, fbo, "after deinstantiate")?;
            check(call(FF_DEINITIALISE, uint(0), std::ptr::null_mut()).u == FF_SUCCESS, "deinitialise")?;
            check(gl.get_error() == glow::NO_ERROR, "no GL errors")?;

            if failures.is_empty() {
                println!("All checks passed.");
                Ok(())
            } else {
                Err(failures.join("; "))
            }
        }
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
        expect("BLEND_EQUATION_RGB", int(glow::BLEND_EQUATION_RGB), glow::FUNC_ADD as i32);
        expect("BLEND_EQUATION_ALPHA", int(glow::BLEND_EQUATION_ALPHA), glow::FUNC_ADD as i32);
        expect("BLEND_SRC_RGB", int(glow::BLEND_SRC_RGB), glow::ONE as i32);
        expect("BLEND_SRC_ALPHA", int(glow::BLEND_SRC_ALPHA), glow::ONE as i32);
        expect("BLEND_DST_RGB", int(glow::BLEND_DST_RGB), glow::ZERO as i32);
        expect("BLEND_DST_ALPHA", int(glow::BLEND_DST_ALPHA), glow::ZERO as i32);
        expect("FRAMEBUFFER_BINDING (host fbo)", int(glow::FRAMEBUFFER_BINDING), fbo.0.get() as i32);
        let err = gl.get_error();
        expect("glGetError", err as i32, glow::NO_ERROR as i32);
        if bad.is_empty() { Ok(()) } else { Err(format!("GL state {when}: {}", bad.join(", "))) }
    }

    unsafe fn read_pixels(gl: &glow::Context) -> Vec<u8> {
        let mut pixels = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
        gl.read_pixels(
            0, 0, WIDTH as i32, HEIGHT as i32, glow::RGBA, glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(&mut pixels)),
        );
        // GL rows start at the bottom; PNG rows start at the top.
        let row = (WIDTH * 4) as usize;
        let mut flipped = Vec::with_capacity(pixels.len());
        for y in (0..HEIGHT as usize).rev() {
            flipped.extend_from_slice(&pixels[y * row..(y + 1) * row]);
        }
        flipped
    }

    /// Hidden window + OpenGL 4.1 core context (what Resolume gives plugins).
    unsafe fn create_gl_context() -> Result<glow::Context, String> {
        let instance = GetModuleHandleA(std::ptr::null());
        let class_name = c"RusciFfglHost".as_ptr().cast();
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
            0, class_name, class_name, WS_OVERLAPPEDWINDOW, 0, 0, WIDTH as i32, HEIGHT as i32,
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
