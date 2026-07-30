use std::fmt::Display;
use std::path::Path;
use std::{ptr, thread};
use std::sync::{Arc, mpsc};

use parking_lot::RwLock;
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, TrackType};
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::audio::sample::Sample;

use crate::DB_AUDIO;

pub mod al;

type AudioTask = Box<dyn FnMut() -> TaskAction + Send>;

const FULL_BUFFER_COUNT: usize = 4;
const FULL_CHUNK_SAMPLES: usize = 8192;

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

pub struct AudioSystem {
    pub device: *mut al::ALCdevice,
    pub context: *mut al::ALCcontext,
    pub thread_commands: mpsc::Sender<AudioThreadCommand>,
    pub audio_refs: Vec<Arc<Audio>>,
}

pub struct AudioInfo {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    sample_rate: u32,
    channels: u16,
    sample_count: Option<usize>,
    track_id: u32,
}

impl AudioSystem {
    pub fn new() -> Result<Self, AudioError> {
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
        sections: Arc<RwLock<Vec<AudioLoadState>>>,
        playback_cursor: Arc<RwLock<usize>>,
        background_cursor: Arc<RwLock<usize>>,
    },
    Full {
        load_position: Arc<RwLock<usize>>,
        playback_position: usize,
        free_buffers: Vec<u32>,
    },
}

#[derive(Debug)]
pub struct Audio {
    data: Arc<RwLock<Vec<i16>>>,
    sample_count: Arc<RwLock<Option<usize>>>,
    source: Arc<RwLock<AudioSource>>,
    loaded: Arc<RwLock<bool>>,
    src_handle: u32,
    channels: u16,
    sample_rate: u32,
}

impl Audio {

    pub fn new(audio_sys: &mut AudioSystem, path: &Path, mode: AudioMode) -> Result<Arc<Self>, AudioError> {
        let span = tracing::debug_span!("audio init");
        let _guard = span.enter();
        match mode {
            AudioMode::Stream => {
                let mut audio_info = AudioSystem::open_decoder(path)?;
                let data: Arc<RwLock<Vec<i16>>> = Arc::default();
                let sections: Arc<RwLock<Vec<AudioLoadState>>> = Arc::default();
                let pb_cursor: Arc<RwLock<usize>> = Arc::default();
                let bg_cursor: Arc<RwLock<usize>> = Arc::default();
                let source = Arc::new(RwLock::new(AudioSource::Stream {
                    sections: sections.clone(),
                    playback_cursor: pb_cursor.clone(),
                    background_cursor: bg_cursor.clone(),
                }));
                let loaded = Arc::new(RwLock::new(true));

                let mut src_handle = [0];
                unsafe { al::alGenSources(1, src_handle.as_mut_ptr()); }
                let [src_handle] = src_handle;

                let audio = Arc::new(Self {
                    data: data.clone(),
                    sample_count: Arc::default(),
                    source: source.clone(),
                    loaded: loaded.clone(),
                    src_handle,
                    channels: audio_info.channels,
                    sample_rate: audio_info.sample_rate,
                });

                audio_sys.audio_refs.push(audio.clone());

                let dat = data.clone();
                let sects = sections.clone();
                let cursor = bg_cursor.clone();
                let src = source.clone();
                let lod = loaded.clone();
                audio_sys.add_task(Box::new(move || {
                    Self::background_task_loop(&dat, &sects, &cursor, &src, &lod, &mut audio_info)
                }));
                let cursor = pb_cursor.clone();
                let mut audio_info = AudioSystem::open_decoder(path)?;
                audio_sys.add_task(Box::new(move || {
                    Self::playback_track_task_loop(&data, &sections, &cursor, &bg_cursor, &loaded, &mut audio_info)
                }));
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
                    load_position: load_pos.clone(),
                    playback_position: 0,
                    free_buffers: buffers,
                }));

                let loaded = Arc::new(RwLock::new(true));

                let audio = Arc::new(Self {
                    data: data.clone(),
                    sample_count: Arc::default(),
                    source,
                    loaded: loaded.clone(),
                    src_handle,
                    channels,
                    sample_rate,
                });

                audio_sys.audio_refs.push(audio.clone());

                audio_sys.add_task(Box::new(move || {
                    Self::load_full(&data, &loaded, &load_pos, &mut audio_info)
                }));

                Ok(audio)
            },
        }
    }

    
pub fn update(&self) {
        let mut source = self.source.write();
 
        match &mut *source {
            AudioSource::Stream { sections, playback_cursor, background_cursor } => todo!(),
            AudioSource::Full { load_position, playback_position, free_buffers } => {
                unsafe {
                    let mut processed = [0i32];
                    al::alGetSourcei(self.src_handle, al::AL_BUFFERS_PROCESSED, processed.as_mut_ptr());
                    for _ in 0..processed[0] {
                        let mut buf = [0u32];
                        al::alSourceUnqueueBuffers(self.src_handle, 1, buf.as_mut_ptr());
                        check_al_error("alSourceUnqueueBuffers");
                        free_buffers.push(buf[0]);
                    }
                }
 
                let available = *load_position.read();
                let dat = self.data.read();
 
                while let Some(buf) = free_buffers.pop() {
                    let remaining = available.saturating_sub(*playback_position);
                    if remaining == 0 {
                        free_buffers.push(buf);
                        break;
                    }
 
                    let take = remaining.min(FULL_CHUNK_SAMPLES);
                    let slice = &dat[*playback_position..*playback_position + take];
 
                    unsafe {
                        let format = if self.channels == 1 { al::AL_FORMAT_MONO16 } else { al::AL_FORMAT_STEREO16 };
                        al::alBufferData(
                            buf, format,
                            slice.as_ptr() as *const _,
                            std::mem::size_of_val(slice) as i32,
                            self.sample_rate as i32,
                        );
                        al::alSourceQueueBuffers(self.src_handle, 1, &buf);
                        check_al_error("alBufferData/QueueBuffers");
                    }
 
                    *playback_position += take;
                }
                drop(dat);
 
            },
        }
    }

    pub fn play(&self) {
        unsafe {
            let mut state = [0i32];
            al::alGetSourcei(self.src_handle, al::AL_SOURCE_STATE, state.as_mut_ptr());
            let mut queued = [0i32];
            al::alGetSourcei(self.src_handle, al::AL_BUFFERS_QUEUED, queued.as_mut_ptr());
            if state[0] != al::AL_PLAYING && queued[0] > 0 {
                al::alSourcePlay(self.src_handle);
                check_al_error("alSourcePlay");
            }
        }
    }

    fn get_cursor(task: LoadTask, cursors: &[Arc<RwLock<usize>>; 2]) -> usize {
        unsafe {
            match task {
                LoadTask::Background => *cursors.get_unchecked(0).read(),
                LoadTask::Playback => *cursors.get_unchecked(1).read(),
            }
        }
    }

    fn collapse(
        s_sections: Arc<RwLock<Vec<AudioLoadState>>>,
        cursors: [Arc<RwLock<usize>>; 2],
        sample_count: Option<usize>,
    ) {
        let mut sections = Vec::new();
        let mut sects = s_sections.write();
        std::mem::swap(&mut sections, &mut sects);
        for section in sections.into_iter() {
            if let Some(sect) = sects.last_mut() {
                match (sect, &section) {
                    (AudioLoadState::Empty { start, end }, AudioLoadState::Empty { start: s2, end: e2 }) if *end >= *s2 => {
                        *end = *e2;
                    },
                    (AudioLoadState::Loaded { start, end }, AudioLoadState::Loaded { start: s2, end: e2 }) if *end == *s2 => {
                        *end = *e2;
                    },
                    (AudioLoadState::Loaded { start, end }, AudioLoadState::Loading { start: s2, task }) if *end == *s2 => {
                        let pos = Self::get_cursor(*task, &cursors);
                        *end = pos;
                        sects.push(AudioLoadState::Empty { start: pos, end: usize::MAX });
                    }
                    _ => {
                        if let AudioLoadState::Loading { start, task } = section {
                            let pos = Self::get_cursor(task, &cursors);
                            sects.push(AudioLoadState::Loaded { start, end: pos });
                            sects.push(AudioLoadState::Empty { start: pos, end: usize::MAX });
                        } else {
                            sects.push(section);
                        }
                    }
                }
            } else {
                sects.push(section);
            }
        }
        if let Some(AudioLoadState::Empty { start: _, end }) = sects.last_mut()
            && let Some(count) = sample_count {
            *end = count;
        }
        let mut bg_c = cursors[0].write();
        *bg_c = 0;
    }

    fn load_full(
        data: &Arc<RwLock<Vec<i16>>>,
        loaded: &Arc<RwLock<bool>>,
        load_pos: &Arc<RwLock<usize>>,
        info: &mut AudioInfo,
    ) -> TaskAction {

        if !{ *loaded.read() } {
            return TaskAction::Remove;
        }

        let packet = loop {
            let packet = match info.format_reader.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => return TaskAction::Remove,
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    unimplemented!("why do I have to deal with OGG");
                }
                Err(err) => {
                    tracing::error!(target: DB_AUDIO, "Audio reader encountered an unrecoverable error: {err}");
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

    fn background_task_loop(
        data: &Arc<RwLock<Vec<i16>>>,
        sections: &Arc<RwLock<Vec<AudioLoadState>>>,
        cursor: &Arc<RwLock<usize>>,
        source: &Arc<RwLock<AudioSource>>,
        loaded: &Arc<RwLock<bool>>,
        info: &mut AudioInfo,
    ) -> TaskAction {

        if !{ *loaded.read() } {
            return TaskAction::Remove;
        }

        let packet = loop {
            let packet = match info.format_reader.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => return TaskAction::Remove,
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    unimplemented!("why do I have to deal with OGG");
                }
                Err(err) => {
                    tracing::error!(target: DB_AUDIO, "Audio reader encountered an unrecoverable error: {err}");
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
                let end = dat.len() - 1;
                let slice = &mut dat[(end-size)..];
                buf.copy_to_slice_interleaved(slice);
                *cursor.write() += size;
                // potentially send AL cmd to main thread to buffer data?
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

    fn playback_track_task_loop(
        data: &Arc<RwLock<Vec<i16>>>,
        sections: &Arc<RwLock<Vec<AudioLoadState>>>,
        cursor: &Arc<RwLock<usize>>,
        bg_cursor: &Arc<RwLock<usize>>,
        loaded: &Arc<RwLock<bool>>,
        info: &mut AudioInfo,
    ) -> TaskAction {

        if !{ *loaded.read() } {
            return TaskAction::Remove;
        }

        // this is the task that only runs when playback is not in-time with the background task

        TaskAction::None
    }

}

impl Drop for Audio {
    fn drop(&mut self) {
        *self.loaded.write() = false;
    }
}


