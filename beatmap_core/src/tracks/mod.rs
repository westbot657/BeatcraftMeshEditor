//! This is my mini-implementation of tracks.
//! Note that a track point in this is NOT an event, but a state.
//! This means that loading from a file requires a bit of load time
//! to convert from beat saber/mod events to track points,
//! but this makes editing and playback much easier.

pub mod points;

use std::borrow::Cow;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use glam::FloatExt;
use tiny_vec::TinyVec;
use crate::RuntimeData;

pub trait Point: Clone + Default {
    type Output: Default;
    /// beat, new_beat, and runtime_data are provided in case time-based changes need to be made.
    fn extend(&self, beat: f32, new_beat: f32, runtime_data: &RuntimeData) -> Self {
        let _ = (beat, new_beat, runtime_data);
        self.clone()
    }
    fn interpolate(&self, other: &Self, span: BeatSpan, runtime_data: &RuntimeData) -> Self::Output;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Angle {
    Radians(f32),
    Degrees(f32),
}

impl Default for Angle {
    fn default() -> Self {
        Self::Radians(0.)
    }
}

impl Angle {
    pub fn to_radians(self) -> f32 {
        match self {
            Self::Radians(r) => r,
            Self::Degrees(r) => r.to_radians(),
        }
    }
    pub fn to_degrees(self) -> f32 {
        match self {
            Self::Radians(r) => r.to_degrees(),
            Self::Degrees(d) => d,
        }
    }
}

impl std::ops::Add for Angle {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Radians(r) => Self::Radians(r + rhs.to_radians()),
            Self::Degrees(r) => Self::Degrees(r + rhs.to_degrees()),
        }
    }
}

impl std::ops::Mul<f32> for Angle {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        match self {
            Self::Radians(r) => Self::Radians(r * rhs),
            Self::Degrees(d) => Self::Degrees(d * rhs),
        }
    }
}

impl std::ops::Mul<Angle> for f32 {
    type Output = Angle;
    fn mul(self, rhs: Angle) -> Self::Output {
        rhs * self
    }
}

impl std::ops::Sub for Angle {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Self::Radians(r) => Self::Radians(r - rhs.to_radians()),
            Self::Degrees(r) => Self::Degrees(r - rhs.to_degrees()),
        }
    }
}

impl std::ops::Neg for Angle {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Radians(r) => Self::Radians(-r),
            Self::Degrees(d) => Self::Degrees(-d),
        }
    }
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

    pub fn new(now: f32, now_seconds: f32, start: f32, end: f32) -> Self {
        Self {
            start,
            end,
            now,
            start_seconds: None,
            end_seconds: None,
            now_seconds,
        }
    }

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

    fn _animate(
        points: &[TimedPoint<P>],
        start: &TimedPoint<P>,
        last_beat: f32,
        beat: f32,
        second: f32,
        rt: &RuntimeData
    ) -> P::Output {
        let i = points.partition_point(|p| p.beat <= beat);

        let a = match i.checked_sub(1) {
            Some(j) => &points[j],
            None => start,
        };

        let extended;
        let b = match points.get(i) {
            Some(p) => p,
            None => {
                extended = TimedPoint {
                    point: a.point.extend(a.beat, last_beat, rt),
                    beat: last_beat,
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

    pub fn animate(&self, beat: f32, second: f32, rt: &RuntimeData) -> P::Output {
        Self::_animate(&self.points, &self.start, self.last_beat, beat, second, rt)
    }

    pub fn points(&self) -> &[TimedPoint<P>] {
        &self.points
    }

    /// This will return Err if the point's beat is outside the song length.
    pub fn insert(&mut self, beat: f32, point: P) -> Result<(), TrackInsertionError> {
        if !(0f32..=self.last_beat).contains(&beat) {
            return Err(TrackInsertionError { bound: self.last_beat, got: beat });
        }

        let i = self.points.partition_point(|p| p.beat <= beat);

        self.points.insert(i, TimedPoint { point, beat });

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

    pub fn iter_animate<R>(&self, beat: f32, second: f32, runtime_data: &RuntimeData, expected_elements: usize, frame: &mut Vec<R>, transformer: impl Fn(P::Output) -> R) {
        for i in 0..expected_elements {
            let val = self.animate(i, beat, second, runtime_data).unwrap_or_default();
            frame.push(transformer(val));
        }
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &Track<P>> {
        self.tracks.iter()
    }

    pub fn animate(&self, object_id: usize, beat: f32, second: f32, runtime_data: &RuntimeData) -> Option<P::Output> {
        let track = self.tracks.get(object_id)?;
        Some(track.animate(beat, second, runtime_data))
    }

    pub fn insert(&mut self, object_id: usize, beat: f32, point: P) -> Result<(), TrackSetInsertionError> {
        let l = self.tracks.len()-1;
        let track = self.tracks.get_mut(object_id).ok_or_else(|| TrackSetInsertionError::InvalidId { max: l, id: object_id })?;
        Ok(track.insert(beat, point)?)
    }

    pub fn points(&self, object_id: usize) -> Option<&[TimedPoint<P>]> {
        let track = self.tracks.get(object_id)?;
        Some(track.points())
    }

}

impl<P: Point> UninitializedTrackSet<P> {
    pub fn add_point(&mut self, object_id: usize, beat: f32, point: P) -> Result<(), TrackSetInsertionError> {
        let l = self.tracks.len()-1;
        let track = self.tracks.get_mut(object_id).ok_or_else(|| TrackSetInsertionError::InvalidId { max: l, id: object_id })?;
        if !(0f32..=self.song_length).contains(&beat) {
            return Err(TrackSetInsertionError::OutOfBounds(TrackInsertionError { bound: self.song_length, got: beat }));
        }
        track.push(TimedPoint { point, beat });
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

/// For events that require multiple points to animate.
/// since get_points returns a `Cow<'_, [TimedPoint<P>]>`,
/// it is valid to store a small list of points in the event
/// to avoid expensive recomputation each frame.
pub trait Event: Default + Clone {
    type Point: Point;

    fn extend(&self, beat: f32, new_beat: f32, runtime_data: &RuntimeData) -> Self {
        let _ = (beat, new_beat, runtime_data);
        self.clone()
    }

    /// Return all points that this event turns into.
    /// EventTrack handles trimming so that points from this event don't appear after the next event's points
    /// the returned points are assumed to be sorted, and is not checked.
    fn get_points<'e>(
        &'e self,
        event_beat: f32,
        previous: &TimedEvent<Self>,
        next: Option<&TimedEvent<Self>>,
        rt: &RuntimeData,
    ) -> Cow<'e, [TimedPoint<Self::Point>]>;

    /// When the points from this event extend past the start of the next event's points,
    /// this function is intended to handle any changes needed (such as interpolating a new
    /// endpoint at the cutoff beat)
    /// by default this does nothing
    fn clip<'e>(
        points: Cow<'e, [TimedPoint<Self::Point>]>,
        cutoff_beat: f32,
        rt: &RuntimeData,
    ) -> Cow<'e, [TimedPoint<Self::Point>]> {
        let _ = (cutoff_beat, rt);
        points
    }

}

#[derive(Default, Clone)]
pub struct TimedEvent<E: Event> {
    pub event: E,
    beat: f32,
}

impl<E: Event> TimedEvent<E> {
    pub fn beat(&self) -> f32 {
        self.beat
    }
}

/// This track type is for more complex events that require multiple
/// points to statelessly animate.
pub struct EventTrack<E: Event> {
    events: Vec<TimedEvent<E>>,
    start: TimedEvent<E>,
    last_beat: f32,
}

impl<E: Event> EventTrack<E> {
    pub fn empty(last_beat: f32) -> Self {
        Self::from_event_vec(Vec::new(), last_beat)
    }
    pub fn from_event_vec(mut events: Vec<TimedEvent<E>>, last_beat: f32) -> Self {
        events.sort_by(|a, b| a.beat.total_cmp(&b.beat));
        #[cfg(feature = "tracing")]
        let l = events.len();
        events.retain(|e| (0f32..=last_beat).contains(&e.beat));
        #[cfg(feature = "tracing")]
        if l != events.len() {
            let diff = l - events.len();
            let s = if diff == 1 { "" } else { "s" };
            tracing::warn!("truncated {diff} event{s} outside the song");
        }

        Self {
            events,
            start: Default::default(),
            last_beat,
        }
    }

    pub fn animate(&self, beat: f32, second: f32, rt: &RuntimeData) -> <E::Point as Point>::Output {
        let i = self.events.partition_point(|e| e.beat <= beat);

        let a = match i.checked_sub(2) {
            Some(j) => &self.events[j],
            None => &self.start,
        };

        let b = i.checked_sub(1)
            .map(|j| &self.events[j]);

        let ab = b.unwrap_or(a);

        let extended;
        let c = match self.events.get(i) {
            Some(e) => e,
            None => {
                extended = TimedEvent {
                    event: ab.event.extend(ab.beat, self.last_beat, rt),
                    beat: self.last_beat,
                };
                &extended
            }
        };

        let d = self.events.get(i+1);

        let abc = b.map(|e| {
            let abc = e.event.get_points(e.beat, a, Some(c), rt);
            debug_assert!(
                abc.windows(2)
                    .all(|w| w[0].beat <= w[1].beat),
                "get_points returned unordered points, {:?}", std::any::type_name::<E>()
            );
            abc
        });

        let bcd = c.event.get_points(c.beat, ab, d, rt);
        debug_assert!(
            bcd.windows(2)
                .all(|w| w[0].beat <= w[1].beat),
            "get_points returned unordered points, {:?}", std::any::type_name::<E>(),
        );

        let cutoff = if let Some(p0) = bcd.first() {
            p0.beat
        } else {
            #[cfg(feature = "tracing")]
            tracing::error!("Event didn't return any points (at beat {beat})");
            let def = E::Point::default();
            let end = def.extend(0., self.last_beat, rt);
            let span = BeatSpan::new(beat, second, 0., self.last_beat);
            return def.interpolate(&end, span, rt)
        };

        let max_count = bcd.len() + abc.as_ref()
            .map(|p| p.len())
            .unwrap_or_default();

        let mut points: TinyVec<_, 6> = TinyVec::with_capacity(max_count);

        if let Some(abc) = abc {
            let abc = E::clip(abc, cutoff, rt);
            match abc {
                Cow::Owned(vec) => {
                    for p in vec {
                        if p.beat <= cutoff {
                            points.push(p);
                        }
                    }
                }
                Cow::Borrowed(slice) => {
                    for p in slice {
                        if p.beat <= cutoff {
                            points.push(p.clone())
                        }
                    }
                }
            }
        }

        for p in bcd.into_owned().into_iter() {
            points.push(p);
        }

        Track::_animate(
            &points,
            &Default::default(),
            self.last_beat,
            beat,
            second,
            rt
        )
    }

    pub fn events(&self) -> &[TimedEvent<E>] {
        &self.events
    }

    pub fn insert(&mut self, beat: f32, event: E) -> Result<(), TrackInsertionError> {
        if !(0f32..=self.last_beat).contains(&beat) {
            return Err(TrackInsertionError { bound: self.last_beat, got: beat });
        }

        let i = self.events.partition_point(|p| p.beat <= beat);

        self.events.insert(i, TimedEvent { event, beat });

        Ok(())
    }

}

pub struct UninitializedEventTrackSet<E: Event> {
    tracks: Vec<Vec<TimedEvent<E>>>,
    song_length: f32,
}

pub struct EventTrackSet<E: Event> {
    tracks: Vec<EventTrack<E>>,
}

impl<E: Event> EventTrackSet<E> {

    pub fn new(object_count: NonZeroUsize, song_length: f32) -> UninitializedEventTrackSet<E> {
        UninitializedEventTrackSet {
            tracks: vec![Vec::new(); object_count.get()],
            song_length,
        }
    }

    pub fn iter_animate<R>(&self, beat: f32, second: f32, runtime_data: &RuntimeData, expected_elements: usize, frame: &mut Vec<R>, transformer: impl Fn(<E::Point as Point>::Output) -> R) {
        for i in 0..expected_elements {
            let val = self.animate(i, beat, second, runtime_data).unwrap_or_default();
            frame.push(transformer(val));
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &EventTrack<E>> {
        self.tracks.iter()
    }

    pub fn animate(&self, object_id: usize, beat: f32, second: f32, runtime_data: &RuntimeData) -> Option<<E::Point as Point>::Output> {
        let track = self.tracks.get(object_id)?;
        Some(track.animate(beat, second, runtime_data))
    }

    pub fn insert(&mut self, object_id: usize, beat: f32, event: E) -> Result<(), TrackSetInsertionError> {
        let l = self.tracks.len()-1;
        let track = self.tracks.get_mut(object_id).ok_or_else(|| TrackSetInsertionError::InvalidId { max: l, id: object_id })?;
        Ok(track.insert(beat, event)?)
    }

    pub fn events(&self, object_id: usize) -> Option<&[TimedEvent<E>]> {
        let track = self.tracks.get(object_id)?;
        Some(&track.events)
    }

}

impl<E: Event> UninitializedEventTrackSet<E> {
    pub fn add_point(&mut self, object_id: usize, beat: f32, event: E) -> Result<(), TrackSetInsertionError> {
        let l = self.tracks.len()-1;
        let track = self.tracks.get_mut(object_id).ok_or_else(|| TrackSetInsertionError::InvalidId { max: l, id: object_id })?;
        if !(0f32..=self.song_length).contains(&beat) {
            return Err(TrackSetInsertionError::OutOfBounds(TrackInsertionError { bound: self.song_length, got: beat }));
        }
        track.push(TimedEvent { event, beat });
        Ok(())
    }

    pub fn finalize(self) -> EventTrackSet<E> {
        EventTrackSet {
            tracks: self.tracks
                .into_iter()
                .map(|t| EventTrack::from_event_vec(t, self.song_length))
                .collect()
        }
    }

}

