use beatcraft_audiolib::{AlInterface, al};


type GenFn = unsafe extern "C" fn(al::ALsizei, *mut al::ALuint);

type SetF = unsafe extern "C" fn(al::ALuint, al::ALenum, al::ALfloat);
type SetI = unsafe extern "C" fn(al::ALuint, al::ALenum, al::ALint);

type GetI = unsafe extern "C" fn(al::ALuint, al::ALenum, *mut al::ALint);

type Unqueue = unsafe extern "C" fn(al::ALuint, al::ALsizei, *mut al::ALuint);
type Queue = unsafe extern "C" fn(al::ALuint, al::ALsizei, *const al::ALuint);

type BufData = unsafe extern "C" fn(al::ALuint, al::ALenum, *const al::ALvoid, al::ALsizei, al::ALsizei);

type OpFn = unsafe extern "C" fn(al::ALuint);

type DeleteFn = unsafe extern "C" fn(al::ALsizei, *const al::ALuint);

#[derive(Copy, Clone)]
pub struct JavaAlLink {
    gen_buffers: GenFn,
    gen_sources: GenFn,
    gen_filters: GenFn,
    filter_i: SetI,
    filter_f: SetF,
    get_source_i: GetI,
    source_unqueue_buffers: Unqueue,
    buffer_data: BufData,
    source_queue_buffers: Queue,
    source_play: OpFn,
    source_pause: OpFn,
    source_stop: OpFn,
    source_i: SetI,
    source_f: SetF,
    delete_sources: DeleteFn,
    delete_filters: DeleteFn,
    delete_buffers: DeleteFn,
}

static mut JAVA_AL: Option<JavaAlLink> = None;

/// # Safety
/// the provided links must correctly bind to all the expected functions
pub unsafe fn bind_al(links: JavaAlLink) {
    unsafe {
        JAVA_AL = Some(links)
    }
}

macro_rules! j_al {
    [$f:tt ( $($a:tt),* )] => {
        unsafe {
            if let Some(java_al) = &JAVA_AL {
                (java_al.$f)( $($a),* )
            }
        }
    };
}

impl AlInterface for JavaAlLink {
    unsafe fn gen_buffers(n: al::ALsizei, buffers: *mut al::ALuint) {
        j_al![ gen_buffers(n, buffers) ]
    }

    unsafe fn gen_sources(n: al::ALsizei, buffers: *mut al::ALuint) {
        j_al![ gen_sources(n, buffers) ]
    }

    unsafe fn gen_filters(n: al::ALsizei, buffers: *mut al::ALuint) {
        j_al![ gen_filters(n, buffers) ]
    }

    unsafe fn filter_i(filter: al::ALuint, param: al::ALenum, i_value: al::ALint) {
        j_al![ filter_i(filter, param, i_value) ]
    }

    unsafe fn filter_f(filter: al::ALuint, param: al::ALenum, f_value: al::ALfloat) {
        j_al![ filter_f(filter, param, f_value) ]
    }

    unsafe fn get_source_i(source: al::ALuint, param: al::ALenum, value: *mut al::ALint) {
        j_al![ get_source_i(source, param, value) ]
    }

    unsafe fn source_unqueue_buffers(source: al::ALuint, nb: al::ALsizei, buffers: *mut al::ALuint) {
        j_al![ source_unqueue_buffers(source, nb, buffers) ]
    }

    unsafe fn buffer_data(buffer: al::ALuint, format: al::ALenum, data: *const al::ALvoid, size: al::ALsizei, samplerate: al::ALsizei) {
        j_al![ buffer_data(buffer, format, data, size, samplerate) ]
    }

    unsafe fn source_queue_buffers(source: al::ALuint, nb: al::ALsizei, buffers: *const al::ALuint) {
        j_al![ source_queue_buffers(source, nb, buffers) ]
    }

    unsafe fn source_play(source: al::ALuint) {
        j_al![ source_play(source) ]
    }

    unsafe fn source_pause(source: al::ALuint) {
        j_al![ source_pause(source) ]
    }

    unsafe fn source_stop(source: al::ALuint) {
        j_al![ source_stop(source) ]
    }

    unsafe fn source_i(source: al::ALuint, param: al::ALenum, value: al::ALint) {
        j_al![ source_i(source, param, value) ]
    }

    unsafe fn source_f(source: al::ALuint, param: al::ALenum, value: al::ALfloat) {
        j_al![ source_f(source, param, value) ]
    }

    unsafe fn delete_sources(n: al::ALsizei, sources: *const al::ALuint) {
        j_al![ delete_sources(n, sources) ]
    }

    unsafe fn delete_filters(n: al::ALsizei, filters: *const al::ALuint) {
        j_al![ delete_filters(n, filters) ]
    }

    unsafe fn delete_buffers(n: al::ALsizei, buffers: *const al::ALuint) {
        j_al![ delete_buffers(n, buffers) ]
    }
}


