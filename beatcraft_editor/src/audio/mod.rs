use std::collections::VecDeque;
use std::fmt::Display;
use std::path::Path;
use std::{ptr, thread};
use std::sync::{Arc, mpsc};
use glow::HasContext;

use eframe::glow;
use parking_lot::RwLock;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader, TrackType};
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::audio::sample::Sample;

use crate::DB_AUDIO;

pub mod al;

type AudioTask = Box<dyn FnMut() -> TaskAction + Send>;

const FULL_BUFFER_COUNT: usize = 4;
const FULL_CHUNK_SAMPLES: usize = 8192;
const SPECTROGRAM_UPLOAD_BATCH: usize = 128;

pub enum AudioThreadCommand {
    AddTask(AudioTask),
}

#[derive(thiserror::Error, Debug)]
pub enum AudioError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Audio Error: {0}")]
    Symphonia(#[from] symphonia::core::errors::Error),
    #[error("{0}")]
    String(String),
    #[error("{0}")]
    Str(&'static str),
}

#[derive(thiserror::Error, Debug)]
pub enum PlaybackError {
    #[error("Audio data is not ready yet")]
    NotReady,
    #[error("Cannot seek to invalid position: {0}")]
    InvalidSeekPosition(f32),
}

pub struct AudioSystem {
    pub device: *mut al::ALCdevice,
    pub context: *mut al::ALCcontext,
    pub thread_commands: mpsc::Sender<AudioThreadCommand>,
    pub audio_refs: Vec<Arc<Audio>>,

    gl: Arc<glow::Context>,
}

pub struct AudioInfo {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    sample_rate: u32,
    channels: u16,
    sample_count: Option<usize>,
    track_id: u32,
}

struct SpectrogramState {
    data: Arc<RwLock<Vec<i16>>>,
    decode_cursor: Arc<RwLock<usize>>,
    columns_done: Arc<RwLock<usize>>,
    decode_finished: Arc<RwLock<bool>>,
    window_size: usize,
    hop: usize,
    channels: u16,
    hann: Vec<f32>,
    fft: Arc<dyn rustfft::Fft<f32>>,
    tex_data: Arc<RwLock<Vec<f32>>>,
    finished: Arc<RwLock<bool>>,
}

impl AudioSystem {
    pub fn new(gl: Arc<glow::Context>) -> Result<Self, AudioError> {
        unsafe {
            let device = al::alcOpenDevice(ptr::null());
            if device.is_null() {
                return Err(AudioError::Str("Failed to open device"));
            }
            let context = al::alcCreateContext(device, ptr::null());
            if context.is_null() {
                return Err(AudioError::Str("Failed to create context"));
            }

            if al::alcMakeContextCurrent(context) == 0 {
                return Err(AudioError::Str("Failed to make context current"));
            }

            let list_ptr = al::alcGetString(ptr::null_mut(), al::ALC_DEVICE_SPECIFIER);
            let mut ptr = list_ptr;
            loop {
                let s = std::ffi::CStr::from_ptr(ptr);
                if s.to_bytes().is_empty() { break; }
                tracing::debug!(target: DB_AUDIO, "Available device: {}", s.to_string_lossy());
                ptr = ptr.add(s.to_bytes_with_nul().len());
            }

            let name_ptr = al::alcGetString(device, al::ALC_DEVICE_SPECIFIER);
            let name = std::ffi::CStr::from_ptr(name_ptr).to_string_lossy();
            tracing::debug!(target: DB_AUDIO, "Bound to device: {name}");

            let (sx, rx) = mpsc::channel();

            let at = AudioThread::new(rx);

            thread::spawn(move || at.main_loop());

            tracing::debug!(target: DB_AUDIO, "Initialized audio system");
            Ok(Self {
                device,
                context,
                thread_commands: sx,
                audio_refs: Vec::new(),
                gl,
            })
        }
    }

    pub fn remove_dead_audio(&mut self) {
        self.audio_refs.retain(|a| Arc::strong_count(a) > 1);
    }

    pub fn clear_audio(&mut self) {
        self.audio_refs.clear();
    }

    pub fn update(&mut self) {

        let mut audios = Vec::with_capacity(self.audio_refs.len());
        std::mem::swap(&mut audios, &mut self.audio_refs);

        for audio in audios.into_iter() {
            audio.update();
            if *audio.loaded.read() {
                self.audio_refs.push(audio);
            } else {
                tracing::debug!(target: DB_AUDIO, "Audio unloaded, removing from update queue");
            }
        }

    }

    pub fn add_task(&self, task: AudioTask) {
        let _ = self.thread_commands.send(AudioThreadCommand::AddTask(task));
    }

    pub fn open_decoder(path: &Path) -> Result<AudioInfo, AudioError> {
        let file = std::fs::File::open(path)?;

        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(mut ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "egg" {
                ext = "ogg";
            }
            hint.with_extension(ext);
        }

        let format = symphonia::default::get_probe()
            .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())?;

        let track = format.default_track(TrackType::Audio).ok_or(AudioError::Str("No valid audio track"))?;

        let track_id = track.id;
        let num_frames = track.num_frames;
        let track = track.codec_params.as_ref().unwrap().audio().unwrap();

        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(track, &AudioDecoderOptions::default())?;

        let sample_rate = track.sample_rate.unwrap_or(44100);
        let channels = track.channels.as_ref().map(|c| c.count() as u16).unwrap_or(2);
        let sample_count = num_frames.map(|u| u as usize);

        Ok(AudioInfo {
            format_reader: format,
            decoder,
            sample_rate,
            channels,
            sample_count,
            track_id,
        })
    }

}

impl Drop for AudioSystem {
    fn drop(&mut self) {
        unsafe {
            al::alcDestroyContext(self.context);
            al::alcCloseDevice(self.device);
        }
        tracing::debug!(target: DB_AUDIO, "Closed audio system");
    }
}

struct AudioThread {
    commands: mpsc::Receiver<AudioThreadCommand>,
    tasks: Vec<AudioTask>,
}

impl AudioThread {
    pub fn new(channel: mpsc::Receiver<AudioThreadCommand>) -> Self {
        Self {
            commands: channel,
            tasks: Vec::new(),
        }
    }

    fn main_loop(mut self) {
        let span = tracing::debug_span!("thread/audio");
        let _guard = span.enter();
        tracing::debug!(target: DB_AUDIO, "Started audio thread");
        'mainloop: loop {

            'io_loop: loop {
                match self.commands.try_recv() {
                    Err(mpsc::TryRecvError::Empty) => break 'io_loop,
                    Err(mpsc::TryRecvError::Disconnected) => break 'mainloop,
                    Ok(cmd) => match cmd {
                        AudioThreadCommand::AddTask(task) => {
                            tracing::debug!(target: DB_AUDIO, "Added new audio task");
                            self.tasks.push(task);
                        }
                    },
                }
            }

            let mut tasks = Vec::with_capacity(self.tasks.len());
            std::mem::swap(&mut tasks, &mut self.tasks);
            for mut task in tasks.into_iter() {
                match task() {
                    TaskAction::None => self.tasks.push(task),
                    TaskAction::Remove => {},
                }
            }
        }
    }
}

pub fn check_al_error(where_: impl Display) {
    unsafe {
        let err = al::alGetError();
        if err != al::AL_NO_ERROR {
            tracing::error!("AL error at: {where_}: {err:#x}");
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum LoadTask {
    Background,
    Playback,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TaskAction {
    None,
    Remove,
}

#[derive(Copy, Clone, Debug)]
enum AudioLoadState {
    /// end is exclusive
    Empty { start: usize, end: usize },
    /// end is exclusive
    Loaded { start: usize, end: usize },
    Loading { start: usize, task: LoadTask },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AudioMode {
    Stream,
    Full,
}

#[derive(Debug)]
enum AudioSource {
    Stream {
        loaded_ranges: Arc<RwLock<Vec<(usize, usize)>>>,
        decode_cursor: Arc<RwLock<usize>>,
        seek_target: Arc<RwLock<Option<usize>>>,
        playback_position: usize,
        free_buffers: Vec<u32>,
        queued_sizes: VecDeque<usize>,
        played_samples: Arc<RwLock<usize>>,
    },
    Full {
        load_position: Arc<RwLock<usize>>,
        playback_position: usize,
        free_buffers: Vec<u32>,
        all_buffers: Vec<u32>,
        queued_sizes: VecDeque<usize>,
        played_samples: Arc<RwLock<usize>>,
    },
}

#[derive(Debug)]
struct SpectrogramGlCache {
    tex: glow::NativeTexture,
    uploaded_columns: usize,
    capacity_columns: usize,
}


#[derive(Debug)]
pub struct Audio {
    data: Arc<RwLock<Vec<i16>>>,
    sample_count: Arc<RwLock<Option<usize>>>,
    source: Arc<RwLock<AudioSource>>,
    loaded: Arc<RwLock<bool>>,
    pending_play: Arc<RwLock<bool>>,
    src_handle: u32,
    channels: u16,
    sample_rate: u32,
    fx_filter: u32,

    spectrogram_tex_data: Arc<RwLock<Vec<f32>>>,
    spectrogram_columns_done: Arc<RwLock<usize>>,
    spectrogram_freq_bins: usize,
    spectrogram_hop: usize,
    spectrogram_finished: Arc<RwLock<bool>>,
    spectrogram_gl_cache: RwLock<Option<SpectrogramGlCache>>,

    gl: Arc<glow::Context>,
}

impl Audio {

    pub fn new(audio_sys: &mut AudioSystem, path: &Path, mode: AudioMode) -> Result<Arc<Self>, AudioError> {
        let span = tracing::debug_span!("audio init");
        let _guard = span.enter();
        match mode {
            AudioMode::Stream => {
                let audio_info = AudioSystem::open_decoder(path)?;
                let data: Arc<RwLock<Vec<i16>>> = Arc::default();
                let sections: Arc<RwLock<Vec<(usize, usize)>>> = Arc::default();
                let pb_cursor: Arc<RwLock<usize>> = Arc::default();
                let seek_target = Arc::new(RwLock::new(None));

                let mut buffers = vec![0u32; FULL_BUFFER_COUNT];
                unsafe { al::alGenBuffers(FULL_BUFFER_COUNT as i32, buffers.as_mut_ptr()); }
                check_al_error("after alGenBuffers");
                tracing::debug!(target: DB_AUDIO, "Generated {} AL buffers for streaming", FULL_BUFFER_COUNT);

                let source = Arc::new(RwLock::new(AudioSource::Stream {
                    loaded_ranges: Arc::clone(&sections),
                    decode_cursor: Arc::clone(&pb_cursor),
                    seek_target: Arc::clone(&seek_target),
                    playback_position: 0,
                    free_buffers: buffers,
                    queued_sizes: VecDeque::new(),
                    played_samples: Arc::default(),
                }));
                let loaded = Arc::new(RwLock::new(true));

                let mut src_handle = [0];
                unsafe { al::alGenSources(1, src_handle.as_mut_ptr()); }
                let [src_handle] = src_handle;

                let mut fx_filter = [0];
                unsafe { al::alGenFilters(1, fx_filter.as_mut_ptr()) };
                let [fx_filter] = fx_filter;

                unsafe {
                    al::alFilteri(fx_filter, al::AL_FILTER_TYPE, al::AL_FILTER_LOWPASS);
                    al::alFilterf(fx_filter, al::AL_LOWPASS_GAIN, 1.);
                    al::alFilterf(fx_filter, al::AL_LOWPASS_GAINHF, 0.1);
                }

                let decode_finished: Arc<RwLock<bool>> = Arc::default();
                let sample_count = Arc::new(RwLock::new(audio_info.sample_count));
                let spec_data = Arc::clone(&data);
                let spec_cursor = Arc::clone(&pb_cursor);
                let sample_rate = audio_info.sample_rate;
                let (spectrogram_tex_data, spectrogram_columns_done, spectrogram_freq_bins, spectrogram_finished, spectrogram_hop) =
                    Self::spawn_spectrogram_task(audio_sys, spec_data, spec_cursor, decode_finished, audio_info.channels, sample_rate);

                let data2 = Arc::clone(&data);
                let loaded2 = Arc::clone(&loaded);
                let mut audio_info2 = AudioSystem::open_decoder(path)?;
                audio_sys.add_task(Box::new(move || {
                    Self::decode_task_loop(&data2, &sections, &pb_cursor, &seek_target, &loaded2, audio_info.channels, &mut audio_info2)
                }));

                let audio = Arc::new(Self {
                    data,
                    sample_count,
                    source,
                    loaded,
                    pending_play: Arc::new(RwLock::new(false)),
                    src_handle,
                    channels: audio_info.channels,
                    sample_rate: audio_info.sample_rate,
                    fx_filter,
                    spectrogram_columns_done,
                    spectrogram_freq_bins,
                    spectrogram_tex_data,
                    spectrogram_hop,
                    spectrogram_finished,
                    spectrogram_gl_cache: RwLock::default(),
                    gl: Arc::clone(&audio_sys.gl),
                });

                audio_sys.audio_refs.push(Arc::clone(&audio));

                Ok(audio)
            },
            AudioMode::Full => {
                tracing::debug!(target: DB_AUDIO, ?path, "Loading file in Full mode.");
                let data = Arc::new(RwLock::new(Vec::new()));
                let mut audio_info = AudioSystem::open_decoder(path)?;
                let channels = audio_info.channels;
                let sample_rate = audio_info.sample_rate;

                let load_pos = Arc::new(RwLock::new(0));

                let mut src_handle = [0];
                unsafe { al::alGenSources(1, src_handle.as_mut_ptr()); }
                let [src_handle] = src_handle;
                check_al_error("after alGenSources");

                let mut buffers = vec![0u32; FULL_BUFFER_COUNT];
                unsafe { al::alGenBuffers(FULL_BUFFER_COUNT as i32, buffers.as_mut_ptr()); }
                check_al_error("after alGenBuffers");
                tracing::debug!(target: DB_AUDIO, "Generated {} AL buffers for full-streaming", FULL_BUFFER_COUNT);

                let source = Arc::new(RwLock::new(AudioSource::Full {
                    load_position: Arc::clone(&load_pos),
                    playback_position: 0,
                    free_buffers: buffers.clone(),
                    all_buffers: buffers,
                    queued_sizes: VecDeque::new(),
                    played_samples: Arc::default(),
                }));

                let loaded = Arc::new(RwLock::new(true));

                let mut fx_filter = [0];
                unsafe { al::alGenFilters(1, fx_filter.as_mut_ptr()) };
                let [fx_filter] = fx_filter;
                unsafe {
                    al::alFilteri(fx_filter, al::AL_FILTER_TYPE, al::AL_FILTER_LOWPASS);
                    al::alFilterf(fx_filter, al::AL_LOWPASS_GAIN, 1.);
                    al::alFilterf(fx_filter, al::AL_LOWPASS_GAINHF, 0.1);
                }

                let sample_count = Arc::new(RwLock::new(audio_info.sample_count));

                let decode_finished: Arc<RwLock<bool>> = Arc::default();

                let data2 = Arc::clone(&data);
                let decode_cursor = Arc::clone(&load_pos);
                let loaded2 = Arc::clone(&loaded);
                let sc2 = Arc::clone(&sample_count);
                let dcf2 = Arc::clone(&decode_finished);
                audio_sys.add_task(Box::new(move || {
                    Self::load_full(&data2, &loaded2, &load_pos, &mut audio_info, &sc2, &dcf2)
                }));

                let (spectrogram_tex_data, spectrogram_columns_done, spectrogram_freq_bins, spectrogram_finished, spectrogram_hop) =
                    Self::spawn_spectrogram_task(audio_sys, Arc::clone(&data), decode_cursor, decode_finished, channels, sample_rate);

                let audio = Arc::new(Self {
                    data,
                    sample_count,
                    source,
                    loaded,
                    pending_play: Arc::new(RwLock::new(false)),
                    src_handle,
                    channels,
                    sample_rate,
                    fx_filter,
                    spectrogram_tex_data,
                    spectrogram_columns_done,
                    spectrogram_freq_bins,
                    spectrogram_hop,
                    spectrogram_finished,
                    spectrogram_gl_cache: RwLock::default(),
                    gl: Arc::clone(&audio_sys.gl),
                });

                audio_sys.audio_refs.push(Arc::clone(&audio));
                Ok(audio)
            },
        }
    }

    pub fn update(&self) {
        let mut source = self.source.write();
 
        match &mut *source {
            AudioSource::Stream { loaded_ranges, playback_position, free_buffers, .. } => {
                unsafe {
                    let mut processed = [0i32];
                    al::alGetSourcei(self.src_handle, al::AL_BUFFERS_PROCESSED, processed.as_mut_ptr());
                    for _ in 0..processed[0] {
                        let mut buf = [0u32];
                        al::alSourceUnqueueBuffers(self.src_handle, 1, buf.as_mut_ptr());
                        free_buffers.push(buf[0]);
                    }
                }

                let ranges = loaded_ranges.read();
                let available = Self::contiguous_loaded_end(&ranges, *playback_position);
                drop(ranges);
                let dat = self.data.read();

                while let Some(buf) = free_buffers.pop() {
                    let remaining = available.saturating_sub(*playback_position);
                    if remaining == 0 { free_buffers.push(buf); break; }
                    let take = remaining.min(FULL_CHUNK_SAMPLES);
                    let slice = &dat[*playback_position..*playback_position + take];
                    unsafe {
                        let format = if self.channels == 1 { al::AL_FORMAT_MONO16 } else { al::AL_FORMAT_STEREO16 };
                        al::alBufferData(buf, format, slice.as_ptr() as *const _, std::mem::size_of_val(slice) as i32, self.sample_rate as i32);
                        al::alSourceQueueBuffers(self.src_handle, 1, &buf);
                    }
                    *playback_position += take;
                }
            },
            AudioSource::Full { load_position, playback_position, free_buffers, queued_sizes, played_samples, all_buffers } => {
                unsafe {
                    let mut processed = [0i32];
                    al::alGetSourcei(self.src_handle, al::AL_BUFFERS_PROCESSED, processed.as_mut_ptr());
                    for _ in 0..processed[0] {
                        let mut buf = [0u32];
                        al::alSourceUnqueueBuffers(self.src_handle, 1, buf.as_mut_ptr());
                        check_al_error("alSourceUnqueueBuffers");
                        free_buffers.push(buf[0]);
                        if let Some(consumed) = queued_sizes.pop_front() {
                            *played_samples.write() += consumed;
                        }
                    }
                }

                let available = *load_position.read();
                let dat = self.data.read();

                while let Some(buf) = free_buffers.pop() {
                    let remaining = available.saturating_sub(*playback_position);
                    if remaining == 0 { free_buffers.push(buf); break; }
                    let take = remaining.min(FULL_CHUNK_SAMPLES);
                    let slice = &dat[*playback_position..*playback_position + take];
                    unsafe {
                        let format = if self.channels == 1 { al::AL_FORMAT_MONO16 } else { al::AL_FORMAT_STEREO16 };
                        al::alBufferData(buf, format, slice.as_ptr() as *const _, std::mem::size_of_val(slice) as i32, self.sample_rate as i32);
                        al::alSourceQueueBuffers(self.src_handle, 1, &buf);
                        check_al_error("alBufferData/QueueBuffers");
                    }
                    queued_sizes.push_back(take);
                    *playback_position += take;
                }
                drop(dat);
            },
        }

        drop(source);

        if *self.pending_play.read() {
            match self.play() {
                Ok(()) => {
                    tracing::debug!(target: DB_AUDIO, "Playing queued audio");
                    *self.pending_play.write() = false
                },
                Err(PlaybackError::NotReady) => {},
                Err(_) => {},
            }
        }

    }

    pub fn play(&self) -> Result<(), PlaybackError> {
        unsafe {
            let mut state = [0i32];
            al::alGetSourcei(self.src_handle, al::AL_SOURCE_STATE, state.as_mut_ptr());
            let mut queued = [0i32];
            al::alGetSourcei(self.src_handle, al::AL_BUFFERS_QUEUED, queued.as_mut_ptr());
            if state[0] == al::AL_PLAYING {
                return Ok(())
            }
            if queued[0] > 0 {
                al::alSourcePlay(self.src_handle);
                Ok(())
            } else {
                Err(PlaybackError::NotReady)
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        unsafe {
            let mut state = [0];
            al::alGetSourcei(self.src_handle, al::AL_SOURCE_STATE, state.as_mut_ptr());
            state[0] == al::AL_PLAYING
        }
    }
    pub fn position_seconds(&self) -> f32 {
        let source = self.source.read();
        let played_samples = match &*source {
            AudioSource::Full { played_samples, .. } => *played_samples.read(),
            AudioSource::Stream { played_samples, .. } => *played_samples.read(),
        };
        drop(source);

        let mut offset_frames = [0i32];
        unsafe {
            al::alGetSourcei(self.src_handle, al::AL_SAMPLE_OFFSET, offset_frames.as_mut_ptr());
        }

        let played_frames = played_samples / self.channels as usize;
        let total_frames = played_frames + offset_frames[0].max(0) as usize;

        total_frames as f32 / self.sample_rate as f32
    }

    pub fn queue_play(&self) {
        tracing::debug!(target: DB_AUDIO, "Queueing audio to play when ready");
        *self.pending_play.write() = true;
    }

    pub fn pause(&self) {
        *self.pending_play.write() = false;
        unsafe {
            al::alSourcePause(self.src_handle);
        }
    }

    pub fn stop(&self) {
        *self.pending_play.write() = false;
        unsafe {
            al::alSourceStop(self.src_handle);
        }
    }

    /// Volume should be between 0-1
    pub fn set_volume(&self, volume: f32) {
        unsafe {
            al::alSourcef(self.src_handle, al::AL_GAIN, volume);
        }
    }

    pub fn enable_fx(&self) {
        unsafe {
            al::alSourcei(self.src_handle, al::AL_DIRECT_FILTER, self.fx_filter as i32);
        }
    }

    pub fn disable_fx(&self) {
        unsafe {
            al::alSourcei(self.src_handle, al::AL_DIRECT_FILTER, 0);
        }
    }

    pub fn set_speed(&self, speed: f32) {
        unsafe {
            al::alSourcef(self.src_handle, al::AL_PITCH, speed);
        }
    }

    pub fn reset_speed(&self) {
        self.set_speed(1.);
    }

    pub fn seek(&self, target: f32) -> Result<(), PlaybackError> {
        if target < 0.0 {
            return Err(PlaybackError::InvalidSeekPosition(target));
        }
        let target_sample = (target * self.sample_rate as f32) as usize * self.channels as usize;

        let mut source = self.source.write();
        match &mut *source {
            AudioSource::Full { load_position, playback_position, free_buffers, queued_sizes, played_samples, all_buffers } => {
                let loaded = *load_position.read();
                if target_sample > loaded {
                    return Err(PlaybackError::InvalidSeekPosition(target));
                }

                let mut was_playing = [0i32];
                unsafe { al::alGetSourcei(self.src_handle, al::AL_SOURCE_STATE, was_playing.as_mut_ptr()); }
                let was_playing = was_playing[0] == al::AL_PLAYING;

                unsafe {
                    al::alSourceStop(self.src_handle);
                    al::alSourcei(self.src_handle, al::AL_BUFFER, 0);
                    check_al_error("seek: flush buffers");
                }
                free_buffers.clear();
                free_buffers.extend_from_slice(all_buffers);

                queued_sizes.clear();
                *played_samples.write() = target_sample;

                *playback_position = target_sample;
                drop(source);

                self.update();
                if was_playing {
                    self.play()?;
                }
                Ok(())
            }
            AudioSource::Stream { .. } => {
                drop(source);
                self.seek_stream(target)
            },
        }
    }

    fn seek_stream(&self, target: f32) -> Result<(), PlaybackError> {
        let target_sample = (target * self.sample_rate as f32) as usize * self.channels as usize;
        let mut source = self.source.write();
        if let AudioSource::Stream { loaded_ranges, playback_position, seek_target, free_buffers, queued_sizes, played_samples, .. } = &mut *source {
            let already_loaded = Self::contiguous_loaded_end(&loaded_ranges.read(), target_sample) > target_sample;

            unsafe {
                al::alSourceStop(self.src_handle);
                let mut queued = [0i32];
                al::alGetSourcei(self.src_handle, al::AL_BUFFERS_QUEUED, queued.as_mut_ptr());
                for _ in 0..queued[0] {
                    let mut buf = [0u32];
                    al::alSourceUnqueueBuffers(self.src_handle, 1, buf.as_mut_ptr());
                    free_buffers.push(buf[0]);
                }
            }

            queued_sizes.clear();
            *played_samples.write() = target_sample;

            *playback_position = target_sample;
            if !already_loaded {
                *seek_target.write() = Some(target_sample);
            }
        }
        Ok(())
    }

    fn contiguous_loaded_end(ranges: &[(usize, usize)], pos: usize) -> usize {
        ranges.iter().find(|&&(s, e)| s <= pos && pos < e).map(|&(_, e)| e).unwrap_or(pos)
    }

    pub fn mode(&self) -> AudioMode {
        let source = self.source.read();
        match *source {
            AudioSource::Stream { .. } => AudioMode::Stream,
            AudioSource::Full { .. } => AudioMode::Full,
        }
    }

    /// This may be None if the audio did not provide the length as metadata.
    /// If not provided by metadata this is calculated during decoding.
    /// If decoding is required to determine length, this might return a
    /// shorter duration than the audio actually is until done.
    pub fn length_seconds(&self) -> Option<f32> {
        let frames = (*self.sample_count.read())?;
        Some(frames as f32 / self.sample_rate as f32)
    }

    fn load_full(
        data: &Arc<RwLock<Vec<i16>>>,
        loaded: &Arc<RwLock<bool>>,
        load_pos: &Arc<RwLock<usize>>,
        info: &mut AudioInfo,
        sample_count: &Arc<RwLock<Option<usize>>>,
        decode_finished: &Arc<RwLock<bool>>,
    ) -> TaskAction {

        if !{ *loaded.read() } {
            *decode_finished.write() = true;
            return TaskAction::Remove;
        }

        let packet = loop {
            let packet = match info.format_reader.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => {
                    *decode_finished.write() = true;
                    return TaskAction::Remove
                },
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    unimplemented!("why do I have to deal with OGG");
                }
                Err(err) => {
                    tracing::error!(target: DB_AUDIO, "Audio reader encountered an unrecoverable error: {err}");
                    *decode_finished.write() = true;
                    return TaskAction::Remove;
                }
            };
            while !info.format_reader.metadata().is_latest() {
                info.format_reader.metadata().pop();
            }
            if packet.track_id != info.track_id {
                continue;
            }
            break packet;
        };

        match info.decoder.decode(&packet) {
            Ok(buf) => {
                let size = buf.samples_interleaved();
                let mut dat = data.write();
                dat.reserve(size);
                dat.append(&mut vec![i16::MID; size]);
                let end = dat.len();
                let slice = &mut dat[(end-size)..];
                buf.copy_to_slice_interleaved(slice);
                *load_pos.write() += size;
            }
            Err(symphonia::core::errors::Error::IoError(err)) => {
                tracing::warn!(target: DB_AUDIO, "IO Error during decode: {err}");
            }
            Err(symphonia::core::errors::Error::DecodeError(err)) => {
                tracing::warn!(target: DB_AUDIO, "Decode error: {err}")
            }
            Err(err) => {
                tracing::error!(target: DB_AUDIO, "Audio decoder encountered an unrecoverable error: {err}")
            }
        }


        TaskAction::None
    }

    #[allow(clippy::type_complexity)]
    fn spawn_spectrogram_task(
        audio_sys: &AudioSystem,
        data: Arc<RwLock<Vec<i16>>>,
        decode_cursor: Arc<RwLock<usize>>,
        decode_finished: Arc<RwLock<bool>>,
        channels: u16,
        sample_rate: u32,
    ) -> (Arc<RwLock<Vec<f32>>>, Arc<RwLock<usize>>, usize, Arc<RwLock<bool>>, usize) {
        const WINDOW_SIZE: usize = 1024;
        let freq_bins = WINDOW_SIZE / 2;
        const TARGET_COLUMNS_PER_SEC: f32 = 16.0;
        let hop = ((sample_rate as f32 / TARGET_COLUMNS_PER_SEC) as usize).max(WINDOW_SIZE / 4);

        let hann: Vec<f32> = (0..WINDOW_SIZE)
            .map(|i| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / (WINDOW_SIZE - 1) as f32).cos())
            .collect();

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(WINDOW_SIZE);

        let tex_data: Arc<RwLock<Vec<f32>>> = Arc::default();
        let columns_done: Arc<RwLock<usize>> = Arc::default();
        let finished: Arc<RwLock<bool>> = Arc::default();

        let mut state = SpectrogramState {
            data,
            decode_cursor,
            decode_finished,
            columns_done: Arc::clone(&columns_done),
            window_size: WINDOW_SIZE,
            hop,
            channels,
            hann,
            fft,
            tex_data: Arc::clone(&tex_data),
            finished: Arc::clone(&finished),
        };

        audio_sys.add_task(Box::new(move || Self::spectrogram_task_loop(&mut state)));

        (tex_data, columns_done, freq_bins, finished, hop)
    }

    fn decode_task_loop(
        data: &Arc<RwLock<Vec<i16>>>,
        loaded_ranges: &Arc<RwLock<Vec<(usize, usize)>>>,
        decode_cursor: &Arc<RwLock<usize>>,
        seek_target: &Arc<RwLock<Option<usize>>>,
        loaded: &Arc<RwLock<bool>>,
        channels: u16,
        info: &mut AudioInfo,
    ) -> TaskAction {
        if !*loaded.read() { return TaskAction::Remove; }

        if let Some(target_sample) = seek_target.write().take() {
            let target_frame = target_sample / channels as usize;
            let target_time_secs = target_frame as f64 / info.sample_rate as f64;
            let time = symphonia::core::units::Time::try_from_secs_f64(target_time_secs).unwrap();

            match info.format_reader.seek(
                symphonia::core::formats::SeekMode::Accurate,
                symphonia::core::formats::SeekTo::Time {
                    time,
                    track_id: Some(info.track_id),
                },
            ) {
                Ok(seeked) => {
                    let actual_frame = seeked.actual_ts.get().max(0) as usize;
                    *decode_cursor.write() = actual_frame * channels as usize;
                }
                Err(err) => tracing::warn!(target: DB_AUDIO, "seek failed: {err}"),
            }
        }

        let packet = loop {
            let packet = match info.format_reader.next_packet() {
                Ok(Some(p)) => p,
                Ok(None) => return TaskAction::Remove,
                Err(symphonia::core::errors::Error::ResetRequired) => unimplemented!("why do I have to deal with OGG"),
                Err(err) => { tracing::error!(target: DB_AUDIO, "{err}"); return TaskAction::Remove; }
            };
            while !info.format_reader.metadata().is_latest() { info.format_reader.metadata().pop(); }
            if packet.track_id != info.track_id { continue; }
            break packet;
        };

        if let Ok(buf) = info.decoder.decode(&packet) {
            let size = buf.samples_interleaved();
            let start = *decode_cursor.read();
            let end = start + size;

            let mut dat = data.write();
            if dat.len() < end { dat.resize(end, i16::MID); }
            buf.copy_to_slice_interleaved(&mut dat[start..end]);
            drop(dat);

            *decode_cursor.write() = end;
            let mut ranges = loaded_ranges.write();
            ranges.push((start, end));
            Self::merge_ranges(&mut ranges);
        }

        TaskAction::None
    }

    fn spectrogram_task_loop(state: &mut SpectrogramState) -> TaskAction {
        let decoded_samples = *state.decode_cursor.read();
        let decoded_frames = decoded_samples / state.channels as usize;

        let columns_done = *state.columns_done.read();
        let next_window_start = columns_done * state.hop;

        if next_window_start + state.window_size > decoded_frames {
            if *state.decode_finished.read() {
                *state.finished.write() = true;
            }
            return TaskAction::None;
        }

        let data = state.data.read();
        let mut buf: Vec<Complex<f32>> = (0..state.window_size)
            .map(|i| {
                let frame_idx = next_window_start + i;
                let sample_idx = frame_idx * state.channels as usize;
                let s: f32 = (0..state.channels as usize)
                    .map(|c| data[sample_idx + c] as f32 / i16::MAX as f32)
                    .sum::<f32>() / state.channels as f32;
                Complex::new(s * state.hann[i], 0.0)
            })
            .collect();
        drop(data);

        state.fft.process(&mut buf);

        let mag_db_norm: Vec<f32> = buf[..state.window_size / 2]
            .iter()
            .map(|c| {
                let mag = c.norm() / state.window_size as f32;
                let db = 20.0 * mag.max(1e-8).log10();
                ((db - (-120.0)) / 120.0f32).clamp(0.0, 1.0)
            })
            .collect();

        let mut tex = state.tex_data.write();
        let col_height = mag_db_norm.len();
        let col_start = columns_done * col_height;
        if tex.len() < col_start + col_height {
            tex.resize(col_start + col_height, 0.0);
        }
        tex[col_start..col_start + col_height].copy_from_slice(&mag_db_norm);
        drop(tex);

        *state.columns_done.write() = columns_done + 1;

        TaskAction::None
    }

    fn merge_ranges(ranges: &mut Vec<(usize, usize)>) {
        ranges.sort_unstable_by_key(|r| r.0);
        let mut merged: Vec<(usize, usize)> = Vec::with_capacity(ranges.len());
        for &(s, e) in ranges.iter() {
            match merged.last_mut() {
                Some(last) if s <= last.1 => last.1 = last.1.max(e),
                _ => merged.push((s, e)),
            }
        }
        *ranges = merged;
    }

    pub fn spectrogram_uploaded_columns(&self) -> usize {
        self.spectrogram_gl_cache.read().as_ref().map(|c| c.uploaded_columns).unwrap_or(0)
    }

    pub fn spectrogram_synced_coverage(&self) -> f32 {
        let Some(total_frames) = *self.sample_count.read() else { return 1.0 };
        if total_frames == 0 || self.spectrogram_hop == 0 {
            return 1.0;
        }
        let total_columns_full = (total_frames / self.spectrogram_hop).max(1);
        let uploaded = self.spectrogram_uploaded_columns();
        (uploaded as f32 / total_columns_full as f32).min(1.0)
    }

    pub fn spectrogram_coverage(&self) -> f32 {
        let Some(total_frames) = *self.sample_count.read() else { return 1.0 };
        if total_frames == 0 || self.spectrogram_hop == 0 {
            return 1.0;
        }
        let total_columns_full = (total_frames / self.spectrogram_hop).max(1);
        let columns_done = *self.spectrogram_columns_done.read();
        (columns_done as f32 / total_columns_full as f32).min(1.0)
    }

    pub fn get_spectrogram_tex(&self, gl: &glow::Context) -> Option<glow::NativeTexture> {
    let freq_bins = self.spectrogram_freq_bins;
    let columns_done = *self.spectrogram_columns_done.read();
    if columns_done == 0 {
        return None;
    }

        let finished = *self.spectrogram_finished.read();
        let max_tex_size = unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as usize };
        let target_columns = columns_done.min(max_tex_size);

        let mut cache = self.spectrogram_gl_cache.write();

        let pending = match &*cache {
            None => true,
            Some(c) => {
                let new_since_upload = target_columns.saturating_sub(c.uploaded_columns);
                new_since_upload >= SPECTROGRAM_UPLOAD_BATCH || (finished && new_since_upload > 0)
            }
        };
        if !pending {
            return cache.as_ref().map(|c| c.tex);
        }

        let need_grow = match &*cache {
            None => true,
            Some(c) => target_columns > c.capacity_columns,
        };

        if need_grow {
            let capacity_columns = target_columns;
            unsafe {
                if let Some(old) = cache.take() {
                    gl.delete_texture(old.tex);
                }
                let tex = gl.create_texture().expect("failed to create spectrogram texture");
                gl.bind_texture(glow::TEXTURE_2D, Some(tex));
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_BORDER as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_BORDER as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
                gl.tex_image_2d(
                    glow::TEXTURE_2D, 0,
                    glow::R32F as i32,
                    freq_bins as i32, capacity_columns as i32, 0,
                    glow::RED, glow::FLOAT,
                    glow::PixelUnpackData::Slice(None),
                );
                gl.bind_texture(glow::TEXTURE_2D, None);
                *cache = Some(SpectrogramGlCache { tex, uploaded_columns: 0, capacity_columns });
            }
        }

        let cache_ref = cache.as_mut().expect("cache just ensured Some");
        let upload_target = target_columns.min(cache_ref.capacity_columns);

        if cache_ref.uploaded_columns < upload_target {
            let tex_data = self.spectrogram_tex_data.read();
            let start = cache_ref.uploaded_columns;
            let available_cols = tex_data.len() / freq_bins;
            let end = upload_target.min(available_cols);
            if end > start {
                let slice: &[f32] = &tex_data[start * freq_bins..end * freq_bins];
                unsafe {
                    gl.bind_texture(glow::TEXTURE_2D, Some(cache_ref.tex));
                    gl.tex_sub_image_2d(
                        glow::TEXTURE_2D, 0,
                        0, start as i32,
                        freq_bins as i32, (end - start) as i32,
                        glow::RED, glow::FLOAT,
                        glow::PixelUnpackData::Slice(Some(bytemuck::cast_slice(slice))),
                    );
                    gl.bind_texture(glow::TEXTURE_2D, None);
                }
                cache_ref.uploaded_columns = end;
            }
        }

        Some(cache_ref.tex)
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        *self.loaded.write() = false;

        if std::thread::panicking() {
            return;
        }

        if let Some(cache) = self.spectrogram_gl_cache.write().take() {
            unsafe {
                self.gl.delete_texture(cache.tex);
            }
        }

        unsafe {
            al::alSourceStop(self.src_handle);
            al::alSourcei(self.src_handle, al::AL_BUFFER, 0);
            check_al_error("drop: detach buffers");

            al::alDeleteSources(1, &self.src_handle);
            check_al_error("drop: delete source");

            al::alDeleteFilters(1, &self.fx_filter);
            check_al_error("drop: delete filter");
        }

        let mut all_buffers: Vec<u32> = Vec::new();
        {
            let source = self.source.read();
            match &*source {
                AudioSource::Full { all_buffers: bufs, .. } => all_buffers.extend_from_slice(bufs),
                AudioSource::Stream { free_buffers, .. } => all_buffers.extend_from_slice(free_buffers),
            }
        }
        if !all_buffers.is_empty() {
            unsafe {
                al::alDeleteBuffers(all_buffers.len() as i32, all_buffers.as_ptr());
                check_al_error("drop: delete buffers");
            }
        }

        tracing::debug!(target: DB_AUDIO, "Dropped audio resources");
    }
}


