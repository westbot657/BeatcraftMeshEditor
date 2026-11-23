use std::f32;

use glam::{Mat4, Quat, Vec3, Vec4};
use rand::{RngExt, rngs::ThreadRng};

use crate::data::map_editing::object_data::ObjectSource;
use crate::render::GameObjectInstanceData;


use beatmap_core::beatmap::{Arc, BombNote, ChainNote, ChainNoteLink, ColorNote, GameObject, Obstacle};
use beatmap_core::*;
use beatmap_core::tracks_rs::base_value::BaseValue;

pub trait GameObjectExt: GameObject<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        lifetime: f32,
        rt: &RuntimeData,
    ) -> GameObjectInstanceData;
    fn get_editor_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        lifetime: f32,
        rt: &RuntimeData,
        _beat_distance: f32,
    ) -> GameObjectInstanceData {
        self.get_instance(clipping_plane, model, lifetime, rt)
    }
}

impl GameObjectExt for ColorNote<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        lifetime: f32,
        rt: &RuntimeData,
    ) -> GameObjectInstanceData {
        let mut color = self.color.color(&rt.color_scheme);
        let mut dissolve = self.dissolve;
        let mut arrow_dissolve = self.dissolve;

        if let Some(a) = &self.custom_data.animation {
            if let Some(BaseValue::Vector4(col)) = a.color.interpolate(lifetime, &rt.base_provider_context) {
                color = col;
            }
            if let Some(BaseValue::Float(dis)) = a.dissolve.interpolate(lifetime, &rt.base_provider_context) {
                dissolve = 1. - dis;
            }
            if let Some(BaseValue::Float(dis)) = a.dissolve_arrow.interpolate(lifetime, &rt.base_provider_context) {
                arrow_dissolve = 1. - dis;
            }
        }
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            color,
            dissolve,
            arrow_dissolve,
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
        lifetime: f32,
        rt: &RuntimeData,
    ) -> GameObjectInstanceData {
        let mut color = self.color.color(&rt.color_scheme);
        let mut dissolve = self.dissolve;

        if let Some(a) = &self.custom_data.animation {
            if let Some(BaseValue::Vector4(col)) = a.color.interpolate(lifetime, &rt.base_provider_context) {
                color = col;
            }
            if let Some(BaseValue::Float(dis)) = a.dissolve.interpolate(lifetime, &rt.base_provider_context) {
                dissolve = 1. - dis;
            }
        }
        GameObjectInstanceData::bomb_note(
            clipping_plane,
            model,
            color,
            dissolve,
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
        lifetime: f32,
        rt: &RuntimeData,
    ) -> GameObjectInstanceData {

        let mut color = self.color.color(&rt.color_scheme);
        let mut dissolve = self.dissolve;
        let mut size = self.size;

        if let Some(a) = &self.custom_data.animation {
            if let Some(BaseValue::Vector4(col)) = a.color.interpolate(lifetime, &rt.base_provider_context) {
                color = col;
            }
            if let Some(BaseValue::Float(dis)) = a.dissolve.interpolate(lifetime, &rt.base_provider_context) {
                dissolve = 1. - dis;
            }
            if let Some(BaseValue::Vector3(v3)) = a.scale.interpolate(lifetime, &rt.base_provider_context) {
                size *= v3;
            }
        }

        GameObjectInstanceData::obstacle(
            clipping_plane,
            model * Mat4::from_translation(Vec3::new(0., 0., -0.25)),
            color,
            dissolve,
            self.index,
            size,
        )
    }
    fn get_editor_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        lifetime: f32,
        rt: &RuntimeData,
        beat_distance: f32,
    ) -> GameObjectInstanceData {
        let mut color = self.color.color(&rt.color_scheme);
        let mut dissolve = self.dissolve;

        if let Some(a) = &self.custom_data.animation {
            if let Some(BaseValue::Vector4(col)) = a.color.interpolate(lifetime, &rt.base_provider_context) {
                color = col;
            }
            if let Some(BaseValue::Float(dis)) = a.dissolve.interpolate(lifetime, &rt.base_provider_context) {
                dissolve = 1. - dis;
            }
        }

        GameObjectInstanceData::obstacle(
            clipping_plane,
            model,
            color,
            dissolve,
            self.index,
            Vec3::new(self.size.x, self.size.y, if self.noodle_logic { self.size.z } else { self.duration * beat_distance }),
        )
    }
}

impl GameObjectExt for ChainNote<ObjectSource> {
    fn get_instance(
        &self,
        clipping_plane: Vec4,
        model: Mat4,
        lifetime: f32,
        rt: &RuntimeData,
    ) -> GameObjectInstanceData {
        let mut color = self.color.color(&rt.color_scheme);
        let mut dissolve = self.dissolve;
        let mut arrow_dissolve = self.dissolve;

        if let Some(a) = &self.custom_data.animation {
            if let Some(BaseValue::Vector4(col)) = a.color.interpolate(lifetime, &rt.base_provider_context) {
                color = col;
            }
            if let Some(BaseValue::Float(dis)) = a.dissolve.interpolate(lifetime, &rt.base_provider_context) {
                dissolve = 1. - dis;
            }
            if let Some(BaseValue::Float(dis)) = a.dissolve_arrow.interpolate(lifetime, &rt.base_provider_context) {
                arrow_dissolve = 1. - dis;
            }
        }
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            color,
            dissolve,
            arrow_dissolve,
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
        lifetime: f32,
        rt: &RuntimeData,
    ) -> GameObjectInstanceData {
        let mut color = self.color.color(&rt.color_scheme);
        let mut dissolve = self.dissolve;
        let mut arrow_dissolve = self.dissolve;

        if let Some(a) = &self.custom_data.animation {
            if let Some(BaseValue::Vector4(col)) = a.color.interpolate(lifetime, &rt.base_provider_context) {
                color = col;
            }
            if let Some(BaseValue::Float(dis)) = a.dissolve.interpolate(lifetime, &rt.base_provider_context) {
                dissolve = 1. - dis;
            }
            if let Some(BaseValue::Float(dis)) = a.dissolve_arrow.interpolate(lifetime, &rt.base_provider_context) {
                arrow_dissolve = 1. - dis;
            }
        }
        GameObjectInstanceData::color_note(
            clipping_plane,
            model,
            color,
            dissolve,
            arrow_dissolve,
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
        _lifetime: f32,
        rt: &RuntimeData,
    ) -> GameObjectInstanceData {
        GameObjectInstanceData::arc(clipping_plane, model, self.color.color(&rt.color_scheme))
    }
}

pub fn get_random_spawn_quat(rng: &mut ThreadRng) -> Quat {
    let get_c = |rng: &mut ThreadRng| -> f32 { rng.random::<f32>() * 0.6f32 * 2. - 0.6 };
    Quat::from_euler(glam::EulerRot::ZYX, get_c(rng), get_c(rng), get_c(rng))
}
