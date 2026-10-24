//! FFGL 2.x ABI, transcribed from Resolume's SDK header
//! (`source/lib/ffgl/FFGL.h` in https://github.com/resolume/ffgl).
//!
//! Only what this plugin uses is included. Layouts are checked by the
//! `layout` tests at the bottom of this file.

#![allow(non_snake_case, dead_code)]

use std::ffi::{c_char, c_void};

// ── Function codes ──────────────────────────────────────────────────
pub const FF_GET_INFO: u32 = 0;
pub const FF_DEINITIALISE: u32 = 2;
pub const FF_GET_NUM_PARAMETERS: u32 = 4;
pub const FF_GET_PARAMETER_NAME: u32 = 5;
pub const FF_GET_PARAMETER_DEFAULT: u32 = 6;
pub const FF_GET_PARAMETER_DISPLAY: u32 = 7;
pub const FF_SET_PARAMETER: u32 = 8;
pub const FF_GET_PARAMETER: u32 = 9;
pub const FF_GET_PLUGIN_CAPS: u32 = 10;
pub const FF_GET_EXTENDED_INFO: u32 = 13;
pub const FF_GET_PARAMETER_TYPE: u32 = 15;
pub const FF_GET_INPUT_STATUS: u32 = 16;
pub const FF_PROCESS_OPENGL: u32 = 17;
pub const FF_INSTANTIATE_GL: u32 = 18;
pub const FF_DEINSTANTIATE_GL: u32 = 19;
pub const FF_SET_TIME: u32 = 20;
pub const FF_CONNECT: u32 = 21;
pub const FF_DISCONNECT: u32 = 22;
pub const FF_RESIZE: u32 = 23;
pub const FF_GET_NUM_PARAMETER_ELEMENTS: u32 = 31;
pub const FF_GET_PARAMETER_USAGE: u32 = 32;
pub const FF_GET_PLUGIN_SHORT_NAME: u32 = 33;
pub const FF_INITIALISE_V2: u32 = 34;
pub const FF_GET_PARAMETER_ELEMENT_NAME: u32 = 35;
pub const FF_GET_PARAMETER_ELEMENT_VALUE: u32 = 36;
pub const FF_SET_PARAMETER_ELEMENT_VALUE: u32 = 37;
pub const FF_SET_BEATINFO: u32 = 38;
pub const FF_SET_HOSTINFO: u32 = 39;
pub const FF_SET_SAMPLERATE: u32 = 40;
pub const FF_GET_RANGE: u32 = 41;
pub const FF_GET_THUMBNAIL: u32 = 42;
pub const FF_GET_NUM_FILE_PARAMETER_EXTENSIONS: u32 = 43;
pub const FF_GET_FILE_PARAMETER_EXTENSION: u32 = 44;
/// (sic) — the SDK spells it "PRAMETER".
pub const FF_GET_PRAMETER_VISIBILITY: u32 = 45;
pub const FF_GET_PARAMETER_EVENTS: u32 = 46;
pub const FF_GET_NUM_ELEMENT_SEPARATORS: u32 = 47;
pub const FF_GET_SEPARATOR_ELEMENT_INDEX: u32 = 48;
pub const FF_ENABLE_PLUGIN_CAP: u32 = 49;
pub const FF_GET_PARAM_GROUP: u32 = 50;
pub const FF_GET_PARAM_DISPLAY_NAME: u32 = 51;

// ── Results ─────────────────────────────────────────────────────────
pub const FF_SUCCESS: u32 = 0;
pub const FF_FAIL: u32 = 0xFFFF_FFFF;
pub const FF_TRUE: u32 = 1;
pub const FF_FALSE: u32 = 0;

// ── Plugin types ────────────────────────────────────────────────────
pub const FF_EFFECT: u32 = 0;
pub const FF_SOURCE: u32 = 1;

// ── Capabilities ────────────────────────────────────────────────────
pub const FF_CAP_SET_TIME: u32 = 5;
pub const FF_CAP_MINIMUM_INPUT_FRAMES: u32 = 10;
pub const FF_CAP_MAXIMUM_INPUT_FRAMES: u32 = 11;
pub const FF_CAP_TOP_LEFT_TEXTURE_ORIENTATION: u32 = 16;

// ── Parameter types / usage ─────────────────────────────────────────
pub const FF_TYPE_BOOLEAN: u32 = 0;
pub const FF_TYPE_RED: u32 = 2;
pub const FF_TYPE_GREEN: u32 = 3;
pub const FF_TYPE_BLUE: u32 = 4;
pub const FF_TYPE_STANDARD: u32 = 10;
pub const FF_TYPE_OPTION: u32 = 11;
pub const FF_INPUT_NOTINUSE: u32 = 0;
pub const FF_USAGE_STANDARD: u32 = 0;

// ── Types ───────────────────────────────────────────────────────────

/// 32-bit value or pointer, depending on the function code.
#[repr(C)]
#[derive(Clone, Copy)]
pub union FFMixed {
    pub UIntValue: u32,
    pub PointerValue: *mut c_void,
}

impl FFMixed {
    pub fn uint(v: u32) -> Self {
        // Zero the full 8 bytes first so the upper half is never garbage.
        let mut m = FFMixed { PointerValue: std::ptr::null_mut() };
        m.UIntValue = v;
        m
    }
    pub fn ptr<T>(p: *const T) -> Self {
        FFMixed { PointerValue: p as *mut c_void }
    }
    /// For structs the plugin writes results into.
    pub fn ptr_mut<T>(p: *mut T) -> Self {
        FFMixed { PointerValue: p.cast() }
    }
    pub fn float(f: f32) -> Self {
        Self::uint(f.to_bits())
    }
    pub fn fail() -> Self {
        Self::uint(FF_FAIL)
    }
    pub fn as_u32(self) -> u32 {
        unsafe { self.UIntValue }
    }
    pub fn as_ptr(self) -> *mut c_void {
        unsafe { self.PointerValue }
    }
}

#[repr(C)]
pub struct PluginInfoStruct {
    pub APIMajorVersion: u32,
    pub APIMinorVersion: u32,
    /// 4 chars, not null-terminated.
    pub PluginUniqueID: [u8; 4],
    /// 16 chars, not null-terminated.
    pub PluginName: [u8; 16],
    pub PluginType: u32,
}

#[repr(C)]
pub struct PluginExtendedInfoStruct {
    pub PluginMajorVersion: u32,
    pub PluginMinorVersion: u32,
    pub Description: *const c_char,
    pub About: *const c_char,
    pub FreeFrameExtendedDataSize: u32,
    pub FreeFrameExtendedDataBlock: *mut c_void,
}

#[repr(C)]
pub struct SetParameterStruct {
    pub ParameterNumber: u32,
    pub NewParameterValue: FFMixed,
}

#[repr(C)]
pub struct SetBeatinfoStruct {
    pub bpm: f32,
    pub barPhase: f32,
}

#[repr(C)]
pub struct RangeStruct {
    pub min: f32,
    pub max: f32,
}

#[repr(C)]
pub struct GetRangeStruct {
    pub parameterNumber: u32,
    pub range: RangeStruct,
}

#[repr(C)]
pub struct StringBufferStruct {
    pub address: *mut c_char,
    /// Max chars to write, excluding the nul. Plugins don't write the nul.
    pub maxToWrite: u32,
}

#[repr(C)]
pub struct GetStringStruct {
    pub parameterNumber: u32,
    pub stringBuffer: StringBufferStruct,
}

#[repr(C)]
pub struct GetThumbnailStruct {
    pub width: u32,
    pub height: u32,
    pub rgbaPixelBuffer: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FFGLViewportStruct {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
pub struct FFGLTextureStruct {
    pub Width: u32,
    pub Height: u32,
    pub HardwareWidth: u32,
    pub HardwareHeight: u32,
    pub Handle: u32,
}

#[repr(C)]
pub struct ProcessOpenGLStruct {
    pub numInputTextures: u32,
    pub inputTextures: *mut *mut FFGLTextureStruct,
    /// FBO the host has bound for our output (0 = default framebuffer).
    pub HostFBO: u32,
}

#[repr(C)]
pub struct GetParameterElementNameStruct {
    pub ParameterNumber: u32,
    pub ElementNumber: u32,
}

#[repr(C)]
pub struct GetParameterElementValueStruct {
    pub ParameterNumber: u32,
    pub ElementNumber: u32,
}

#[repr(C)]
pub struct ParamEventStruct {
    pub ParameterNumber: u32,
    pub eventFlags: u64,
}

#[repr(C)]
pub struct GetParamEventsStruct {
    pub numEvents: u32,
    pub events: *mut ParamEventStruct,
}

/// `void (__stdcall *PFNLog)(char*)`
pub type PFNLog = Option<unsafe extern "system" fn(*mut c_char)>;

#[cfg(all(test, target_pointer_width = "64"))]
mod layout {
    use super::*;
    use std::mem::{align_of, size_of};

    // Sizes a 64-bit C compiler produces for the SDK's structs.
    #[test]
    fn struct_sizes_match_c() {
        assert_eq!(size_of::<FFMixed>(), 8);
        assert_eq!(size_of::<PluginInfoStruct>(), 32);
        assert_eq!(size_of::<PluginExtendedInfoStruct>(), 40);
        assert_eq!(size_of::<SetParameterStruct>(), 16);
        assert_eq!(size_of::<GetRangeStruct>(), 12);
        assert_eq!(size_of::<StringBufferStruct>(), 16);
        assert_eq!(size_of::<GetStringStruct>(), 24);
        assert_eq!(size_of::<GetThumbnailStruct>(), 16);
        assert_eq!(size_of::<FFGLViewportStruct>(), 16);
        assert_eq!(size_of::<FFGLTextureStruct>(), 20);
        assert_eq!(size_of::<ProcessOpenGLStruct>(), 24);
        assert_eq!(size_of::<ParamEventStruct>(), 16);
        assert_eq!(size_of::<GetParamEventsStruct>(), 16);
        assert_eq!(align_of::<SetParameterStruct>(), 8);
    }

    #[test]
    fn mixed_uint_zeroes_upper_bits() {
        let m = FFMixed::uint(7);
        assert_eq!(m.as_ptr() as usize, 7);
    }
}
