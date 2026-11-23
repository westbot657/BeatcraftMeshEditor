use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::{BeatmapFile, InfoFile};
use crate::info_v2::InfoV2;
use crate::info_v4::InfoV4;
use crate::v2::BeatmapFileV2;
use crate::v3::BeatmapFileV3;
use crate::v4::{BeatmapFileV4, LightshowFileV4};

pub mod v2;
pub mod v3;
pub mod v4;

#[derive(thiserror::Error, Debug)]
pub enum DataError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

pub fn try_parse_info(map_folder: &Path) -> Result<Option<InfoFile>, DataError> {
    for file in std::fs::read_dir(map_folder)? {
        let file = file?;
        if let path = file.path()
            && path.is_file()
            && path.file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase() == "info.dat"
        {
            let data = std::fs::read_to_string(path)?;
            let v2: Result<InfoV2, _> = serde_json::from_str(&data);
            match v2 {
                Ok(v2) => return Ok(Some(InfoFile::V2(v2))),
                Err(e) => println!("{e:?}")
            }
            let v4: Result<InfoV4, _> = serde_json::from_str(&data);
            match v4 {
                Ok(v4) => return Ok(Some(InfoFile::V4(v4))),
                Err(e) => println!("{e:?}")
            }
        }
    }

    Ok(None)
}

pub fn try_parse_beatmap(map_file: &Path) -> Result<BeatmapFile, DataError> {
    let data = std::fs::read_to_string(map_file)?;
    let v2: Result<BeatmapFileV2, _> = serde_json::from_str(&data);
    match v2 {
        Ok(v2) => return Ok(BeatmapFile::V2(v2)),
        Err(e) => println!("{e:?}"),
    }
    let v3: Result<BeatmapFileV3, _> = serde_json::from_str(&data);
    match v3 {
        Ok(v3) => return Ok(BeatmapFile::V3(v3)),
        Err(e) => println!("{e:?}"),
    }
    let v4: Result<BeatmapFileV4, _> = serde_json::from_str(&data);
    match v4 {
        Ok(v4) => Ok(BeatmapFile::V4(v4)),
        Err(e) => {
            println!("{e:?}");
            Err(e.into())
        }
    }
}

#[test]
pub fn test_deserialize_all() -> Result<(), DataError> {

    const BEATMAPS: &str = "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps";

    let Ok(folder) = PathBuf::from_str(BEATMAPS);

    if let Ok(dir) = std::fs::read_dir(&folder) {
        for map in dir {
            if let Ok(map) = map {
                let map = map.path();
                println!("attempting to read map: {}", map.to_string_lossy());
                if let Some(info) = try_parse_info(&map)? {
                    println!("{info:#?}");

                    match info {
                        InfoFile::V2(v2) => {
                            for set in v2.difficulty_beatmap_sets {
                                for diff in set.difficulty_beatmaps {
                                    println!("Reading map: {}", diff.beatmap_filename);
                                    let beatmap_file = map.join(diff.beatmap_filename);
                                    let _diff = try_parse_beatmap(&beatmap_file)?;
                                }
                            }
                        }
                        InfoFile::V4(v4) => {
                            for diff in v4.difficulty_beatmaps {
                                println!("Reading map: {}", diff.beatmap_data_filename);
                                let beatmap_file = map.join(diff.beatmap_data_filename);
                                let _diff: BeatmapFile = try_parse_beatmap(&beatmap_file)?;
                                println!("Reading lightshow: {}", diff.lightshow_data_filename);
                                let lightshow_file = map.join(diff.lightshow_data_filename);
                                let data = std::fs::read_to_string(lightshow_file)?;
                                let _lightshow: LightshowFileV4 = serde_json::from_str(&data)?;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

