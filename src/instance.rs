//! One plugin instance (one clip in Resolume).

use std::ffi::CString;
use std::num::NonZeroU32;

use glow::HasContext;
use osci_visualizer::OsciRenderer;

use crate::ffi::FFGLViewportStruct;
use crate::params::ParamValues;
use crate::scene::Scene;

/// Frame rate assumed for animation if the host never sends a time.
const FALLBACK_FPS: f64 = 60.0;

struct GlState {
    gl: glow::Context,
    renderer: OsciRenderer,
}

pub struct Instance {
    pub params: ParamValues,
    /// Backing storage for the last `FF_GET_PARAMETER_DISPLAY` string.
    pub display: CString,
    viewport: FFGLViewportStruct,
    host_time: Option<f64>,
    frames: u64,
    scene: Scene,
    gl: Option<GlState>,
}

impl Instance {
    /// An instance with no GL resources (used by tests and before `init_gl`).
    pub fn new(viewport: FFGLViewportStruct) -> Self {
        Self {
            params: ParamValues::default(),
            display: CString::default(),
            viewport,
            host_time: None,
            frames: 0,
            scene: Scene::new(),
            gl: None,
        }
    }

    /// Create GL resources in the host's current context.
    ///
    /// # Safety
    /// The host's GL context must be current on this thread.
    pub unsafe fn init_gl(&mut self) {
        let gl = crate::gl_loader::load_current_context();
        let host_fbo = current_framebuffer(&gl);
        let renderer = OsciRenderer::new(&gl);
        reset_to_host_defaults(&gl, host_fbo);
        self.gl = Some(GlState { gl, renderer });
    }

    pub fn set_viewport(&mut self, viewport: FFGLViewportStruct) {
        self.viewport = viewport;
    }

    pub fn set_time(&mut self, seconds: f64) {
        self.host_time = Some(seconds);
    }

    fn time(&self) -> f64 {
        self.host_time.unwrap_or(self.frames as f64 / FALLBACK_FPS)
    }

    /// Draw one frame into `host_fbo`.
    ///
    /// # Safety
    /// The host's GL context must be current on this thread.
    pub unsafe fn process(&mut self, host_fbo: u32) -> bool {
        let time = self.time();
        self.frames += 1;
        let Some(state) = self.gl.as_mut() else {
            return false;
        };

        let settings = self.params.visualiser_settings();
        let (xs, ys) = self.scene.update(&self.params, time);

        // OsciRenderer renders its final pass into whatever FBO is bound.
        let host_fbo = NonZeroU32::new(host_fbo).map(glow::NativeFramebuffer);
        state.gl.bind_framebuffer(glow::FRAMEBUFFER, host_fbo);
        // Clear the whole frame to transparent, then draw the scope in a
        // centred square so shapes keep their proportions at any aspect ratio.
        let vp = self.viewport;
        state.gl.viewport(vp.x as i32, vp.y as i32, vp.width as i32, vp.height as i32);
        state.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl.clear(glow::COLOR_BUFFER_BIT);
        state.renderer.render(&state.gl, xs, ys, &settings, square_viewport(vp));
        reset_to_host_defaults(&state.gl, host_fbo);
        true
    }

    /// Free GL resources.
    ///
    /// # Safety
    /// The host's GL context must be current on this thread.
    pub unsafe fn deinit_gl(&mut self) {
        if let Some(state) = self.gl.take() {
            let host_fbo = current_framebuffer(&state.gl);
            state.renderer.destroy(&state.gl);
            reset_to_host_defaults(&state.gl, host_fbo);
        }
    }
}

/// Largest square centred in `vp`, as [x, y, width, height].
pub fn square_viewport(vp: FFGLViewportStruct) -> [i32; 4] {
    let side = vp.width.min(vp.height);
    [
        (vp.x + (vp.width - side) / 2) as i32,
        (vp.y + (vp.height - side) / 2) as i32,
        side as i32,
        side as i32,
    ]
}

unsafe fn current_framebuffer(gl: &glow::Context) -> Option<glow::Framebuffer> {
    NonZeroU32::new(gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING) as u32).map(glow::NativeFramebuffer)
}

/// FFGL hosts expect plugins to hand back a context with no program,
/// buffers or textures bound (see `ValidateContextState` in the SDK), and
/// with the host's framebuffer still bound.
/// The visualiser restores most of this itself; this catches the rest.
unsafe fn reset_to_host_defaults(gl: &glow::Context, host_fbo: Option<glow::Framebuffer>) {
    gl.bind_framebuffer(glow::FRAMEBUFFER, host_fbo);
    gl.use_program(None);
    gl.bind_vertex_array(None);
    gl.bind_buffer(glow::ARRAY_BUFFER, None);
    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
    for unit in 0..4 {
        gl.active_texture(glow::TEXTURE0 + unit);
        gl.bind_texture(glow::TEXTURE_2D, None);
    }
    gl.active_texture(glow::TEXTURE0);
    gl.disable(glow::BLEND);
    gl.blend_func(glow::ONE, glow::ZERO);
    gl.blend_equation(glow::FUNC_ADD);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vp(x: u32, y: u32, width: u32, height: u32) -> FFGLViewportStruct {
        FFGLViewportStruct { x, y, width, height }
    }

    #[test]
    fn square_viewport_is_centred() {
        assert_eq!(square_viewport(vp(0, 0, 1920, 1080)), [420, 0, 1080, 1080]);
        assert_eq!(square_viewport(vp(0, 0, 1080, 1920)), [0, 420, 1080, 1080]);
        assert_eq!(square_viewport(vp(10, 20, 500, 500)), [10, 20, 500, 500]);
    }
}
