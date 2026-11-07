
pub mod spline;
pub mod math;
pub mod beatmap;
pub mod lightshow;
pub mod tracks;

use std::fmt::Debug;

use beatcraft_editor_proc::hex;
use bs_mapping_data::BpmRegion;
use beatmap::{Arc, BombNote, ChainNote, ColorNote, Obstacle};
use glam::{Vec3, Vec4, FloatExt};


pub struct BeatmapController<ObjectSource> {
    pub runtime_data: RuntimeData,
    pub color_notes: Vec<ColorNote<ObjectSource>>,
    pub bomb_notes: Vec<BombNote<ObjectSource>>,
    pub obstacles: Vec<Obstacle<ObjectSource>>,
    pub chain_notes: Vec<ChainNote<ObjectSource>>,
    pub arcs: Vec<Arc<ObjectSource>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeData {
    pub njs: f32,
    pub bpm: f32,
    pub spawn_offset: f32,
    pub color_scheme: ColorScheme,
    pub bpm_regions: Vec<BpmRegion>,
    pub sample_count: usize,
    pub sample_rate: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimeUnit {
    Beat(f32),
    Seconds(f32),
    Sample(usize),
}

impl RuntimeData {
    pub fn new(
        njs: f32,
        bpm: f32,
        spawn_offset: f32,
        bpm_regions: Vec<BpmRegion>,
        sample_count: usize,
        sample_rate: u32,
    ) -> Self {
        Self {
            njs,
            bpm,
            spawn_offset,
            color_scheme: Default::default(),
            bpm_regions,
            sample_count,
            sample_rate,
        }
    }

    pub fn base_jumps(&self) -> (f32, f32) {
        Self::calc_jumps(self.njs, self.bpm, self.spawn_offset)
    }

    pub fn jumps(&self, beat: f32) -> (f32, f32) {
        let bpm = self.bpm(TimeUnit::Beat(beat));
        Self::calc_jumps(self.njs, bpm, self.spawn_offset)
    }

    pub fn seconds_to_beat(&self, seconds: f32) -> f32 {
        let sample = seconds as f64 * self.sample_rate as f64;
        for BpmRegion {
            start_sample,
            end_sample,
            start_beat,
            end_beat,
        } in self.bpm_regions.iter()
        {
            if (*start_sample..=*end_sample).contains(&(sample as usize)) {
                let x = f64::inverse_lerp(*start_sample as f64, *end_sample as f64, sample);
                return f64::lerp(*start_beat, *end_beat, x) as f32;
            }
        }
        (seconds as f64 / (60. / self.bpm as f64)) as f32
    }

    pub fn beat_to_seconds(&self, beat: f32) -> f32 {
        let sample = self.beat_to_sample(beat as f64);
        (sample as f64 / self.sample_rate as f64) as f32
    }

    fn beat_to_sample(&self, beat: f64) -> usize {
        for BpmRegion {
            start_sample,
            end_sample,
            start_beat,
            end_beat,
        } in self.bpm_regions.iter()
        {
            if (*start_beat..=*end_beat).contains(&beat) {
                let x = f64::inverse_lerp(*start_beat, *end_beat, beat);
                return f64::lerp(*start_sample as f64, *end_sample as f64, x) as usize;
            }
        }
        let s = beat * (60. / self.bpm as f64);
        (s * self.sample_rate as f64) as usize
    }

    pub fn bpm(&self, time: TimeUnit) -> f32 {
        let time_samples = match time {
            TimeUnit::Sample(s) => s,
            TimeUnit::Seconds(s) => (s * self.sample_rate as f32) as usize,
            TimeUnit::Beat(b) => self.beat_to_sample(b as f64),
        };

        for BpmRegion {
            start_sample,
            end_sample,
            start_beat,
            end_beat,
        } in self.bpm_regions.iter()
        {
            if (*start_sample..=*end_sample).contains(&time_samples) {
                let seconds = (*end_sample - *start_sample) as f32 / self.sample_rate as f32;
                let beats = *end_beat - *start_beat;
                return (beats / seconds as f64 * 60.) as f32;
            }
        }
        self.bpm
    }

    fn calc_jumps(njs: f32, bpm: f32, spawn_offset: f32) -> (f32, f32) {
        let mut hjd = 4.;
        let spb = 60. / bpm;

        let n2 = njs * spb;
        let mut n3 = n2 * hjd;
        while n3 > 17.999 {
            hjd /= 2.;
            n3 = n2 * hjd;
        }

        hjd += spawn_offset;
        if hjd < 0.25 {
            hjd = 0.25;
        }

        let jd = hjd * 2. * spb * njs;

        (hjd, jd)
    }

    pub fn beat_to_pos_simple(&self, current_beat: f32, target_beat: f32) -> f32 {
        let bps = self.bpm / 60.;
        let delta_b = target_beat - current_beat;
        let delta_s = delta_b / bps;
        delta_s * self.njs
    }
}

pub const HEAD_POS: Vec3 = Vec3::new(0., 1.62, 0.);

#[derive(Clone, Debug, PartialEq)]
pub struct LightColors {
    pub primary: Vec4,
    pub secondary: Vec4,
    pub white: Vec4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorScheme {
    pub left_note: Vec4,
    pub right_note: Vec4,
    pub obstacle: Vec4,
    pub lights: LightColors,
    pub boost: LightColors,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            left_note: hex!(#BF2F2F),
            right_note: hex!(#1F63A7),
            obstacle: hex!(#FF2F2F),
            lights: LightColors {
                primary: hex!(#BF2F2F),
                secondary: hex!(#1F63A7),
                white: hex!(#FFFFFF),
            },
            boost: LightColors {
                primary: hex!(#BF2F2F),
                secondary: hex!(#1F63A7),
                white: hex!(#FFFFFF),
            },
        }
    }
}



