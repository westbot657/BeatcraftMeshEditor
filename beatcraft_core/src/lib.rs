#![allow(non_snake_case, static_mut_refs)]
use std::mem::MaybeUninit;

use java_jni_extras::java_class_decl;
use jni::strings::JNIStr;
use jni::{Env, JValue, jni_sig, jni_str};
use jni::objects::{JClass, JObject, JString};
use jni::refs::Global;

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

java_class_decl! {
    package com.beatcraft.interop;

    class BeatcraftCore {
        static native void beatcraftCoreInit();

        static void logInfo(String msg);
        static void logWarn(String msg);
        static void logCritical(String msg);
        static void logDebug(String msg);
    }
}

java_class_decl! {
    package com.beatcraft.interop;

    class Beatmap {
        static native com.beatcraft.interop.Beatmap load(java.lang.String path);
        // native java.lang.String[] getSets();
        // native java.lang.String[] getDiffs(java.lang.String set);
        // native void openDiff(java.lang.String set, java.lang.String diff);
    }
}


fn get_global_class<S>(env: &mut Env, name: S) -> Result<GlbCls, jni::errors::Error>
where
    S: AsRef<JNIStr>,
{
    let cls = env.find_class(name)?;
    env.new_global_ref(cls)
}

fn init_types<'c>(env: &mut Env<'c>) -> Result<(), jni::errors::Error> {
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


// Class functions

fn beatcraftCoreInit<'c>(
    env: &mut Env<'c>,
    _class: JClass<'c>,
) -> Result<(), jni::errors::Error> {
    // Class link validation
    BeatcraftCore::_validate_interface(env)?;

    init_types(env)?;

    Ok(())
}


fn load<'c>(
    env: &mut Env<'c>,
    _class: JClass<'c>,
    path: JString<'c>,
) -> Result<JObject<'c>, jni::errors::Error> {

    let obj = env.alloc_object(jni_str!("com/beatcraft/interop/Beatmap"))?;

    env.set_field(&obj, jni_str!("path"), jni_sig!("Ljava.lang.String"), JValue::from(&path))?;

    Ok(obj)
}



