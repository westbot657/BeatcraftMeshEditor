#![allow(non_snake_case, static_mut_refs)]

use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::sync::{LazyLock};
use java_jni_extras::*;
use jni::strings::{JNIStr, JNIString};
use jni::{Env, JValue, jni_sig, jni_str};
use jni::objects::JClass;
use jni::refs::Global;
use jni::sys::jlong;
use parking_lot::RwLock;
use slotmap::{new_key_type, SlotMap, KeyData};
use beatmap_core::BeatmapController;
use bs_mapping_data::InfoFile;

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


java_class! {
    package com.beatcraft.interop;

    class BeatcraftCore {
        static native fn beatcraftCoreInit() {
            BeatcraftCore::_validate_interface(env)?;
            Info::_validate_interface(env)?;

            init_types(env)?;
        }

        static fn logInfo(msg: String);
        static fn logWarn(msg: String);
        static fn logCritical(msg: String);
        static fn logDebug(msg: String);
    }
}

new_key_type! { pub struct DataKey; }
impl From<u64> for DataKey {
    fn from(value: u64) -> Self {
        Self::from(KeyData::from_ffi(value))
    }
}

impl From<DataKey> for JValue<'_> {
    fn from(value: DataKey) -> Self {
        (value.0.as_ffi() as jlong).into()
    }
}

pub enum DataItem {
    Info(Box<InfoFile>),
    BeatmapController(Box<BeatmapController<()>>)
}

/// Safety:
/// Passing raw pointers to java may be unsafe (in the modding environment at least),
/// so as a precausion, all objects are id'd and
/// must be looked up, instead of assuming pointers are valid.
pub static DATA: LazyLock<RwLock<SlotMap<DataKey, DataItem>>> = LazyLock::new(Default::default);

java_class! {
    package com.beatcraft.interop;

    class Info {
        static native fn load(path: String) -> Info {
            let path = PathBuf::from(path.to_string());
            let info = Box::new(
                InfoFile::load_from_folder(&path)
                    .map_err(|e| {
                        let s = JNIString::new(e.to_string());
                        env.throw_new(
                            jni_str!("java.io.IOException"),
                            s
                        ).unwrap_err()
                    })?
            );
            let info_key = DATA.write().insert(DataItem::Info(info));

            let info_obj = env.alloc_object(jni_str!("com.beatcraft.interop.Info"))?;
            env.set_field(&info_obj, jni_str!("handle"), jni_sig!("J"), info_key.into())?;

            info_obj
        }

    }
}

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


