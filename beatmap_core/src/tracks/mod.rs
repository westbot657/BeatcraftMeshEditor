//! This is my mini-implementation of tracks.
//! Note that a track point in this is NOT an event, but a state.
//! This means that loading from a file requires a bit of load time
//! to convert from beat saber/mod events to track points,
//! but this makes editing and playback much easier.

pub mod points;

use crate::RuntimeData;
use glam::FloatExt;
use std::borrow::Cow;
use std::cell::OnceCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use tiny_vec::TinyVec;

// Errors
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


// Helpers
/// Handles base interpolation animation between points.
pub(crate) fn animate_timed<P: Point, T: Timeable<Data =P>>(
    default: &T,
    data: &[T],
    last_beat: f32,
    now: Now,
    rt: &RuntimeData,
) -> P::State {
    let i = data.partition_point(|p| p.beat() <= now.beat);

    let a = match i.checked_sub(1) {
        Some(j) => &data[j],
        None => default,
    };

    let extended;
    let b = match data.get(i) {
        Some(p) => p,
        None => {
            extended = T::new(a.data().extend(a.beat(), last_beat, rt), last_beat);
            &extended
        }
    };

    a.data()
        .interpolate(b.data(), TimeSpan::new(now, a.beat(), b.beat()), rt)
}

/// invalidates `i`+1. `i` is left alone since it's assumed to get invalidated by
/// whatever operation caused this to be called in the first place.
fn invalidate_neighbors<T: Timeable>(list: &mut [T], i: usize) {
    if let Some(p) = list.get_mut(i.saturating_add(1)) {
        p.invalidate_cache();
    }
}

/// Clips any Data outside the song range and logs warnings.
fn clip_vec<D: TimedData, T: Timeable<Data = D>>(data: &mut Vec<T>, last_beat: f32) {
    #[cfg(feature = "tracing")]
    let l = data.len();

    data.retain(|t| (0f32..=last_beat).contains(&t.beat()));

    #[cfg(feature = "tracing")]
    if l != data.len() {
        let diff = l - data.len();
        let s = if diff == 1 { "" } else { "s" };
        tracing::warn!("truncated {diff} element{s} outside the song bounds");
    }
}

/// Handles insertion logic for both Track and TrackSet
fn track_insert<D: TimedData, T: Timeable<Data = D>>(
    list: &mut Vec<T>,
    last_beat: f32,
    beat: f32,
    data: D,
) -> Result<(), TrackInsertionError> {
    if (0f32..=last_beat).contains(&beat) {
        let i = list.partition_point(|p| p.beat() <= beat);

        list.insert(i, T::new(data, beat));
        invalidate_neighbors(list.as_mut_slice(), i);

        Ok(())
    } else {
        Err(TrackInsertionError {
            bound: last_beat,
            got: beat,
        })
    }
}


// Traits
/// Represents an Animator over data.
pub trait Animator<D: TimedData, T: Timeable<Data = D>> {
    fn animate(
        default: &T,
        track: &[T],
        last_beat: f32,
        now: Now,
        rt: &RuntimeData,
    ) -> D::State;
}

/// Represents a type that sits at a point in time.
/// this trait determines what Animator is needed, and what the output state is.
pub trait TimedData: Clone + Default {
    /// Which animator is needed to handle this data;
    /// EventAnimator is for Events.
    /// PointAnimator is for Points.
    type Animator<T: Timeable<Data = Self>>: Animator<Self, T>;

    /// The output state of the data interpolation.
    /// i.e. F32Point's State is f32, and an Event that produces F32Points also has an f32 State.
    type State: Clone + Default;
}

/// Points are interpolated to determine the State at the given time.
/// Points can read the point directly ahead of itself to determine it's output state.
pub trait Point: TimedData {
    /// interpolates between this point and the next point.
    fn interpolate(&self, next: &Self, span: TimeSpan, runtime_data: &RuntimeData) -> Self::State;

    /// When animating at a time after the last point in the list, this is called on the last point
    /// to determine the stand-in point.
    /// By default, Clone is used, though this may not be desired in all cases.
    fn extend(&self, beat: f32, new_beat: f32, runtime_data: &RuntimeData) -> Self {
        let _ = (beat, new_beat, runtime_data);
        self.clone()
    }
}

/// Events are queried for Points that are then used for interpolation.
/// during animation, up to 3 events are sampled, and up to 2 are queried for Points.
/// Given the Events:
/// ```ignore
/// [A     B   | C]
///        ... | ...
/// ```
/// where `|` is the current time,
/// `B` is asked for point values, and given `A` as reference,
/// `C` is then asked for more point values, given `B` as reference.
/// Any points given by `B` that are placed after the first point given by `C` are discarded after
/// all of `B`'s points are handed to `Event::clip`, where interrupted interpolations can
/// have a new endpoint inserted at the cutoff time.
pub trait Event: TimedData {
    /// The Point type that this Event produces for interpolations.
    type Point: Point + TimedData<State = Self::State>;

    /// Return all points that this event turns into.
    /// Track<E: Event> handles trimming so that points from
    /// this event don't appear after the next event's points
    /// the returned points are assumed to be sorted, and is not checked.
    fn get_points<'e, T: Timeable<Data = Self>>(
        &'e self,
        event_beat: f32,
        previous: &T,
        rt: &RuntimeData,
    ) -> TinyVec<(f32, Cow<'e, Self::Point>), 4>;

    /// When the points from this event extend past the start of the next event's points,
    /// this function is intended to handle any changes needed (such as interpolating a new
    /// endpoint at or before the cutoff beat).
    /// By default, this does nothing
    fn clip<'e>(
        points: TinyVec<(f32, Cow<'e, Self::Point>), 4>,
        cutoff_beat: f32,
        rt: &RuntimeData,
    ) -> TinyVec<(f32, Cow<'e, Self::Point>), 4> {
        let _ = (cutoff_beat, rt);
        points
    }
}

pub(crate) trait PrivateTimeable<D: TimedData> {
    /// new is private.
    fn new(data: D, beat: f32) -> Self;
    /// take_data is private.
    fn take_data(self) -> D;
}

/// Represents both Timed<D> and CacheableTimed<D>
#[allow(private_bounds)]
pub trait Timeable: PrivateTimeable<Self::Data> {
    type Data: TimedData;
    fn beat(&self) -> f32;
    fn data(&self) -> &<Self as Timeable>::Data;
    fn data_mut(&mut self) -> &mut <Self as Timeable>::Data;

    /// for Events, this is used to get a Point list to be used for interpolations
    fn get_points<'e>(
        &'e self,
        previous: &Self,
        rt: &RuntimeData,
    ) -> TinyVec<(f32, Cow<'e, <<Self as Timeable>::Data as Event>::Point>), 4>
    where
        <Self as Timeable>::Data: Event;

    /// for CacheableTimed<D>, this clears the event's cache,
    /// for Timed<D>, it's a no-op
    fn invalidate_cache(&mut self) {}
}


// Structs
/// Represents the span between 2 points and the current time.
#[derive(Copy, Clone, Debug)]
pub struct TimeSpan {
    pub start_beat: f32,
    pub end_beat: f32,
    pub now_beat: f32,
    start_seconds: Option<f32>,
    end_seconds: Option<f32>,
    pub now_seconds: f32,
}

/// Holds the beat and second representation of the current time.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Now {
    pub beat: f32,
    pub second: f32,
}

/// Animator for points.
/// Directly interpolates between 2 points to determine the output state.
pub struct PointAnimator<P: Point>(PhantomData<P>);

/// Animator for events.
/// First queries points from 2 target events, and then defers to PointAnimator.
pub struct EventAnimator<E: Event>(PhantomData<E>);

/// Simple timing wrapper, usable with both Points and Events.
pub struct Timed<D: TimedData> {
    pub data: D,
    beat: f32,
}

/// Caching timing wrapper, only usable with Events, and caches the points from get_points
/// to avoid repeated calculations.
pub struct CacheableTimed<E: Event> {
    data: E,
    beat: f32,
    // Box requires an allocation, but only requires it once, and gives the benefit
    // of a smaller struct and faster reading when not looking up cache.
    // Weather this is actually more useful is entirely dependent on the size of E::Point.
    cache: OnceCell<Box<[(f32, E::Point)]>>,
}

/// Holds a single animation track.
///
/// Points are looked up by a binary search which is the fastest and simplest
/// way to statelessly animate.
///
/// A track can use either Timed<D> or CacheableTimed<D> for Events,
/// and only Timed<D> for Points.
///
/// CacheableTimed<D> is only recommended if the Event you're using has a get_points implementation
/// that requires a lot of calculations or allocates memory.
pub struct Track<T: Timeable> {
    data: Vec<T>,
    /// Holds a cached default element for interpolating when the data list
    /// is empty or is animating before it's first element
    default: T,
    // only needed internally, the user should already
    // have this value anyways since it comes from them
    last_beat: f32,
}

/// Holds a set of animation tracks.
///
/// See Track<T> for more detail.
pub struct TrackSet<T: Timeable> {
    tracks: Vec<Vec<T>>,
    /// Holds a cached default element for interpolating when the data list
    /// is empty or is animating before it's first element
    default: T,
    // only needed internally, the user should already
    // have this value anyways since it comes from them
    last_beat: f32,
}

/// TrackSet builder for adding data elements quickly, and sorting at finalization time.
pub struct UninitializedTrackSet<T: Timeable> {
    tracks: Vec<Vec<T>>,
    // only needed internally, the user should already
    // have this value anyways since it comes from them
    last_beat: f32,
}


// Implementations
impl TimeSpan {
    pub fn new(now: Now, start: f32, end: f32) -> Self {
        Self {
            start_beat: start,
            end_beat: end,
            now_beat: now.beat,
            start_seconds: None,
            end_seconds: None,
            now_seconds: now.second,
        }
    }

    pub fn get_start_seconds(&mut self, rt: &RuntimeData) -> f32 {
        *self
            .start_seconds
            .get_or_insert_with(|| rt.beat_to_seconds(self.start_beat))
    }

    pub fn get_end_seconds(&mut self, rt: &RuntimeData) -> f32 {
        *self
            .end_seconds
            .get_or_insert_with(|| rt.beat_to_seconds(self.end_beat))
    }

    pub fn inverse_lerp_beats(&self) -> f32 {
        if self.end_beat <= self.start_beat {
            return 1.0;
        }
        f32::inverse_lerp(self.start_beat, self.end_beat, self.now_beat).clamp(0., 1.)
    }

    pub fn inverse_lerp_seconds(&mut self, rt: &RuntimeData) -> f32 {
        let s = self.get_start_seconds(rt);
        let e = self.get_end_seconds(rt);
        if e <= s {
            return 1.0;
        }
        f32::inverse_lerp(s, e, self.now_seconds).clamp(0., 1.)
    }

    pub fn elapsed_beats(&self) -> f32 {
        self.now_beat - self.start_beat
    }

    pub fn elapsed_seconds(&mut self, rt: &RuntimeData) -> f32 {
        self.now_seconds - self.get_start_seconds(rt)
    }
}

impl<'d, P: Point> TimedData for Cow<'d, P> {
    type Animator<T: Timeable<Data = Cow<'d, P>>> = PointAnimator<Self>;
    type State = P::State;
}

impl<'d, P: Point> Point for Cow<'d, P> {
    fn interpolate(&self, next: &Self, span: TimeSpan, rt: &RuntimeData) -> Self::State {
        P::interpolate(self, next, span, rt)
    }
    fn extend(&self, beat: f32, new_beat: f32, rt: &RuntimeData) -> Self {
        Cow::Owned(P::extend(self, beat, new_beat, rt))
    }
}

impl<P: Point, T: Timeable<Data = P>> Animator<P, T> for PointAnimator<P> {
    fn animate(
        default: &T,
        data: &[T],
        last_beat: f32,
        now: Now,
        rt: &RuntimeData,
    ) -> P::State {
        animate_timed(default, data, last_beat, now, rt)
    }
}

impl<E: Event, T: Timeable<Data = E>> Animator<E, T> for EventAnimator<E> {
    fn animate(
        default: &T,
        data: &[T],
        last_beat: f32,
        now: Now,
        rt: &RuntimeData,
    ) -> E::State {
        let i = data.partition_point(|e| e.beat() <= now.beat);

        let a = match i.checked_sub(2) {
            Some(j) => &data[j],
            None => default,
        };
        let b = i.checked_sub(1).map(|j| &data[j]);
        let c = data.get(i);

        let default_point: <E as Event>::Point = Default::default();
        match (b, c) {
            // Empty event list
            (None, None) => {
                let default_point = Timed::new(default_point, 0.);
                animate_timed(
                    &default_point,
                    &[],
                    last_beat,
                    now,
                    rt
                )
            }
            // first or last event, `a` is default when first event,
            // and the previous event when last
            (None, Some(e)) | (Some(e), None) => {
                let default_point = Timed::new(
                    Cow::Owned(default_point),
                    0.
                );
                let points = e.get_points(a, rt);
                let points: TinyVec<_, 4> = points.into_iter()
                    .map(|(beat, data)| Timed::new(data, beat))
                    .collect();
                animate_timed(
                    &default_point,
                    points.as_slice(),
                    last_beat,
                    now,
                    rt
                )
            }
            // between 2 events
            (Some(b), Some(c)) => {
                let default_point = Timed::new(
                    Cow::Owned(default_point),
                    0.
                );

                let b_points = b.get_points(a, rt);
                debug_assert!(
                    !b_points.is_empty(),
                    "get_points returned an empty point list ({})",
                    std::any::type_name::<E>()
                );
                debug_assert!(
                    b_points.windows(2).all(|w| w[0].0 <= w[1].0),
                    "get_points returned an unordered point list ({})",
                    std::any::type_name::<E>()
                );

                let c_points = c.get_points(b, rt);
                debug_assert!(
                    !c_points.is_empty(),
                    "get_points returned an empty point list ({})",
                    std::any::type_name::<E>()
                );
                debug_assert!(
                    c_points.windows(2).all(|w| w[0].0 <= w[1].0),
                    "get_points returned an unordered point list ({})",
                    std::any::type_name::<E>()
                );

                let cutoff = c_points.first()
                    .map_or(f32::INFINITY, |(beat, _)| *beat);
                let b_points = E::clip(b_points, cutoff, rt);

                let points: TinyVec<_, 8> = b_points.into_iter()
                    .filter(|(beat, _)| *beat <= cutoff)
                    .chain(c_points.into_iter())
                    .map(|(beat, data)| Timed::new(data, beat))
                    .collect();

                animate_timed(
                    &default_point,
                    points.as_slice(),
                    last_beat,
                    now,
                    rt
                )
            }
        }
    }
}

impl<D: TimedData> PrivateTimeable<D> for Timed<D> {
    #[inline]
    fn new(data: D, beat: f32) -> Self {
        Self { data, beat }
    }
    #[inline]
    fn take_data(self) -> D {
        self.data
    }
}

impl<D: TimedData> Timeable for Timed<D> {
    type Data = D;
    #[inline]
    fn beat(&self) -> f32 {
        self.beat
    }
    #[inline]
    fn data(&self) -> &D {
        &self.data
    }
    #[inline]
    fn data_mut(&mut self) -> &mut D {
        &mut self.data
    }
    #[inline]
    fn get_points<'e>(
        &'e self,
        previous: &Self,
        rt: &RuntimeData,
    ) -> TinyVec<(f32, Cow<'e, <<Self as Timeable>::Data as Event>::Point>), 4>
    where
        <Self as Timeable>::Data: Event,
    {
        self.data.get_points(self.beat, previous, rt)
    }
}

impl<E: Event> PrivateTimeable<E> for CacheableTimed<E> {
    #[inline]
    fn new(data: E, beat: f32) -> Self {
        Self {
            data,
            beat,
            cache: Default::default(),
        }
    }
    #[inline]
    fn take_data(self) -> E {
        self.data
    }
}

impl<E: Event> Timeable for CacheableTimed<E> {
    type Data = E;
    #[inline]
    fn beat(&self) -> f32 {
        self.beat
    }
    #[inline]
    fn data(&self) -> &E {
        &self.data
    }
    #[inline]
    fn data_mut(&mut self) -> &mut E {
        self.invalidate_cache();
        &mut self.data
    }

    fn get_points<'e>(
        &'e self,
        previous: &Self,
        rt: &RuntimeData,
    ) -> TinyVec<(f32, Cow<'e, <Self::Data as Event>::Point>), 4>
    where
        Self::Data: Event,
    {
        let cached = self.cache.get_or_init(|| {
            self.data.get_points(self.beat, previous, rt)
                .into_iter()
                .map(|(b, c)| (b, c.into_owned()))
                .collect::<Box<[_]>>()
        });
        let mut out = TinyVec::with_capacity(cached.len());
        for (b, p) in cached.iter() {
            out.push((*b, Cow::Borrowed(p)));
        }
        out
    }

    fn invalidate_cache(&mut self) {
        self.cache.take();
    }
}

impl<T: Timeable> Track<T> {
    pub fn empty(last_beat: f32) -> Self {
        Self::from_vec(Vec::new(), last_beat)
    }

    pub(crate) fn from_vec(mut data: Vec<T>, last_beat: f32) -> Self {
        data.sort_by(|a, b| a.beat().total_cmp(&b.beat()));
        clip_vec(&mut data, last_beat);
        Self {
            data,
            default: T::new(Default::default(), 0.),
            last_beat,
        }
    }

    pub fn animate(&self, now: Now, rt: &RuntimeData) -> <T::Data as TimedData>::State {
        <T::Data as TimedData>::Animator::animate(
            &self.default,
            &self.data,
            self.last_beat,
            now,
            rt,
        )
    }

    pub fn insert(&mut self, beat: f32, data: T::Data) -> Result<(), TrackInsertionError> {
        track_insert(&mut self.data, self.last_beat, beat, data)
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn get_mut(&mut self, index: usize) -> Option<(f32, &mut T::Data)> {
        invalidate_neighbors(self.data.as_mut_slice(), index);
        let d = self.data.get_mut(index)?;
        Some((d.beat(), d.data_mut()))
    }

    pub fn remove(&mut self, index: usize) -> Option<(f32, T::Data)> {
        if index >= self.data.len() {
            None
        } else {
            invalidate_neighbors(self.data.as_mut_slice(), index);
            let t = self.data.remove(index);
            Some((t.beat(), t.take_data()))
        }
    }

    /// Should be called after any changes are made to the RuntimeData or the Track
    pub fn invalidate_caches(&mut self) {
        self.data.iter_mut().for_each(|p| p.invalidate_cache());
    }
}

impl<T: Timeable> TrackSet<T> {
    pub fn empty(tracks: usize, last_beat: f32) -> Self {
        let tracks = (0..tracks).map(|_| Vec::new()).collect();
        Self {
            tracks,
            default: T::new(Default::default(), 0.),
            last_beat,
        }
    }

    pub fn new(tracks: usize, last_beat: f32) -> UninitializedTrackSet<T> {
        let tracks = (0..tracks).map(|_| Vec::new()).collect();
        UninitializedTrackSet { tracks, last_beat }
    }

    pub(crate) fn from_vecs(mut tracks: Vec<Vec<T>>, last_beat: f32) -> Self {
        for track in tracks.iter_mut() {
            track.sort_by(|a, b| a.beat().total_cmp(&b.beat()));
            clip_vec(track, last_beat);
        }
        Self {
            tracks,
            default: T::new(Default::default(), 0.),
            last_beat,
        }
    }

    pub fn animate(
        &self,
        track: usize,
        now: Now,
        rt: &RuntimeData,
    ) -> Option<<T::Data as TimedData>::State> {
        let track = self.tracks.get(track)?;
        Some(<T::Data as TimedData>::Animator::animate(
            &self.default,
            track,
            self.last_beat,
            now,
            rt,
        ))
    }

    /// Note: if the track set has fewer tracks than the frame expects, this creates a default
    /// data value to place in the list for each missing track.
    pub fn iter_animate<R>(
        &self,
        now: Now,
        rt: &RuntimeData,
        track_count: usize,
        frame: &mut Vec<R>,
        transformer: impl Fn(<T::Data as TimedData>::State) -> R,
    ) {
        for i in 0..track_count {
            let state = match self.tracks.get(i) {
                Some(track) => <T::Data as TimedData>::Animator::animate(
                    &self.default,
                    track,
                    self.last_beat,
                    now,
                    rt,
                ),
                None => <T::Data as TimedData>::State::default(),
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
        now: Now,
        rt: &RuntimeData,
        frame: &mut [MaybeUninit<R>],
        transformer: impl Fn(<T::Data as TimedData>::State) -> R,
    ) -> usize {
        for i in 0..frame.len() {
            if i == self.tracks.len() {
                return i;
            }
            let track = &self.tracks[i];
            let state = <T::Data as TimedData>::Animator::animate(
                &self.default,
                track,
                self.last_beat,
                now,
                rt,
            );
            frame[i].write(transformer(state));
        }
        frame.len()
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    pub fn insert(
        &mut self,
        track: usize,
        beat: f32,
        data: T::Data,
    ) -> Result<(), TrackSetInsertionError> {
        if let Some(track) = self.tracks.get_mut(track) {
            Ok(track_insert(track, self.last_beat, beat, data)?)
        } else {
            Err(TrackSetInsertionError::InvalidId {
                max: self.tracks.len(),
                id: track,
            })
        }
    }

    pub fn data(&self, track: usize) -> Option<&[T]> {
        Some(self.tracks.get(track)?)
    }

    pub fn get_mut(&mut self, track: usize, index: usize) -> Option<(f32, &mut T::Data)> {
        let track = self.tracks.get_mut(track)?;
        invalidate_neighbors(track.as_mut_slice(), index);
        let d = track.get_mut(index)?;
        Some((d.beat(), d.data_mut()))
    }

    pub fn remove(&mut self, track: usize, index: usize) -> Option<(f32, T::Data)> {
        let track = self.tracks.get_mut(track)?;
        if index >= track.len() {
            None
        } else {
            invalidate_neighbors(track.as_mut_slice(), index);
            let t = track.remove(index);
            Some((t.beat(), t.take_data()))
        }
    }

    /// Should be called after any changes are made to the RuntimeData or the TrackSet
    pub fn invalidate_caches(&mut self) {
        self.tracks.iter_mut().for_each(|t| {
            t.iter_mut().for_each(|p| p.invalidate_cache());
        });
    }
}

impl<T: Timeable> UninitializedTrackSet<T> {
    pub fn add(
        &mut self,
        track: usize,
        beat: f32,
        data: T::Data,
    ) -> Result<(), TrackSetInsertionError> {
        if let Some(track) = self.tracks.get_mut(track) {
            if (0f32..=self.last_beat).contains(&beat) {
                track.push(T::new(data, beat));
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


// Macros
/// Simplifies implementing TimedData for types.
/// Usage:
/// ```rs
/// timed!(point MyPoint: f32); // sets State = f32, and Animator = PointAnimator
/// timed!(event MyEvent: f32); // sets State = f32, and Animator = EventAnimator
/// ```
#[macro_export]
macro_rules! timed {
    (event $name:ident: $state:ty) => {
        impl $crate::tracks::TimedData for $name {
            type Animator<T: $crate::tracks::Timeable<Data=Self>>
                = $crate::tracks::EventAnimator<Self>;
            type State = $state;
        }
    };
    (point $name:ident: $state:ty) => {
        impl $crate::tracks::TimedData for $name {
            type Animator<T: $crate::tracks::Timeable<Data=Self>>
                = $crate::tracks::PointAnimator<Self>;
            type State = $state;
        }
    };
}

pub use timed;
