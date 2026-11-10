//! This is my mini-implementation of tracks.
//! Note that a track point in this is NOT an event, but a state.
//! This means that loading from a file requires a bit of load time
//! to convert from beat saber/mod events to track points,
//! but this makes editing and playback much easier.

pub mod points;

use std::fmt::Debug;
use std::num::NonZeroUsize;
use glam::FloatExt;
use crate::RuntimeData;

pub trait Point: Clone + Default {
    type Output;
    /// beat, new_beat, and runtime_data are provided in case time-based changes need to be made.
    fn extend(&self, beat: f32, new_beat: f32, runtime_data: &RuntimeData) -> Self {
        let _ = (beat, new_beat, runtime_data);
        self.clone()
    }
    fn interpolate(&self, other: &Self, span: BeatSpan, runtime_data: &RuntimeData) -> Self::Output;
}

#[derive(Copy, Clone, Debug)]
pub struct BeatSpan {
    pub start: f32,
    pub end: f32,
    pub now: f32,
    start_seconds: Option<f32>,
    end_seconds: Option<f32>,
    pub now_seconds: f32,
}

impl BeatSpan {

    pub fn get_start_seconds(&mut self, rt: &RuntimeData) -> f32 {
        *self.start_seconds.get_or_insert_with(|| rt.beat_to_seconds(self.start))
    }

    pub fn get_end_seconds(&mut self, rt: &RuntimeData) -> f32 {
        *self.end_seconds.get_or_insert_with(|| rt.beat_to_seconds(self.end))
    }

    pub fn inverse_lerp_beats(&self) -> f32 {
        if self.end <= self.start { return 1.0; }
        f32::inverse_lerp(self.start, self.end, self.now).clamp(0., 1.)
    }
    pub fn inverse_lerp_seconds(&mut self, rt: &RuntimeData) -> f32 {
        let s = self.get_start_seconds(rt);
        let e = self.get_end_seconds(rt);
        if e <= s { return 1.0; }
        f32::inverse_lerp(
            s,
            e,
            self.now_seconds,
        ).clamp(0., 1.)
    }
    pub fn elapsed_beats(&self) -> f32 {
        self.now - self.start
    }
    pub fn elapsed_seconds(&mut self, rt: &RuntimeData) -> f32 {
        self.now_seconds - self.get_start_seconds(rt)
    }
}

#[derive(Clone, Default)]
pub struct TimedPoint<P: Point> {
    pub point: P,
    /// beat is private so that the user can't modify it when holding a &mut TimedPoint<P>
    beat: f32,
}

#[derive(Default, Clone)]
pub struct Track<P: Point> {
    points: Vec<TimedPoint<P>>,
    start: TimedPoint<P>,
    last_beat: f32,
}

#[derive(thiserror::Error, Debug)]
#[error("insertion beat {got} is out of bounds 0..={bound}")]
pub struct TrackInsertionError {
    bound: f32,
    got: f32,
}

impl<P: Point> Track<P> {
    pub fn empty(last_beat: f32) -> Self {
        Self::from_point_vec(Vec::new(), last_beat)
    }
    pub fn from_point_vec(mut points: Vec<TimedPoint<P>>, last_beat: f32) -> Self {
        points.sort_by(|a, b| a.beat.total_cmp(&b.beat));
        #[cfg(feature = "tracing")]
        let l = points.len();
        points.retain(|p| (0f32..=last_beat).contains(&p.beat) );
        #[cfg(feature = "tracing")]
        if l != points.len() {
            let diff = l - points.len();
            let s = if diff == 1 { "" } else { "s" };
            tracing::warn!("truncated {diff} point{s} outside the song");
        }

        Self { points, start: Default::default(), last_beat }
    }

    pub fn animate(&self, beat: f32, second: f32, rt: &RuntimeData) -> P::Output {
        let i = self.points.partition_point(|p| p.beat <= beat);

        let a = match i.checked_sub(1) {
            Some(j) => &self.points[j],
            None => &self.start,
        };

        let extended;
        let b = match self.points.get(i) {
            Some(p) => p,
            None => {
                extended = TimedPoint {
                    point: a.point.extend(a.beat, self.last_beat, rt),
                    beat: self.last_beat,
                };
                &extended
            }
        };

        a.point.interpolate(
            &b.point,
            BeatSpan { start: a.beat, end: b.beat, now: beat, start_seconds: None, end_seconds: None, now_seconds: second },
            rt,
        )
    }

    pub fn points(&self) -> &[TimedPoint<P>] {
        &self.points
    }

    /// This will return Err if the point's beat is outside the song length.
    pub fn insert(&mut self, point: TimedPoint<P>) -> Result<(), TrackInsertionError> {
        if !(0f32..=self.last_beat).contains(&point.beat) {
            return Err(TrackInsertionError { bound: self.last_beat, got: point.beat });
        }

        let i = self.points.partition_point(|p| p.beat <= point.beat);

        self.points.insert(i, point);

        Ok(())
    }
}

pub struct UninitializedTrackSet<P: Point> {
    tracks: Vec<Vec<TimedPoint<P>>>,
    song_length: f32,
}

pub struct TrackSet<P: Point> {
    tracks: Vec<Track<P>>
}

#[derive(thiserror::Error, Debug)]
pub enum TrackSetInsertionError {
    #[error("{0}")]
    OutOfBounds(#[from] TrackInsertionError),
    #[error("invalid object id: {id} expected 0..={max}")]
    InvalidId { max: usize, id: usize },
}

impl<P: Point> TrackSet<P> {

    pub fn new(object_count: NonZeroUsize, song_length: f32) -> UninitializedTrackSet<P> {
        UninitializedTrackSet {
            tracks: vec![Vec::new(); object_count.get()],
            song_length,
        }
    }

    pub fn animate(&self, object_id: usize, beat: f32, second: f32, runtime_data: &RuntimeData) -> Option<P::Output> {
        let track = self.tracks.get(object_id)?;
        Some(track.animate(beat, second, runtime_data))
    }

    pub fn insert(&mut self, object_id: usize, point: TimedPoint<P>) -> Result<(), TrackSetInsertionError> {
        let l = self.tracks.len()-1;
        let track = self.tracks.get_mut(object_id).ok_or_else(|| TrackSetInsertionError::InvalidId { max: l, id: object_id })?;
        Ok(track.insert(point)?)
    }

    pub fn points(&self, object_id: usize) -> Option<&[TimedPoint<P>]> {
        let track = self.tracks.get(object_id)?;
        Some(track.points())
    }

}

impl<P: Point> UninitializedTrackSet<P> {
    pub fn add_point(&mut self, object_id: usize, point: TimedPoint<P>) -> Result<(), TrackSetInsertionError> {
        let l = self.tracks.len()-1;
        let track = self.tracks.get_mut(object_id).ok_or_else(|| TrackSetInsertionError::InvalidId { max: l, id: object_id })?;
        if !(0f32..=self.song_length).contains(&point.beat) {
            return Err(TrackSetInsertionError::OutOfBounds(TrackInsertionError { bound: self.song_length, got: point.beat }));
        }
        track.push(point);
        Ok(())
    }

    pub fn finalize(self) -> TrackSet<P> {
        TrackSet {
            tracks: self.tracks
                .into_iter()
                .map(|t| Track::from_point_vec(t, self.song_length))
                .collect()
        }
    }
}

