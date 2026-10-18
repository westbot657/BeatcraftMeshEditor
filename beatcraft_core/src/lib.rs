#![allow(non_snake_case, static_mut_refs)]

use std::mem::MaybeUninit;
use std::sync::LazyLock;
use jni::objects::JClass;
use jni::refs::Global;
use parking_lot::RwLock;
use crate::type_map::TypeMap;

pub mod classes;
mod type_map;
pub mod audio_system;

pub type GlbCls = Global<JClass<'static>>;
pub struct JavaTypes {
    pub i8: GlbCls,
    pub i16: GlbCls,
    pub i32: GlbCls,
    pub i64: GlbCls,
    pub u8: GlbCls,
    pub u16: GlbCls,
    pub u32: GlbCls,
    pub u64: GlbCls,
    pub f32: GlbCls,
    pub f64: GlbCls,
    pub bool: GlbCls,
    pub string: GlbCls,
    pub error: GlbCls,
    pub vec2: GlbCls,
    pub vec3: GlbCls,
    pub vec4: GlbCls,
    pub quat: GlbCls,
    pub mat4: GlbCls,
    pub u8buf: GlbCls,
    pub object: GlbCls,
}

pub static mut TYPES: MaybeUninit<JavaTypes> = MaybeUninit::uninit();

/// Safety:
/// Passing raw pointers to java may be unsafe (in the modding environment at least),
/// so as a precaution, all objects are ID'd and
/// must be looked up, instead of assuming pointers are valid.
pub static DATA: LazyLock<RwLock<TypeMap>>
    = LazyLock::new(Default::default);



