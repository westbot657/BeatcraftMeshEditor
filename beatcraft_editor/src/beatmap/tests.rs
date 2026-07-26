
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use egui::TextBuffer;

use crate::beatmap::data::v2::BeatmapFileV2;

use super::data::MapCharacteristic;
use super::data::v2::InfoV2;

type MapSelection = (&'static str, &'static str, &'static str);

// 2.0.0 | 2.1.0
static SOMEWHERE_OUT_THERE: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/1e6ff (Somewhere Out There - Swifter_ Mawntee_ Reddek)",
    "Standard", "Expert"
);
// 2.0.0 | 2.2.0
static RINGED_GENESIS: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/1694f (Ringed Genesis - That_Narwhal)",
    "360Degree", "ExpertPlus"
);
// 2.0.0 | 2.2.0
static HEADHUNTER: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/28043 (HEADHUNTER - Swifter)",
    "Standard", "ExpertPlus"
);
// 2.0.0 | 2.0.0
static SEQUENCE_EP: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/14c34 (SEQUENCE EP - Swifter1243)",
    "Standard", "ExpertPlus"
);
// 2.1.0 | 3.3.0
static ILL_SHARP_MINOR: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/40e94 (Ill Sharp Minor - UglyApe)",
    "Standard", "ExpertPlus"
);
// 2.0.0 | 2.0.0
static TENEBROUS: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/11f9c (Tenebrous - Swifter1243)",
    "Standard", "ExpertPlus"
);
// 2.1.0 | 3.3.0
static SPIN_ETERNALLY: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/43774 (Spin Eternally - (MaRrAtOk)____ _______)",
    "Standard", "Expert"
);
// 2.0.0 | 3.0.0
static GHOST: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/2c878 (GHOST - Gaming James 828)",
    "Standard", "ExpertPlus"
);
// 2.0.0 | 2.2.0
static CHEAT_CODES: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/29341 (Cheat Codes - Avexus)",
    "Standard", "ExpertPlus"
);



static TEST_MAPS_V2: [MapSelection; 6] = [
    SOMEWHERE_OUT_THERE,
    RINGED_GENESIS,
    HEADHUNTER,
    SEQUENCE_EP,
    TENEBROUS,
    CHEAT_CODES,
];

static TEST_MAPS_V3: [MapSelection; 3] = [
    ILL_SHARP_MINOR,
    SPIN_ETERNALLY,
    GHOST,
];

static TEST_MAPS: [MapSelection; 9] = [
    SOMEWHERE_OUT_THERE,
    RINGED_GENESIS,
    HEADHUNTER,
    SEQUENCE_EP,
    TENEBROUS,
    CHEAT_CODES,
    ILL_SHARP_MINOR,
    SPIN_ETERNALLY,
    GHOST,
];

static NOODLE_V2_MAPS: [MapSelection; 3] = [
    TENEBROUS,
    SEQUENCE_EP,
    SOMEWHERE_OUT_THERE,
];

fn open_info_v2(folder: &Path) -> Result<InfoV2> {
    let files = fs::read_dir(folder)?;
    let mut info_file = None;
    for file in files {
        let file = file?;
        match file.file_name().to_string_lossy().as_str() {
            "info.dat" |
            "Info.dat" => {
                info_file = Some(file);
                break
            },
            _ => continue
        }
    }
    let file = info_file.ok_or(anyhow!("no info file found"))?;
    let path = &file.path();
    let data = fs::read(path)?;
    let info: InfoV2 = serde_json::from_slice(&data)?;
    Ok(info)
}

fn open_char_diff<S>(folder: &Path, info: &InfoV2, set: S, difficulty: &str) -> Result<BeatmapFileV2>
where
    MapCharacteristic: PartialEq<S>
{
    let mut found = None;
    for sets in info.difficulty_beatmap_sets.iter() {
        if sets.beatmap_characteristic_name == set {
            for diff in sets.difficulty_beatmaps.iter() {
                if diff.difficulty == difficulty {
                    found = Some(diff)
                }
            }
        }
    }
    let map = found.ok_or(anyhow!("set/difficutly not found"))?;
    let file = folder.join(&map.beatmap_filename);
    let data = fs::read(file)?;
    let map: BeatmapFileV2 = serde_json::from_slice(&data)?;
    Ok(map)
}

#[test]
pub fn deserialize_vanilla_map_files_v2() -> Result<()> {

    let path = PathBuf::from(CHEAT_CODES.0);
    let info = open_info_v2(&path)?;

    println!("Cheat Codes Info.dat:\n{info:#?}");

    let mut exp = open_char_diff(
        &path, &info,
        CHEAT_CODES.1, CHEAT_CODES.2
    )?;

    exp.events.clear();
    println!("Cheat codes data:\n{exp:#?}");

    Ok(())
}

#[test]
pub fn deserialize_noodle_map_files_v2() -> Result<()> {
    for (file, set, diff) in NOODLE_V2_MAPS {
        println!("parsing {}", file);

        let path = PathBuf::from(file);
        let info = open_info_v2(&path)?;

        println!("Info for {}:\n{:#?}", file, info);

        let exp = open_char_diff(
            &path, &info,
            set, diff
        )?;

        println!("map for {}:\n{:?}", file, exp);
    }
    Ok(())
}

