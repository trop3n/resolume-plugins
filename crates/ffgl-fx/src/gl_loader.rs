//! Loads OpenGL functions from the host's current context.

use std::ffi::{c_void, CStr};

/// Build a glow context for the GL context the host has made current.
///
/// # Safety
/// A GL context must be current on this thread.
pub unsafe fn load_current_context() -> glow::Context {
    glow::Context::from_loader_function_cstr(get_proc_address)
}

#[cfg(windows)]
fn get_proc_address(name: &CStr) -> *const c_void {
    use windows_sys::Win32::Graphics::OpenGL::wglGetProcAddress;
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

    unsafe {
        // wglGetProcAddress only knows extension/post-1.1 functions and may
        // return small sentinel values (1, 2, 3, -1) instead of null on failure.
        let p = wglGetProcAddress(name.as_ptr().cast()).map_or(0, |f| f as usize);
        if !matches!(p, 0 | 1 | 2 | 3 | usize::MAX) {
            return p as *const c_void;
        }
        // GL 1.1 entry points come straight from opengl32.dll.
        let module = GetModuleHandleA(c"opengl32.dll".as_ptr().cast());
        if module.is_null() {
            return std::ptr::null();
        }
        GetProcAddress(module, name.as_ptr().cast()).map_or(std::ptr::null(), |f| f as *const c_void)
    }
}

#[cfg(target_os = "macos")]
fn get_proc_address(name: &CStr) -> *const c_void {
    use std::ffi::{c_char, c_int};
    use std::sync::OnceLock;

    extern "C" {
        fn dlopen(path: *const c_char, mode: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }
    // Values from macOS <dlfcn.h>.
    const RTLD_LAZY: c_int = 0x1;
    const RTLD_LOCAL: c_int = 0x4;

    // Every GL entry point macOS has (up to 4.1 core) is exported by the
    // OpenGL framework. Resolume has already loaded it, so this just finds it.
    static OPENGL: OnceLock<usize> = OnceLock::new();
    let handle = *OPENGL.get_or_init(|| unsafe {
        dlopen(c"/System/Library/Frameworks/OpenGL.framework/OpenGL".as_ptr(), RTLD_LAZY | RTLD_LOCAL) as usize
    });
    if handle == 0 {
        return std::ptr::null();
    }
    unsafe { dlsym(handle as *mut c_void, name.as_ptr()) as *const c_void }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn get_proc_address(_name: &CStr) -> *const c_void {
    // Resolume runs only on Windows and macOS.
    std::ptr::null()
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn finds_gl_functions() {
        // No context is needed to look functions up on macOS.
        for name in [c"glClear", c"glBindVertexArray", c"glTexImage2D", c"glBlendFuncSeparate"] {
            assert!(!get_proc_address(name).is_null(), "{name:?} not found");
        }
        assert!(get_proc_address(c"glNotARealFunction").is_null());
    }
}
