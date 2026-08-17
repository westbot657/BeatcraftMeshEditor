
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use egui::TextBuffer;

use crate::beatmap::data::v2::BeatmapFileV2;
use crate::beatmap::data::v3::BeatmapFileV3;
use crate::beatmap::data::v4::{BeatmapFileV4, InfoV4};

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
// 2.0.0 | 3.0.0
static ASCENT: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/2a629 (Ascent - nitronik.exe)",
    "Standard", "ExpertPlus"
);
// 2.0.0 | 2.0.0
static REALITY_CHECK: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/25f (Reality Check Through The Skull - DM DOKURO)",
    "Standard", "ExpertPlus"
);

// 4.0.0 | 4.0.0
static MOONBEAM: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/Moonbeam",
    "Standard", "ExpertPlus"
);

// 4.0.0 | 4.1.0
static PLAYFUL_MASSACRE: MapSelection = (
    "/home/westbot/IdeaProjects/BeatCraft/fabric/run/beatmaps/Playful Massacre - Song for Wemmbu",
    "Standard", "ExpertPlus"
);


static TEST_MAPS_V2: [MapSelection; 7] = [
    SOMEWHERE_OUT_THERE,
    RINGED_GENESIS,
    HEADHUNTER,
    SEQUENCE_EP,
    TENEBROUS,
    CHEAT_CODES,
    REALITY_CHECK,
];

static TEST_MAPS_V3: [MapSelection; 4] = [
    ILL_SHARP_MINOR,
    SPIN_ETERNALLY,
    GHOST,
    ASCENT,
];

static TEST_MAPS_V4: [MapSelection; 2] = [
    MOONBEAM,
    PLAYFUL_MASSACRE,
];

static TEST_MAPS: [MapSelection; 12] = [
    SOMEWHERE_OUT_THERE,
    RINGED_GENESIS,
    HEADHUNTER,
    SEQUENCE_EP,
    TENEBROUS,
    CHEAT_CODES,
    ILL_SHARP_MINOR,
    SPIN_ETERNALLY,
    GHOST,
    ASCENT,
    REALITY_CHECK,
    MOONBEAM,
];

static NOODLE_V2_MAPS: [MapSelection; 3] = [
    TENEBROUS,
    SEQUENCE_EP,
    SOMEWHERE_OUT_THERE,
];

fn open_info<V: serde::de::DeserializeOwned>(folder: &Path) -> Result<V> {
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
    let info: V = serde_json::from_slice(&data)?;
    Ok(info)
}

fn open_char_diff_v4<S, V>(folder: &Path, info: &InfoV4, set: S, difficulty: &str) -> Result<V>
where
    MapCharacteristic: PartialEq<S>,
    V: serde::de::DeserializeOwned,
{
    let mut found = None;
    for diff in info.difficulty_beatmaps.iter() {
        if diff.characteristic == set && diff.difficulty == difficulty {
            found = Some(diff);
        }
    }
    let map = found.ok_or(anyhow!("set/difficutly not found"))?;
    let file = folder.join(&map.beatmap_data_filename);
    let data = fs::read(file)?;
    let map: V = serde_json::from_slice(&data)?;
    Ok(map)
}

fn open_char_diff_v2<S, V>(folder: &Path, info: &InfoV2, set: S, difficulty: &str) -> Result<V>
where
    MapCharacteristic: PartialEq<S>,
    V: serde::de::DeserializeOwned,
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
    let map: V = serde_json::from_slice(&data)?;
    Ok(map)
}

#[test]
fn deserialize_vanilla_map_files_v3() -> Result<()> {

    let path = PathBuf::from(ASCENT.0);
    let info: InfoV2 = open_info(&path)?;

    println!("Ascent Info.dat:\n{info:#?}");

    let exp = open_char_diff_v2::<_, BeatmapFileV3>(
        &path, &info,
        CHEAT_CODES.1, CHEAT_CODES.2
    )?;

    println!("Ascent data:\n{exp:#?}");

    Ok(())
}

#[test]
fn test_deserialize_all_v4() -> Result<()> {
    for (file, set, diff) in TEST_MAPS_V4 {
        println!("parsing {}", file);

        let path = PathBuf::from(file);
        let info: InfoV4 = open_info(&path)?;
        println!("Info for {}:\n{:#?}", file, info);

        let exp: BeatmapFileV4 = open_char_diff_v4(&path, &info, set, diff)?;

        println!("map for {}:\n{:?}", file, exp);
    }
    Ok(())
}

#[test]
fn test_deserialize_all_v3() -> Result<()> {
    for (file, set, diff) in TEST_MAPS_V3 {
        println!("parsing {}", file);

        let path = PathBuf::from(file);
        let info: InfoV2 = open_info(&path)?;
        println!("Info for {}:\n{:#?}", file, info);

        let exp: BeatmapFileV3 = open_char_diff_v2(&path, &info, set, diff)?;

        println!("map for {}:\n{:?}", file, exp);
    }
    Ok(())
}

#[test]
fn test_deserialize_all_v2() -> Result<()> {
    for (file, set, diff) in TEST_MAPS_V2 {
        println!("parsing {}", file);

        let path = PathBuf::from(file);
        let info: InfoV2 = open_info(&path)?;
        println!("Info for {}:\n{:#?}", file, info);

        let exp: BeatmapFileV2 = open_char_diff_v2(&path, &info, set, diff)?;

        println!("map for {}:\n{:?}", file, exp);
    }
    Ok(())
}

#[test]
pub fn deserialize_vanilla_map_files_v2() -> Result<()> {

    let path = PathBuf::from(CHEAT_CODES.0);
    let info: InfoV2 = open_info(&path)?;

    println!("Cheat Codes Info.dat:\n{info:#?}");

    let mut exp = open_char_diff_v2::<_, BeatmapFileV2>(
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
        let info: InfoV2 = open_info(&path)?;

        println!("Info for {}:\n{:#?}", file, info);

        let exp = open_char_diff_v2::<_, BeatmapFileV2>(
            &path, &info,
            set, diff
        )?;

        println!("map for {}:\n{:?}", file, exp);
    }
    Ok(())
}

