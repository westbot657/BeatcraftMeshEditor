use std::collections::HashMap;
use std::f32;

use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use rand::{RngExt, rngs::ThreadRng};

use crate::data::map_editing::ObjectSource;
use crate::render::GameObjectInstanceData;

use super::BeatmapProjectDiff;
use bs_mapping_data::{
    BeatmapDataError, BeatmapFile, BpmRegion,
    CutDirection, InfoFile, v2,
};

use beatmap_core::*;

pub trait GameObjectExt: GameObject<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData;
    fn get_editor_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
        _beat_distance: f32,
    ) -> GameObjectInstanceData {
        self.get_instance(clipping_plane, model, cs)
    }
}

impl GameObjectExt for ColorNote<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}

impl GameObjectExt for BombNote<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::bomb_note(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}

impl GameObjectExt for Obstacle<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::obstacle(
            clipping_plane,
            model * Mat4::from_translation(Vec3::new(0., 0., -0.25)),
            self.color.color(cs),
            self.dissolve,
            self.index,
            self.size,
        )
    }
    fn get_editor_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
        beat_distance: f32,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::obstacle(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec3::new(self.size.x, self.size.y, self.duration * beat_distance),
        )
    }
}

impl GameObjectExt for ChainNote<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}

impl GameObjectExt for ChainNoteLink<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            self.color.color(cs),
            self.dissolve,
            self.index,
            Vec4::ZERO,
        )
    }
}

impl GameObjectExt for Arc<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        cs: &ColorScheme,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::arc(clipping_plane, model, self.color.color(cs))
    }
}

pub fn get_random_spawn_quat(rng: &mut ThreadRng) -> Quat {
    let get_c = |rng: &mut ThreadRng| -> f32 { rng.random::<f32>() * 0.6f32 * 2. - 0.6 };
    Quat::from_euler(glam::EulerRot::ZYX, get_c(rng), get_c(rng), get_c(rng))
}
