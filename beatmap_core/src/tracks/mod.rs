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
    pub fn new(now: Now, start: f32, end: f32) -> Self {
        Self {
            start,
            end,
            now: now.beat,
            start_seconds: None,
            end_seconds: None,
            now_seconds: now.second,
        }
    }

    pub fn get_start_seconds(&mut self, rt: &RuntimeData) -> f32 {
        *self
            .start_seconds
            .get_or_insert_with(|| rt.beat_to_seconds(self.start))
    }

    pub fn get_end_seconds(&mut self, rt: &RuntimeData) -> f32 {
        *self
            .end_seconds
            .get_or_insert_with(|| rt.beat_to_seconds(self.end))
    }

    pub fn inverse_lerp_beats(&self) -> f32 {
        if self.end <= self.start {
            return 1.0;
        }
        f32::inverse_lerp(self.start, self.end, self.now).clamp(0., 1.)
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
        self.now - self.start
    }

    pub fn elapsed_seconds(&mut self, rt: &RuntimeData) -> f32 {
        self.now_seconds - self.get_start_seconds(rt)
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

pub trait Animator<D: TimedData, T: Timeable<Data = D>> {
    fn animate(
        default: &T,
        track: &[T],
        last_beat: f32,
        now: Now,
        rt: &RuntimeData,
    ) -> D::State;
}

pub trait TimedData: Clone + Default {
    type Animator<T: Timeable<Data = Self>>: Animator<Self, T>;

    type State: Clone + Default;

    fn extend(&self, beat: f32, new_beat: f32, runtime_data: &RuntimeData) -> Self {
        let _ = (beat, new_beat, runtime_data);
        self.clone()
    }
}

pub trait Point: TimedData {
    fn interpolate(&self, other: &Self, span: BeatSpan, runtime_data: &RuntimeData) -> Self::State;
}

/// For events that require multiple points to animate.
pub trait Event: TimedData {
    type Point: Point + TimedData<State = Self::State>;

    /// Return all points that this event turns into.
    /// Track<E: Event> handles trimming so that points from this event don't appear after the next event's points
    /// the returned points are assumed to be sorted, and is not checked.
    fn get_points<'e, T: Timeable<Data = Self>>(
        &'e self,
        event_beat: f32,
        previous: &T,
        next: Option<&T>,
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

pub struct PointAnimator<P: Point>(PhantomData<P>);
pub struct EventAnimator<E: Event>(PhantomData<E>);

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

        let ab = b.unwrap_or(a);

        let extended;
        let c = match data.get(i) {
            Some(e) => e,
            None => {
                extended = T::new(ab.data().extend(ab.beat(), last_beat, rt), last_beat);
                &extended
            }
        };

        let d = data.get(i + 1);

        let abc = b.map(|e| {
            let abc = e.get_points(a, data.get(i), rt);
            debug_assert!(
                abc.windows(2).all(|w| w[0].0 <= w[1].0),
                "get_points returned unordered points, {}",
                std::any::type_name::<E>()
            );
            abc
        });

        let bcd = c.get_points(ab, d, rt);
        debug_assert!(
            bcd.windows(2).all(|w| w[0].0 <= w[1].0),
            "get_points returned unordered points, {}",
            std::any::type_name::<E>(),
        );

        let cutoff = if let Some(p0) = bcd.first() {
            p0.0
        } else {
            #[cfg(feature = "tracing")]
            tracing::error!("Event didn't return any points (at beat {})", now.beat);
            let def = E::Point::default();
            let end = def.extend(0., last_beat, rt);
            let span = BeatSpan::new(now, 0., last_beat);
            return def.interpolate(&end, span, rt);
        };

        let max_count = bcd.len() + abc.as_ref().map(|p| p.len()).unwrap_or_default();

        let mut points: TinyVec<_, 6> = TinyVec::with_capacity(max_count);

        if let Some(abc) = abc {
            let abc = E::clip(abc, cutoff, rt);
            for (beat, data) in abc {
                if beat <= cutoff {
                    points.push(Timed::new(data, beat))
                }
            }
        }

        for (beat, data) in bcd {
            points.push(Timed::new(data, beat));
        }

        animate_timed(
            &Timed::new(Default::default(), 0.),
            &points,
            last_beat,
            now,
            rt,
        )
    }
}

pub(crate) trait PrivateTimeable<D: TimedData> {
    /// new is private.
    fn new(data: D, beat: f32) -> Self;
}

#[allow(private_bounds)]
pub trait Timeable: PrivateTimeable<Self::Data> {
    type Data: TimedData;
    fn beat(&self) -> f32;
    fn data(&self) -> &<Self as Timeable>::Data;
    fn data_mut(&mut self) -> &mut <Self as Timeable>::Data;
    fn take_data(self) -> <Self as Timeable>::Data;

    fn get_points<'e>(
        &'e self,
        previous: &Self,
        next: Option<&Self>,
        rt: &RuntimeData,
    ) -> TinyVec<(f32, Cow<'e, <<Self as Timeable>::Data as Event>::Point>), 4>
    where
        <Self as Timeable>::Data: Event;

    fn invalidate_cache(&mut self) {}
}

pub struct Timed<D: TimedData> {
    pub data: D,
    beat: f32,
}

// intentionally doesn't have Clone or any other public ctor since this is essentially a private
// struct. The only reason it's public is to allow reading a slice without having to transmute or
// clone.
// this one has even more reason to not be Cloneable due to the cache.
pub struct CacheableTimedEvent<E: Event> {
    data: E,
    beat: f32,
    // Box requires an allocation, but only requires it once, and gives the benefit
    // of a smaller struct and faster reading when not looking up cache.
    // Weather this is actually more useful is entirely dependent on the size of E::Point.
    cache: OnceCell<Box<TinyVec<(f32, E::Point), 4>>>,
}

pub struct Track<T: Timeable> {
    data: Vec<T>,
    /// Holds a cached default element for interpolating when the data list
    /// is empty or is animating before it's first element
    default: T,
    // only needed internally, the user should already
    // have this value anyways since it comes from them
    last_beat: f32,
}

pub struct TrackSet<T: Timeable> {
    tracks: Vec<Vec<T>>,
    /// Holds a cached default element for interpolating when the data list
    /// is empty or is animating before it's first element
    default: T,
    // only needed internally, the user should already
    // have this value anyways since it comes from them
    last_beat: f32,
}

pub struct UninitializedTrackSet<T: Timeable> {
    tracks: Vec<Vec<T>>,
    // only needed internally, the user should already
    // have this value anyways since it comes from them
    last_beat: f32,
}

impl<'d, P: Point> TimedData for Cow<'d, P> {
    type Animator<T: Timeable<Data = Cow<'d, P>>> = PointAnimator<Self>;
    type State = P::State;
    fn extend(&self, beat: f32, new_beat: f32, rt: &RuntimeData) -> Self {
        Cow::Owned(P::extend(self, beat, new_beat, rt))
    }
}

impl<'d, P: Point> Point for Cow<'d, P> {
    fn interpolate(&self, other: &Self, span: BeatSpan, rt: &RuntimeData) -> Self::State {
        P::interpolate(self, other, span, rt)
    }
}

impl<D: TimedData> PrivateTimeable<D> for Timed<D> {
    #[inline]
    fn new(data: D, beat: f32) -> Self {
        Self { data, beat }
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
    fn take_data(self) -> D {
        self.data
    }

    fn get_points<'e>(
        &'e self,
        previous: &Self,
        next: Option<&Self>,
        rt: &RuntimeData,
    ) -> TinyVec<(f32, Cow<'e, <<Self as Timeable>::Data as Event>::Point>), 4>
    where
        <Self as Timeable>::Data: Event,
    {
        self.data.get_points(self.beat, previous, next, rt)
    }
}

impl<E: Event> PrivateTimeable<E> for CacheableTimedEvent<E> {
    #[inline]
    fn new(data: E, beat: f32) -> Self {
        Self {
            data,
            beat,
            cache: Default::default(),
        }
    }
}

impl<E: Event> Timeable for CacheableTimedEvent<E> {
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
    #[inline]
    fn take_data(self) -> E {
        self.data
    }

    fn get_points<'e>(
        &'e self,
        previous: &Self,
        next: Option<&Self>,
        rt: &RuntimeData,
    ) -> TinyVec<(f32, Cow<'e, <Self::Data as Event>::Point>), 4>
    where
        Self::Data: Event,
    {
        let cached = self.cache.get_or_init(|| {
            let mut out = Box::new(TinyVec::new());
            for (b, c) in self.data.get_points(self.beat, previous, next, rt) {
                out.push((b, c.into_owned()));
            }
            out
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Now {
    pub beat: f32,
    pub second: f32,
}

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
        .interpolate(b.data(), BeatSpan::new(now, a.beat(), b.beat()), rt)
}

/// invalidates `i`'s neighbors. `i` is left alone since it's assumed to get invalidated by
/// whatever operation caused this to be called in the first place.
fn invalidate_neighbors<T: Timeable>(list: &mut [T], i: usize) {
    if let Some(j) = i.checked_sub(1) && let Some(p) = list.get_mut(j) {
        p.invalidate_cache();
    }
    if let Some(p) = list.get_mut(i.saturating_add(1)) {
        p.invalidate_cache();
    }
}

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
