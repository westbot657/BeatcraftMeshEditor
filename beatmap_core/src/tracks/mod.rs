//! This is my mini-implementation of tracks.
//! Note that a track point in this is NOT an event, but a state.
//! This means that loading from a file requires a bit of load time
//! to convert from beat saber/mod events to track points,
//! but this makes editing and playback much easier.

pub mod points;

use std::borrow::Cow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Index;
use std::slice::SliceIndex;
use glam::FloatExt;
use tiny_vec::TinyVec;
use crate::RuntimeData;

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

/// For events that require multiple points to animate.
/// since get_points returns a `Cow<'_, [TimedPoint<P>]>`,
/// it is valid to store a small list of points in the event
/// to avoid expensive recomputation each frame.
pub trait Event: Default + Clone + TimedData {
    type Point: TimedData<State = Self::State>;

    /// Return all points that this event turns into.
    /// EventTrack handles trimming so that points from this event don't appear after the next event's points
    /// the returned points are assumed to be sorted, and is not checked.
    fn get_points<'e>(
        &'e self,
        event_beat: f32,
        previous: &Timed<Self>,
        next: Option<&Timed<Self>>,
        rt: &RuntimeData,
    ) -> TinyVec<(f32, Cow<'e, Self::Point>), 4>;

    /// When the points from this event extend past the start of the next event's points,
    /// this function is intended to handle any changes needed (such as interpolating a new
    /// endpoint at the cutoff beat)
    /// by default this does nothing
    fn clip<'e>(
        points: TinyVec<(f32, Cow<'e, Self::Point>), 4>,
        cutoff_beat: f32,
        rt: &RuntimeData,
    ) -> TinyVec<(f32, Cow<'e, Self::Point>), 4> {
        let _ = (cutoff_beat, rt);
        points
    }

}

#[derive(thiserror::Error, Debug)]
#[error("insertion beat {got} is out of bounds 0..={bound}")]
pub struct TrackInsertionError {
    pub bound: f32,
    pub got: f32,
}

#[derive(thiserror::Error, Debug)]
pub enum TrackSetInsertionError {
    #[error("{0}")]
    OutOfBounds(#[from] TrackInsertionError),
    #[error("invalid object id: {id} expected 0..{max}")]
    InvalidId { max: usize, id: usize },
}

pub trait Animator<T: TimedData> {
    fn animate(
        default: &Timed<T>,
        track: &[Timed<T>],
        last_beat: f32,
        beat: f32,
        second: f32,
        rt: &RuntimeData,
    ) -> T::State;
}

pub trait TimedData: Clone + Default {

    type Animator: Animator<Self>;

    type State: Clone + Default;

    fn extend(
        &self,
        beat: f32,
        new_beat: f32,
        runtime_data: &RuntimeData
    ) -> Self {
        let _ = (beat, new_beat, runtime_data);
        self.clone()
    }

    fn interpolate(
        &self,
        other: &Self,
        span: BeatSpan,
        runtime_data: &RuntimeData
    ) -> Self::State;

}

pub struct PointAnimator<P: TimedData>(PhantomData<P>);
pub struct EventAnimator<E: TimedData>(PhantomData<E>);

impl<P: TimedData> Animator<P> for PointAnimator<P> {
    fn animate(default: &Timed<P>, data: &[Timed<P>], last_beat: f32, beat: f32, second: f32, rt: &RuntimeData) -> P::State {
        animate_timed(
            default,
            data,
            last_beat,
            beat,
            second,
            rt
        )
    }
}

impl<E: Event + TimedData> Animator<E> for EventAnimator<E>
where
    E::Point: TimedData
{
    fn animate(default: &Timed<E>, data: &[Timed<E>], last_beat: f32, beat: f32, second: f32, rt: &RuntimeData) -> E::State {
        let i = data.partition_point(|e| e.beat <= beat);

        let a = match i.checked_sub(2) {
            Some(j) => &data[j],
            None => default,
        };

        let b = i.checked_sub(1)
            .map(|j| &data[j]);

        let ab = b.unwrap_or(a);

        let extended;
        let c = match data.get(i) {
            Some(e) => e,
            None => {
                extended = Timed {
                    data: ab.data.extend(ab.beat, last_beat, rt),
                    beat: last_beat,
                };
                &extended
            }
        };

        let d = data.get(i+1);

        let abc = b.map(|e| {
            let abc = e.data.get_points(e.beat, a, Some(c), rt);
            debug_assert!(
                abc.windows(2)
                    .all(|w| w[0].0 <= w[1].0),
                "get_points returned unordered points, {:?}", std::any::type_name::<E>()
            );
            abc
        });

        let bcd = c.data.get_points(c.beat, ab, d, rt);
        debug_assert!(
            bcd.windows(2)
                .all(|w| w[0].0 <= w[1].0),
            "get_points returned unordered points, {:?}", std::any::type_name::<E>(),
        );

        let cutoff = if let Some(p0) = bcd.first() {
            p0.0
        } else {
            #[cfg(feature = "tracing")]
            tracing::error!("Event didn't return any points (at beat {beat})");
            let def = E::Point::default();
            let end = def.extend(0., last_beat, rt);
            let span = BeatSpan::new(beat, second, 0., last_beat);
            return def.interpolate(&end, span, rt)
        };

        let max_count = bcd.len() + abc.as_ref()
            .map(|p| p.len())
            .unwrap_or_default();

        let mut points: TinyVec<_, 6> = TinyVec::with_capacity(max_count);

        if let Some(abc) = abc {
            let abc = E::clip(abc, cutoff, rt);
            for (beat, data) in abc {
                if beat <= cutoff {
                    points.push(Timed { beat, data: data.into_owned() })
                }
            }
        }

        for (beat, data) in bcd {
            points.push(Timed { beat, data: data.into_owned() });
        }

        animate_timed(
            &Timed { data: Default::default(), beat: 0. },
            &points,
            last_beat,
            beat,
            second,
            rt
        )
    }
}

/// This struct has no public constructor so that holding a &mut Timed<T>
/// is safe from unwanted beat modifications
pub struct Timed<T: TimedData> {
    pub data: T,
    beat: f32,
}

pub struct Track<T: TimedData> {
    data: Vec<Timed<T>>,
    /// Holds a cached default element for interpolating when the data list
    /// is empty or is animating before it's first element
    default: Timed<T>,
    last_beat: f32,
}

pub struct TrackSet<T: TimedData> {
    tracks: Vec<Vec<Timed<T>>>,
    /// Holds a cached default element for interpolating when the data list
    /// is empty or is animating before it's first element
    default: Timed<T>,
    last_beat: f32,
}

pub struct UninitializedTrackSet<T: TimedData> {
    tracks: Vec<Vec<Timed<T>>>,
    last_beat: f32,
}

pub struct DataView<'d, T>(&'d mut [T]);

impl<T: TimedData> Timed<T> {
    #[inline]
    pub fn beat(&self) -> f32 {
        self.beat
    }
}

impl<'d, T> DataView<'d, T> {
    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.0.get_mut(index)
    }
}

pub fn animate_timed<T: TimedData>(
    default: &Timed<T>,
    data: &[Timed<T>],
    last_beat: f32,
    beat: f32,
    second: f32,
    rt: &RuntimeData,
) -> T::State {
    let i = data.partition_point(|p| p.beat <= beat);

    let a = match i.checked_sub(1) {
        Some(j) => &data[j],
        None => default,
    };

    let extended;
    let b = match data.get(i) {
        Some(p) => p,
        None => {
            extended = Timed {
                data: a.data.extend(a.beat, last_beat, rt),
                beat: last_beat,
            };
            &extended
        }
    };

    a.data.interpolate(
        &b.data,
        BeatSpan {
            start: a.beat,
            end: b.beat,
            now: beat,
            start_seconds: None,
            end_seconds: None,
            now_seconds: second
        },
        rt,
    )
}

fn clip_vec<T: TimedData>(data: &mut Vec<Timed<T>>, last_beat: f32) {
    #[cfg(feature = "tracing")]
    let l = data.len();

    data.retain(|t| (0f32..=last_beat).contains(&t.beat));

    #[cfg(feature = "tracing")]
    if l != data.len() {
        let diff = l - data.len();
        let s = if diff == 1 { "" } else { "s" };
        tracing::warn!("truncated {diff} element{s} outside the song bounds");
    }
}

fn track_insert<T: TimedData>(list: &mut Vec<Timed<T>>, last_beat: f32, beat: f32, data: T) -> Result<(), TrackInsertionError> {
    if (0f32..=last_beat).contains(&beat) {

        let i = list
            .partition_point(|p| p.beat <= beat);

        list.insert(i, Timed { data, beat });

        Ok(())
    } else {
        Err(TrackInsertionError {
            bound: last_beat,
            got: beat,
        })
    }
}

impl<T: TimedData> Track<T> {

    pub fn empty(last_beat: f32) -> Self {
        Self::from_vec(Vec::new(), last_beat)
    }

    pub fn from_vec(mut data: Vec<Timed<T>>, last_beat: f32) -> Self {
        data.sort_by(|a, b| a.beat.total_cmp(&b.beat));
        clip_vec(&mut data, last_beat);
        Self { data, default: Timed { data: Default::default(), beat: 0. }, last_beat }
    }

    pub fn insert(&mut self, beat: f32, data: T) -> Result<(), TrackInsertionError> {
        track_insert(&mut self.data, self.last_beat, beat, data)
    }

    pub fn animate(&self, beat: f32, second: f32, rt: &RuntimeData) -> T::State {
        T::Animator::animate(&self.default, &self.data, self.last_beat, beat, second, rt)
    }

    pub fn data(&self) -> &[Timed<T>] {
        &self.data
    }

    pub fn data_mut(&mut self) -> DataView<'_, Timed<T>> {
        DataView(self.data.as_mut_slice())
    }

}



impl<T: TimedData> TrackSet<T> {
    pub fn empty(tracks: usize, last_beat: f32) -> Self {
        let mut tracks2 = Vec::with_capacity(tracks);
        for _ in 0..tracks {
            tracks2.push(Vec::new());
        }
        Self { tracks: tracks2, default: Timed { data: Default::default(), beat: 0. }, last_beat }
    }

    pub fn new(tracks: usize, last_beat: f32) -> UninitializedTrackSet<T> {
        let mut tracks2 = Vec::with_capacity(tracks);
        for _ in 0..tracks {
            tracks2.push(Vec::new());
        }
        UninitializedTrackSet {
            tracks: tracks2,
            last_beat
        }
    }

    pub fn from_vecs(mut tracks: Vec<Vec<Timed<T>>>, last_beat: f32) -> Self {
        for track in tracks.iter_mut() {
            track.sort_by(|a, b| a.beat.total_cmp(&b.beat));
            clip_vec(track, last_beat);
        }
        Self { tracks, default: Timed { data: Default::default(), beat: 0. }, last_beat }
    }

    pub fn insert(&mut self, track: usize, beat: f32, data: T) -> Result<(), TrackSetInsertionError> {
        if let Some(track) = self.tracks.get_mut(track) {
            Ok(track_insert(track, self.last_beat, beat, data)?)
        } else {
            Err(TrackSetInsertionError::InvalidId { max: self.tracks.len(), id: track })
        }
    }

    /// Note: if the track set has fewer tracks than the frame expects, this creates a default
    /// data value to place in the list for each missing track.
    pub fn iter_animate<R>(
        &self,
        beat: f32,
        second: f32,
        rt: &RuntimeData,
        track_count: usize,
        frame: &mut Vec<R>,
        transformer: impl Fn(T::State) -> R,
    ) {
        for i in 0..track_count {
            let state = match self.tracks.get(i) {
                Some(track) => T::Animator::animate(&self.default, track, self.last_beat, beat, second, rt),
                None => T::State::default(),
            };
            frame.push(transformer(state));
        }

    }

    /// `frame` should hold uninitialized R elements.
    /// in the case that the track set has fewer
    /// tracks than the frame, it's the caller's responsibility to remove, or finish initializing
    /// the elements that this function didn't touch.
    /// this function therefore returns N as the number of values it wrote (0<=x<=frame.len())
    pub fn direct_animate<R>(
        &self,
        beat: f32,
        second: f32,
        rt: &RuntimeData,
        frame: &mut [MaybeUninit<R>],
        transformer: impl Fn(T::State) -> R,
    ) -> usize {
        for i in 0..frame.len() {
            if i == self.tracks.len() {
                return i;
            }
            let track = &self.tracks[i];
            let state = T::Animator::animate(
                &self.default,
                track,
                self.last_beat,
                beat,
                second,
                rt
            );
            frame[i].write(transformer(state));
        }
        frame.len()
    }

    pub fn data(&self, track: usize) -> Option<&[Timed<T>]> {
        Some(self.tracks.get(track)?)
    }

}

impl<T: TimedData> UninitializedTrackSet<T> {

    pub fn add(&mut self, track: usize, beat: f32, data: T) -> Result<(), TrackSetInsertionError> {
        if let Some(track) = self.tracks.get_mut(track) {
            if (0f32..=self.last_beat).contains(&beat) {
                track.push(Timed { data, beat });
                Ok(())
            } else {
                Err(TrackSetInsertionError::OutOfBounds(TrackInsertionError {
                    bound: self.last_beat,
                    got: beat,
                }))
            }
        } else {
            Err(TrackSetInsertionError::InvalidId {
                max: self.tracks.len(),
                id: track,
            })
        }
    }

    pub fn finalize(self) -> TrackSet<T> {
        TrackSet::from_vecs(self.tracks, self.last_beat)
    }
}

