//! One plugin instance (one effect slot in Resolume) and its GL resources.
//!
//! Every pass draws a full-screen triangle. Passes before the last render
//! into a pair of RGBA16F textures that ping-pong, so each pass (and each
//! iteration of a repeated pass) reads what the previous one wrote through
//! `passIn`. The last pass renders into the host's framebuffer.

use std::ffi::CString;
use std::num::NonZeroU32;

use glow::HasContext;

use crate::ffi::{FFGLTextureStruct, FFGLViewportStruct};
use crate::params::Kind;
use crate::EffectDef;

/// Frame rate assumed for animation if the host never sends a time.
const FALLBACK_FPS: f64 = 60.0;

/// Uniforms every pass gets, in the order of `Program::builtins`.
pub(crate) const BUILTINS: [&str; 8] =
    ["inputImage", "passIn", "inputMaxUV", "RENDERSIZE", "TIME", "FRAMEINDEX", "PASSINDEX", "ITERATION"];

const VERTEX_SHADER: &str = "#version 410 core
out vec2 uv;
void main() {
    // Full-screen triangle: (0,0), (2,0), (0,2) in uv space.
    vec2 p = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2);
    uv = p;
    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}
";

const FRAGMENT_PRELUDE: &str = "#version 410 core
in vec2 uv;
out vec4 fragColor;

uniform sampler2D inputImage;   // the clip this effect is applied to
uniform sampler2D passIn;       // what the previous pass (or iteration) wrote
uniform vec2 inputMaxUV;        // inputImage may be padded; this is the used part
uniform vec2 RENDERSIZE;        // output size in pixels
uniform float TIME;             // seconds
uniform int FRAMEINDEX;
uniform int PASSINDEX;
uniform int ITERATION;          // 0.. within a repeated pass

vec4 IMG_INPUT(vec2 p) { return texture(inputImage, p * inputMaxUV); }
vec4 IMG_PASS(vec2 p) { return texture(passIn, p); }
vec4 FETCH_INPUT(ivec2 p) { return texelFetch(inputImage, clamp(p, ivec2(0), ivec2(RENDERSIZE) - 1), 0); }
vec4 FETCH_PASS(ivec2 p) { return texelFetch(passIn, clamp(p, ivec2(0), ivec2(RENDERSIZE) - 1), 0); }
// Pixel coordinate that is the same in every pass (gl_FragCoord is offset
// by the host viewport in the last pass).
ivec2 pixelCoord() { return ivec2(uv * RENDERSIZE); }
// Multiply sizes given in pixels at 1080p by this to keep looks resolution-independent.
float sizeScale() { return RENDERSIZE.y / 1080.0; }

// ── Helpers ──
float hash12(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
vec2 hash22(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * vec3(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}
float valueNoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash12(i), hash12(i + vec2(1, 0)), f.x),
               mix(hash12(i + vec2(0, 1)), hash12(i + vec2(1, 1)), f.x), f.y);
}
// Four octaves of value noise, 0..1.
float fbm(vec2 p) {
    float sum = 0.0;
    float amp = 0.5;
    for (int i = 0; i < 4; i++) {
        sum += amp * valueNoise(p);
        p = p * 2.03 + 17.1;
        amp *= 0.5;
    }
    return sum / 0.9375;
}
float luma(vec3 c) { return dot(c, vec3(0.2126, 0.7152, 0.0722)); }
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    float d = q.x - min(q.w, q.y);
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + 1e-10)), d / (q.x + 1e-10), q.x);
}
vec3 hsv2rgb(vec3 c) {
    vec3 p = abs(fract(c.xxx + vec3(1.0, 2.0 / 3.0, 1.0 / 3.0)) * 6.0 - 3.0);
    return c.z * mix(vec3(1.0), clamp(p - 1.0, 0.0, 1.0), c.y);
}
// Ordered-dither thresholds in [0, 1) for integer cell coordinates.
float bayer2(vec2 a) { a = floor(a); return fract(a.x / 2.0 + a.y * a.y * 0.75); }
float bayer4(vec2 a) { return bayer2(0.5 * a) * 0.25 + bayer2(a); }
float bayer8(vec2 a) { return bayer4(0.5 * a) * 0.25 + bayer2(a); }
";

/// Full fragment shader source for one pass: prelude, param uniforms, body.
pub fn fragment_source(def: &EffectDef, pass: usize) -> String {
    let mut src = String::from(FRAGMENT_PRELUDE);
    for p in def.params {
        src.push_str(&format!("uniform {} {};\n", p.glsl_type(), p.ident));
    }
    for name in def.buffers() {
        src.push_str(&format!("uniform sampler2D {name};   // persistent buffer\n"));
    }
    src.push_str("#line 1\n");
    src.push_str(def.passes[pass].shader);
    src
}

/// Texture unit of persistent buffer `b` (0 is the input, 1 is `passIn`).
const fn buffer_unit(b: usize) -> u32 {
    2 + b as u32
}

struct Program {
    program: glow::Program,
    builtins: Vec<Option<glow::UniformLocation>>,
    params: Vec<Option<glow::UniformLocation>>,
    buffers: Vec<Option<glow::UniformLocation>>,
}

struct Target {
    texture: glow::Texture,
    fbo: glow::Framebuffer,
}

struct GlState {
    gl: glow::Context,
    vao: glow::VertexArray,
    programs: Vec<Program>,
    /// Ping-pong pair (0, 1), then a front/back pair per persistent buffer.
    targets: Vec<Target>,
    target_size: (u32, u32),
    /// Which of its two targets holds each buffer's latest contents.
    buffer_front: Vec<usize>,
}

impl GlState {
    fn buffer_target(&self, b: usize, front: bool) -> &Target {
        let which = if front { self.buffer_front[b] } else { 1 - self.buffer_front[b] };
        &self.targets[2 + 2 * b + which]
    }
}

pub struct Instance {
    def: &'static EffectDef,
    /// Real values, one per host param.
    values: Vec<f32>,
    /// Backing storage for the last `FF_GET_PARAMETER_DISPLAY` string.
    pub display: CString,
    viewport: FFGLViewportStruct,
    host_time: Option<f64>,
    frames: u64,
    gl: Option<GlState>,
}

impl Instance {
    /// An instance with no GL resources (used by tests and before `init_gl`).
    pub fn new(def: &'static EffectDef, viewport: FFGLViewportStruct) -> Self {
        Self {
            def,
            values: def.host_params().iter().map(|p| p.default).collect(),
            display: CString::default(),
            viewport,
            host_time: None,
            frames: 0,
            gl: None,
        }
    }

    /// Real value of a host param.
    pub fn value(&self, index: usize) -> Option<f32> {
        self.values.get(index).copied()
    }

    pub fn get_host(&self, index: usize) -> Option<f32> {
        Some(self.def.host_params().get(index)?.to_host(self.values[index]))
    }

    /// Returns false for an unknown index or a non-finite value.
    pub fn set_host(&mut self, index: usize, host: f32) -> bool {
        match self.def.host_params().get(index) {
            Some(p) if host.is_finite() => {
                self.values[index] = p.from_host(host);
                true
            }
            _ => false,
        }
    }

    pub fn display_text(&self, index: usize) -> Option<String> {
        Some(self.def.host_params().get(index)?.display(self.values[index]))
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

    /// How many times pass `index` runs this frame (the last pass runs once).
    pub fn iterations(&self, index: usize) -> u32 {
        let def = self.def;
        if index + 1 == def.passes.len() {
            return 1;
        }
        let pass = &def.passes[index];
        let Some(ident) = pass.repeat else {
            return 1;
        };
        let param = def.params.iter().position(|p| p.ident == ident).expect("validated");
        let steps = self.values[def.first_host(param)].max(0.0) as u32;
        steps.div_ceil(pass.steps_per_run.max(1))
    }

    /// Compile the shaders in the host's current context.
    ///
    /// # Safety
    /// The host's GL context must be current on this thread.
    pub unsafe fn init_gl(&mut self) -> Result<(), String> {
        let gl = crate::gl_loader::load_current_context();
        let host_fbo = current_framebuffer(&gl);
        let result = build_programs(&gl, self.def);
        let vp = self.viewport;
        reset_to_host_defaults(&gl, host_fbo, vp, 0);
        let programs = result?;
        let vao = gl.create_vertex_array()?;
        let buffer_front = vec![0; self.def.buffers().len()];
        self.gl = Some(GlState { gl, vao, programs, targets: Vec::new(), target_size: (0, 0), buffer_front });
        Ok(())
    }

    /// Render one frame of the effect into `host_fbo`.
    ///
    /// # Safety
    /// The host's GL context must be current on this thread, and `input`'s
    /// handle a texture in it.
    pub unsafe fn process(&mut self, input: Option<&FFGLTextureStruct>, host_fbo: u32) -> bool {
        let time = self.time();
        let frame = self.frames;
        self.frames += 1;
        let iterations: Vec<u32> = (0..self.def.passes.len()).map(|i| self.iterations(i)).collect();
        let (def, vp) = (self.def, self.viewport);
        let (Some(input), Some(state)) = (input, self.gl.as_mut()) else {
            return false;
        };
        let host_fbo = NonZeroU32::new(host_fbo).map(glow::NativeFramebuffer);
        let (w, h) = (vp.width.max(1), vp.height.max(1));
        let buffers = def.buffers();
        let units = buffer_unit(buffers.len());
        if def.passes.len() > 1 && !state.ensure_targets(w, h) {
            crate::log("could not create intermediate render targets");
            reset_to_host_defaults(&state.gl, host_fbo, vp, units);
            return false;
        }
        // Which buffer, if any, each pass stores into.
        let stores: Vec<Option<usize>> =
            def.passes.iter().map(|p| p.store.map(|n| buffers.iter().position(|b| *b == n).expect("listed"))).collect();

        let ratio = |used: u32, full: u32| if full == 0 { 1.0 } else { used as f32 / full as f32 };
        let max_uv = [ratio(input.Width, input.HardwareWidth), ratio(input.Height, input.HardwareHeight)];
        let input_tex = NonZeroU32::new(input.Handle).map(glow::NativeTexture);

        state.gl.bind_vertex_array(Some(state.vao));
        state.gl.active_texture(glow::TEXTURE0);
        state.gl.bind_texture(glow::TEXTURE_2D, input_tex);

        let last = def.passes.len() - 1;
        let (mut read, mut write) = (1, 0);
        for pi in 0..state.programs.len() {
            let gl = &state.gl;
            let prog = &state.programs[pi];
            gl.use_program(Some(prog.program));
            for (bi, loc) in prog.buffers.iter().enumerate() {
                gl.uniform_1_i32(loc.as_ref(), buffer_unit(bi) as i32);
            }
            let b = &prog.builtins;
            gl.uniform_1_i32(b[0].as_ref(), 0);
            gl.uniform_1_i32(b[1].as_ref(), 1);
            gl.uniform_2_f32(b[2].as_ref(), max_uv[0], max_uv[1]);
            gl.uniform_2_f32(b[3].as_ref(), w as f32, h as f32);
            gl.uniform_1_f32(b[4].as_ref(), time as f32);
            gl.uniform_1_i32(b[5].as_ref(), frame as i32);
            gl.uniform_1_i32(b[6].as_ref(), pi as i32);
            for (i, p) in def.params.iter().enumerate() {
                let Some(loc) = prog.params[i].as_ref() else { continue };
                let v = &self.values[def.first_host(i)..];
                match p.kind {
                    Kind::Float { .. } => gl.uniform_1_f32(Some(loc), v[0]),
                    Kind::Int { .. } | Kind::Option { .. } | Kind::Bool { .. } => gl.uniform_1_i32(Some(loc), v[0] as i32),
                    Kind::Colour { .. } => gl.uniform_3_f32(Some(loc), v[0], v[1], v[2]),
                }
            }

            for it in 0..iterations[pi] {
                let gl = &state.gl;
                gl.uniform_1_i32(state.programs[pi].builtins[7].as_ref(), it as i32);
                if pi == last {
                    gl.bind_framebuffer(glow::FRAMEBUFFER, host_fbo);
                    gl.viewport(vp.x as i32, vp.y as i32, vp.width as i32, vp.height as i32);
                } else {
                    let target = match stores[pi] {
                        Some(bi) => state.buffer_target(bi, false),
                        None => &state.targets[write],
                    };
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target.fbo));
                    gl.viewport(0, 0, w as i32, h as i32);
                }
                gl.active_texture(glow::TEXTURE1);
                gl.bind_texture(glow::TEXTURE_2D, state.targets.get(read).map(|t| t.texture));
                for bi in 0..buffers.len() {
                    gl.active_texture(glow::TEXTURE0 + buffer_unit(bi));
                    gl.bind_texture(glow::TEXTURE_2D, Some(state.buffer_target(bi, true).texture));
                }
                gl.active_texture(glow::TEXTURE0);
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
                match stores[pi] {
                    // The back target now holds the newest contents.
                    Some(bi) => state.buffer_front[bi] ^= 1,
                    None if pi != last => std::mem::swap(&mut read, &mut write),
                    None => {}
                }
            }
        }

        reset_to_host_defaults(&state.gl, host_fbo, vp, units);
        true
    }

    /// Free GL resources.
    ///
    /// # Safety
    /// The host's GL context must be current on this thread.
    pub unsafe fn deinit_gl(&mut self) {
        if let Some(mut state) = self.gl.take() {
            let host_fbo = current_framebuffer(&state.gl);
            state.delete_targets();
            for p in &state.programs {
                state.gl.delete_program(p.program);
            }
            state.gl.delete_vertex_array(state.vao);
            let units = buffer_unit(state.buffer_front.len());
            reset_to_host_defaults(&state.gl, host_fbo, self.viewport, units);
        }
    }
}

impl GlState {
    /// Make sure the ping-pong pair and every buffer's pair exist at
    /// `w`x`h`. Recreated targets start transparent black. Leaves FBO
    /// bindings changed.
    unsafe fn ensure_targets(&mut self, w: u32, h: u32) -> bool {
        let count = 2 + 2 * self.buffer_front.len();
        if self.targets.len() == count && self.target_size == (w, h) {
            return true;
        }
        self.delete_targets();
        self.buffer_front.iter_mut().for_each(|f| *f = 0);
        let gl = &self.gl;
        for _ in 0..count {
            let (Ok(texture), Ok(fbo)) = (gl.create_texture(), gl.create_framebuffer()) else {
                return false;
            };
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0, glow::RGBA16F as i32, w as i32, h as i32, 0,
                glow::RGBA, glow::HALF_FLOAT, glow::PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(texture), 0);
            let complete = gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE;
            if complete {
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
            }
            self.targets.push(Target { texture, fbo });
            if !complete {
                self.delete_targets();
                return false;
            }
        }
        self.target_size = (w, h);
        true
    }

    unsafe fn delete_targets(&mut self) {
        for t in self.targets.drain(..) {
            self.gl.delete_framebuffer(t.fbo);
            self.gl.delete_texture(t.texture);
        }
        self.target_size = (0, 0);
    }
}

unsafe fn compile(gl: &glow::Context, kind: u32, src: &str, what: &str) -> Result<glow::Shader, String> {
    let shader = gl.create_shader(kind)?;
    gl.shader_source(shader, src);
    gl.compile_shader(shader);
    if gl.get_shader_compile_status(shader) {
        Ok(shader)
    } else {
        let log = gl.get_shader_info_log(shader);
        gl.delete_shader(shader);
        Err(format!("{what} failed to compile:\n{log}"))
    }
}

unsafe fn build_programs(gl: &glow::Context, def: &EffectDef) -> Result<Vec<Program>, String> {
    let vs = compile(gl, glow::VERTEX_SHADER, VERTEX_SHADER, "vertex shader")?;
    let mut programs = Vec::new();
    let mut result = Ok(());
    for i in 0..def.passes.len() {
        let fs = match compile(gl, glow::FRAGMENT_SHADER, &fragment_source(def, i), &format!("pass {i}")) {
            Ok(fs) => fs,
            Err(e) => {
                result = Err(e);
                break;
            }
        };
        let program = gl.create_program()?;
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        gl.detach_shader(program, vs);
        gl.detach_shader(program, fs);
        gl.delete_shader(fs);
        if !gl.get_program_link_status(program) {
            result = Err(format!("pass {i} failed to link:\n{}", gl.get_program_info_log(program)));
            gl.delete_program(program);
            break;
        }
        programs.push(Program {
            program,
            builtins: BUILTINS.iter().map(|n| gl.get_uniform_location(program, n)).collect(),
            params: def.params.iter().map(|p| gl.get_uniform_location(program, p.ident)).collect(),
            buffers: def.buffers().iter().map(|n| gl.get_uniform_location(program, n)).collect(),
        });
    }
    gl.delete_shader(vs);
    match result {
        Ok(()) => Ok(programs),
        Err(e) => {
            for p in programs {
                gl.delete_program(p.program);
            }
            Err(e)
        }
    }
}

unsafe fn current_framebuffer(gl: &glow::Context) -> Option<glow::Framebuffer> {
    NonZeroU32::new(gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING) as u32).map(glow::NativeFramebuffer)
}

/// FFGL hosts expect plugins to hand back a context with no program,
/// buffers or textures bound (see `ValidateContextState` in the SDK), and
/// with the host's framebuffer and viewport still set.
/// `units` is how many texture units the effect used (at least 4 are reset).
unsafe fn reset_to_host_defaults(gl: &glow::Context, host_fbo: Option<glow::Framebuffer>, vp: FFGLViewportStruct, units: u32) {
    gl.bind_framebuffer(glow::FRAMEBUFFER, host_fbo);
    gl.viewport(vp.x as i32, vp.y as i32, vp.width as i32, vp.height as i32);
    gl.use_program(None);
    gl.bind_vertex_array(None);
    gl.bind_buffer(glow::ARRAY_BUFFER, None);
    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
    for unit in 0..units.max(4) {
        gl.active_texture(glow::TEXTURE0 + unit);
        gl.bind_texture(glow::TEXTURE_2D, None);
    }
    gl.active_texture(glow::TEXTURE0);
}
