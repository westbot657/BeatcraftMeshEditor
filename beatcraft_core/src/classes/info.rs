#![allow(deprecated)] // TODO: remove when jni updates

//

use std::path::PathBuf;

use bs_mapping_data::InfoFile;
use java_jni_extras::*;
use jni::Env;

use crate::DATA;

pub(crate) fn _validate_interfaces(env: &mut Env) -> jni::errors::Result<()> {
    Info::_validate_interface(env)?;
    SetInfo::_validate_interface(env)?;
    DifficultyInfo::_validate_interface(env)?;
    Ok(())
}

java_class! {
    package com.beatcraft.interop;

    import java.io.IOException;

    class Info {
        handle: long;

        static native fn load(path: String) -> Info {
            let path = PathBuf::from(path.to_string());
            let info = Box::new(
                InfoFile::load_from_folder(&path)
                    .map_err(|e| {
                        #throw new IOException(e.to_string()).unwrap_err();
                    })?
            );
            let info_key = DATA.write().insert(info);

            let info_obj = #alloc Info();
            #info_obj.handle = info_key;

            info_obj
        }

    }
}


java_class! {
    package com.beatcraft.interop;

    class SetInfo {

    }
}

java_class! {
    package com.beatcraft.interop;

    use crate::classes::beatmap::com.beatcraft.interop.BeatmapController;

    class DifficultyInfo {
        handle: long;

        static native fn load() -> BeatmapController {
            let controller = #alloc BeatmapController();

            #controller.handle = 0;

            controller
        }
    }
}
