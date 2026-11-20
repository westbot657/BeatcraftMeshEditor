#![allow(deprecated)] // TODO: remove when jni updates

use std::ops::{Deref, DerefMut};
use beatmap_core::BeatmapController as MapController;

use glam::Vec3;
use java_jni_extras::*;
use jni::objects::JObject;
use jni::Env;
use crate::DATA;

pub struct BeatmapControllerExtension {
    pub controller: MapController<()>,
    pub origin: Vec3,
}

impl Deref for BeatmapControllerExtension {
    type Target = MapController<()>;
    fn deref(&self) -> &Self::Target {
        &self.controller
    }
}
impl DerefMut for BeatmapControllerExtension {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.controller
    }
}

pub(crate) fn _validate_interfaces(env: &mut Env) -> jni::errors::Result<()> {
    BeatmapController::_validate_interface(env)?;
    RuntimeData::_validate_interface(env)?;
    ColorScheme::_validate_interface(env)?;

    Ok(())
}

java_class! {
    package com.beatcraft.interop;

    import org.joml.Vector3f;
    import java.lang.NullPointerException;

    class BeatmapController {
        handle: long;

        native fn getOrigin(out: Vector3f) -> Vector3f {
            let handle = #this.handle as u64;

            let origin = DATA.with_borrow(|map| {
                let Some(controller) = map.get::<BeatmapControllerExtension>(handle.into()) else {
                    #throw new NullPointerException("handle is invalid");
                };
                Ok(controller.origin)
            })?;

            #out.x #["F"] = origin.x;
            #out.y #["F"] = origin.y;
            #out.z #["F"] = origin.z;

            out
        }

        native fn setOrigin(origin: Vector3f) {
            let handle = #this.handle as u64;

            let x = #origin.x #["F"];
            let y = #origin.y #["F"];
            let z = #origin.z #["F"];

            DATA.with_borrow_mut(|data| {
                let Some(controller) = data.get_mut::<BeatmapControllerExtension> (handle.into()) else {
                    #throw new NullPointerException("handle is invalid");
                };
                controller.origin = Vec3::new(x, y, z);
                Ok(())
            })?;
        }

    }
}

java_class! {
    package com.beatcraft.interop;

    use self::com.beatcraft.interop.ColorScheme;

    class RuntimeData {
        handle: long;

        native fn getNjs() -> float {
            0.
        }

        native fn getBpm() -> float {
            0.
        }

        native fn getSpawnOffset() -> float {
            0.
        }

        native fn getColorScheme() -> ColorScheme {

            ColorScheme(JObject::null())
        }
    }
}

java_class! {
    package com.beatcraft.interop;

    import org.joml.Vector4f;
    import java.lang.NullPointerException;

    class ColorScheme {
        handle: long;


        static native fn default() -> ColorScheme {
            let cs = beatmap_core::ColorScheme::default();
            let key = DATA.with_borrow_mut(|data| data.insert(cs));
            let scheme = #alloc ColorScheme();

            #scheme.handle = key;

            scheme
        }

        native fn leftNote(out: Vector4f) -> Vector4f {
            let key = #this.handle;
            let l = DATA.with_borrow(|d| {
                let Some(cs) = d.get::<beatmap_core::ColorScheme>(key.into()) else {
                    #throw new NullPointerException("handle is invalid");
                };
                Ok(cs.left_note)
            })?;

            #out.x #["F"] = l.x;
            #out.y #["F"] = l.y;
            #out.z #["F"] = l.z;
            #out.w #["F"] = l.w;

            out
        }

        native fn rightNote(out: Vector4f) -> Vector4f {
            let key = #this.handle;
            let l = DATA.with_borrow(|d| {
                let Some(cs) = d.get::<beatmap_core::ColorScheme>(key.into()) else {
                    #throw new NullPointerException("handle is invalid");
                };
                Ok(cs.right_note)
            })?;

            #out.x #["F"] = l.x;
            #out.y #["F"] = l.y;
            #out.z #["F"] = l.z;
            #out.w #["F"] = l.w;

            out
        }

    }
}


