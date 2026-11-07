use crate::RuntimeData;

pub mod v2;
pub mod v3;

pub trait Point: Clone {
    type Output;
    fn startpoint(beat: f32) -> Self;
    fn extend(&self, beat: f32) -> Self;

    fn beat(&self) -> f32;
    fn interpolate(&self, other: &Self, beat: f32, runtime_data: &RuntimeData) -> Self::Output;
}

#[derive(Default, Clone)]
pub struct Track<P: Point> {
    points: Vec<P>,
}

impl<P: Point> Track<P> {
    pub fn new(last_beat: f32) -> Self {
        Self::from_point_vec(Vec::new(), last_beat)
    }
    pub fn from_point_vec(mut points: Vec<P>, last_beat: f32) -> Self {
        points.sort_by(|a, b| a.beat().total_cmp(&b.beat()));
        #[cfg(feature = "tracing")]
        let l = points.len();
        points.retain(|p| p.beat() <= last_beat );
        #[cfg(feature = "tracing")]
        if l != points.len() {
            let s = if l == 1 { "" } else { "s" };
            let diff = l - points.len();
            tracing::warn!("{diff} point{s} exceeded the length of the song");
        }

        // # Safety
        // this block ensures that the list contains at least 1 item.
        if points.get(0).is_none_or(|p| p.beat() != 0.) {
            let p0 = P::startpoint(0.);
            points.insert(0, p0);
        }

        // # Safety
        // above we have guaranteed that the list has at least 1 item.
        let last =  unsafe { points.get_unchecked(points.len()-1) };
        if last.beat() != last_beat {
            points.push(last.extend(last_beat));
        }

        Self { points }
    }

    /// This takes both beats and seconds
    pub fn animate(&self, beat: f32, runtime_data: &RuntimeData) -> P::Output {
        let i = self.points.partition_point(|p| p.beat() <= beat).clamp(1, self.points.len()-1);

        let a = self.points.get(i-1).unwrap();
        let b = self.points.get(i).unwrap();

        a.interpolate(b, beat, runtime_data)
    }

    pub fn points(&self) -> &[P] {
        &self.points
    }

}

pub struct UninitializedTrackSet<P: Point> {
    tracks: Vec<Vec<P>>,
    song_length: f32,
}

pub struct TrackSet<P: Point> {
    tracks: Vec<Track<P>>
}

impl<P: Point> TrackSet<P> {

    pub fn new(light_count: usize, song_length: f32) -> UninitializedTrackSet<P> {
        UninitializedTrackSet {
            tracks: vec![Vec::new(); light_count],
            song_length,
        }
    }

    pub fn animate(&self, light_id: usize, beat: f32, runtime_data: &RuntimeData) -> Option<P::Output> {
        let track = self.tracks.get(light_id)?;
        Some(track.animate(beat, runtime_data))
    }

}

impl<P: Point> UninitializedTrackSet<P> {
    pub fn add_point(&mut self, light_id: usize, point: P) {
        if let Some(track) = self.tracks.get_mut(light_id) {
            track.push(point);
        }
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

