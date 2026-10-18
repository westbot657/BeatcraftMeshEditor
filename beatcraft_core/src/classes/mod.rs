#![allow(deprecated)] // TODO: remove when jni updates
use java_jni_extras::*;
use jni::{Env, jni_str};
use jni::strings::JNIStr;

use crate::{GlbCls, JavaTypes, TYPES};

pub mod info;
pub mod beatmap;

fn get_global_class<S>(env: &mut Env, name: S) -> Result<GlbCls, jni::errors::Error>
where
    S: AsRef<JNIStr>,
{
    let cls = env.find_class(name)?;
    env.new_global_ref(cls)
}

fn init_types(env: &mut Env<'_>) -> Result<(), jni::errors::Error> {
    let types = JavaTypes {
        i8: get_global_class(env, jni_str!("java/lang/Byte"))?,
        i16: get_global_class(env, jni_str!("java/lang/Short"))?,
        i32: get_global_class(env, jni_str!("java/lang/Integer"))?,
        i64: get_global_class(env, jni_str!("java/lang/Long"))?,
        u8: get_global_class(
            env,
            jni_str!("com/beatcraft/interop/unsigned/UByte"),
        )?,
        u16: get_global_class(
            env,
            jni_str!("com/beatcraft/interop/unsigned/UShort"),
        )?,
        u32: get_global_class(
            env,
            jni_str!("com/beatcraft/interop/unsigned/UInt"),
        )?,
        u64: get_global_class(
            env,
            jni_str!("com/beatcraft/interop/unsigned/ULong"),
        )?,
        f32: get_global_class(env, jni_str!("java/lang/Float"))?,
        f64: get_global_class(env, jni_str!("java/lang/Double"))?,
        bool: get_global_class(env, jni_str!("java/lang/Boolean"))?,
        string: get_global_class(env, jni_str!("java/lang/String"))?,
        error: get_global_class(
            env,
            jni_str!("com/beatcraft/interop/BeatcraftError"),
        )?,
        vec2: get_global_class(env, jni_str!("org/joml/Vector2f"))?,
        vec3: get_global_class(env, jni_str!("org/joml/Vector3f"))?,
        vec4: get_global_class(env, jni_str!("org/joml/Vector4f"))?,
        quat: get_global_class(env, jni_str!("org/joml/Quaternionf"))?,
        mat4: get_global_class(env, jni_str!("org/joml/Matrix4f"))?,
        u8buf: get_global_class(env, jni_str!("[B"))?,
        object: get_global_class(env, jni_str!("java/lang/Object"))?,
    };

    unsafe { TYPES.write(types) };

    Ok(())
}

pub(crate) fn jtypes() -> &'static JavaTypes {
    unsafe { TYPES.assume_init_ref() }
}


java_class! {
    package com.beatcraft.interop;

    class BeatcraftCore {
        static native fn init() {
            init_types(env)?;
            BeatcraftCore::_validate_interface(env)?;
            info::_validate_interfaces(env)?;
            beatmap::_validate_interfaces(env)?;
        }
    }
}









