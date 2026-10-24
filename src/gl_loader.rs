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

#[cfg(not(windows))]
fn get_proc_address(_name: &CStr) -> *const c_void {
    // macOS/Linux hosts are not supported yet.
    std::ptr::null()
}
