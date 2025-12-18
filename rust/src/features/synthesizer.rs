use crate::state::AppState;
use serde_json::{json, Value};
use std::ffi::{CStr, CString};
use std::io::Read;
use std::os::unix::io::FromRawFd;
use std::ptr;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::Ordering;
use mir_sys::*;
use libc::{c_void, c_int};

#[cfg(target_os = "android")]
mod aaudio {
    #![allow(non_camel_case_types, non_snake_case)]

    use libc::{c_void, c_char};

    pub enum AAudioStream {}
    pub enum AAudioStreamBuilder {}

    pub type AAudioStream_dataCallback = Option<
        unsafe extern "C" fn(
            stream: *mut AAudioStream,
            userData: *mut c_void,
            audioData: *mut c_void,
            numFrames: i32,
        ) -> i32,
    >;
    pub type AAudioStream_errorCallback =
        Option<unsafe extern "C" fn(stream: *mut AAudioStream, userData: *mut c_void, error: i32)>;

    pub const AAUDIO_OK: i32 = 0;
    pub const AAUDIO_DIRECTION_OUTPUT: i32 = 0;
    pub const AAUDIO_SHARING_MODE_SHARED: i32 = 1;
    pub const AAUDIO_PERFORMANCE_MODE_LOW_LATENCY: i32 = 12;
    pub const AAUDIO_FORMAT_PCM_FLOAT: i32 = 2;
    pub const AAUDIO_CALLBACK_RESULT_CONTINUE: i32 = 0;

    #[link(name = "aaudio")]
    extern "C" {
        pub fn AAudio_createStreamBuilder(builder: *mut *mut AAudioStreamBuilder) -> i32;
        pub fn AAudioStreamBuilder_delete(builder: *mut AAudioStreamBuilder) -> i32;
        pub fn AAudioStreamBuilder_setDirection(builder: *mut AAudioStreamBuilder, direction: i32);
        pub fn AAudioStreamBuilder_setPerformanceMode(builder: *mut AAudioStreamBuilder, mode: i32);
        pub fn AAudioStreamBuilder_setSharingMode(builder: *mut AAudioStreamBuilder, mode: i32);
        pub fn AAudioStreamBuilder_setChannelCount(builder: *mut AAudioStreamBuilder, channels: i32);
        pub fn AAudioStreamBuilder_setSampleRate(builder: *mut AAudioStreamBuilder, sample_rate: i32);
        pub fn AAudioStreamBuilder_setFormat(builder: *mut AAudioStreamBuilder, format: i32);
        pub fn AAudioStreamBuilder_setDataCallback(
            builder: *mut AAudioStreamBuilder,
            callback: AAudioStream_dataCallback,
            userData: *mut c_void,
        );
        pub fn AAudioStreamBuilder_setErrorCallback(
            builder: *mut AAudioStreamBuilder,
            callback: AAudioStream_errorCallback,
            userData: *mut c_void,
        );
        pub fn AAudioStreamBuilder_openStream(
            builder: *mut AAudioStreamBuilder,
            stream: *mut *mut AAudioStream,
        ) -> i32;
        pub fn AAudioStream_requestStart(stream: *mut AAudioStream) -> i32;
        pub fn AAudioStream_requestStop(stream: *mut AAudioStream) -> i32;
        pub fn AAudioStream_close(stream: *mut AAudioStream) -> i32;
        pub fn AAudioStream_getSampleRate(stream: *mut AAudioStream) -> i32;
        pub fn AAudioStream_getChannelCount(stream: *mut AAudioStream) -> i32;
        pub fn AAudioStream_getFramesPerBurst(stream: *mut AAudioStream) -> i32;
        pub fn AAudioStream_getBufferSizeInFrames(stream: *mut AAudioStream) -> i32;
        pub fn AAudioStream_setBufferSizeInFrames(stream: *mut AAudioStream, numFrames: i32) -> i32;
        pub fn AAudio_convertResultToText(result: i32) -> *const c_char;
    }
}

// Declare C math functions manually
extern "C" {
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
    fn tan(x: f64) -> f64;
    fn exp(x: f64) -> f64;
    fn log(x: f64) -> f64;
    fn pow(x: f64, y: f64) -> f64;
    fn fmod(x: f64, y: f64) -> f64;
}

// Global synthesizer engine instance
static SYNTH_ENGINE: OnceLock<Mutex<SynthesizerEngine>> = OnceLock::new();

pub struct SynthesizerEngine {
    #[cfg(target_os = "android")]
    stream: Option<AAudioStreamHandle>,
    #[cfg(not(target_os = "android"))]
    stream: Option<()>,
    sample_rate: f64,
    // The JIT context must be kept alive while the stream is running
    mir_ctx: Option<MIR_context_t>,
    // Pointer to the current render function: double (*render)(double, double, double)
    render_fn: Arc<std::sync::atomic::AtomicPtr<c_void>>,
}

// Safety: MIR context is raw pointer, but we manage access. 
// The render_fn is atomic and can be shared.
unsafe impl Send for SynthesizerEngine {}
unsafe impl Sync for SynthesizerEngine {}

impl SynthesizerEngine {
    fn new() -> Self {
        Self {
            stream: None,
            sample_rate: 44100.0,
            mir_ctx: None,
            render_fn: Arc::new(std::sync::atomic::AtomicPtr::new(ptr::null_mut())),
        }
    }

    fn init_audio(&mut self) -> Result<(), String> {
        #[cfg(target_os = "android")]
        {
            if self.stream.is_some() {
                return Ok(());
            }

            let mut builder: *mut aaudio::AAudioStreamBuilder = ptr::null_mut();
            let result = unsafe { aaudio::AAudio_createStreamBuilder(&mut builder) };
            aaudio_result(result)?;

            let mut callback_data = Box::new(AAudioCallbackData {
                render_fn: self.render_fn.clone(),
                sample_rate: self.sample_rate,
                channels: 2,
                buffer: Arc::new(Mutex::new(LoopBuffer::default())),
            });

            unsafe {
                aaudio::AAudioStreamBuilder_setDirection(builder, aaudio::AAUDIO_DIRECTION_OUTPUT);
                aaudio::AAudioStreamBuilder_setPerformanceMode(
                    builder,
                    aaudio::AAUDIO_PERFORMANCE_MODE_LOW_LATENCY,
                );
                aaudio::AAudioStreamBuilder_setSharingMode(builder, aaudio::AAUDIO_SHARING_MODE_SHARED);
                aaudio::AAudioStreamBuilder_setChannelCount(builder, 2);
                // Use 0 to let the system pick the optimal sample rate.
                aaudio::AAudioStreamBuilder_setSampleRate(builder, 0);
                aaudio::AAudioStreamBuilder_setFormat(builder, aaudio::AAUDIO_FORMAT_PCM_FLOAT);
                aaudio::AAudioStreamBuilder_setDataCallback(
                    builder,
                    Some(aaudio_data_callback),
                    &mut *callback_data as *mut _ as *mut c_void,
                );
                aaudio::AAudioStreamBuilder_setErrorCallback(
                    builder,
                    Some(aaudio_error_callback),
                    &mut *callback_data as *mut _ as *mut c_void,
                );
            }

            let mut stream: *mut aaudio::AAudioStream = ptr::null_mut();
            let result = unsafe { aaudio::AAudioStreamBuilder_openStream(builder, &mut stream) };
            unsafe {
                aaudio::AAudioStreamBuilder_delete(builder);
            }
            aaudio_result(result)?;

            let actual_sample_rate = unsafe { aaudio::AAudioStream_getSampleRate(stream) };
            let actual_channels = unsafe { aaudio::AAudioStream_getChannelCount(stream) };

            if actual_sample_rate > 0 {
                self.sample_rate = actual_sample_rate as f64;
                callback_data.sample_rate = self.sample_rate;
            }
            if actual_channels > 0 {
                callback_data.channels = actual_channels as usize;
            }

            let burst = unsafe { aaudio::AAudioStream_getFramesPerBurst(stream) };
            if burst > 0 {
                let target = burst * 2;
                let current = unsafe { aaudio::AAudioStream_getBufferSizeInFrames(stream) };
                if current > 0 && current < target {
                    let _ = unsafe { aaudio::AAudioStream_setBufferSizeInFrames(stream, target) };
                }
            }

            let loop_data = generate_loop_buffer(
                &callback_data.render_fn,
                callback_data.sample_rate,
                callback_data.channels,
                LOOP_SECONDS,
            );
            if let Ok(mut buf) = callback_data.buffer.lock() {
                *buf = LoopBuffer::new(loop_data, callback_data.channels);
            }

            let result = unsafe { aaudio::AAudioStream_requestStart(stream) };
            aaudio_result(result)?;

            self.stream = Some(AAudioStreamHandle {
                stream,
                callback_data,
            });
            Ok(())
        }

        #[cfg(not(target_os = "android"))]
        {
            let _ = self;
            Err("AAudio output is only available on Android".to_string())
        }
    }

    fn stop_audio(&mut self) {
        #[cfg(target_os = "android")]
        {
            if let Some(handle) = self.stream.take() {
                unsafe {
                    let _ = aaudio::AAudioStream_requestStop(handle.stream);
                    let _ = aaudio::AAudioStream_close(handle.stream);
                }
            }
        }

        #[cfg(not(target_os = "android"))]
        {
            self.stream = None;
        }
    }

    fn compile_and_load(&mut self, source: &str) -> Result<(), String> {
        unsafe {
            // Cleanup previous context if exists
            if let Some(ctx) = self.mir_ctx {
                MIR_gen_finish(ctx);
                MIR_finish(ctx);
                self.mir_ctx = None;
                // Atomic swap to null to silence audio temporarily
                self.render_fn.store(ptr::null_mut(), std::sync::atomic::Ordering::Relaxed);
            }

            // Init new MIR context
            let ctx = _MIR_init(ptr::null_mut(), ptr::null_mut());
            if ctx.is_null() {
                return Err("Failed to init MIR".to_string());
            }
            self.mir_ctx = Some(ctx);

            MIR_gen_init(ctx);
            MIR_gen_set_optimize_level(ctx, 1);
            c2mir_init(ctx);

            // Register math functions
            MIR_load_external(ctx, CString::new("sin").unwrap().as_ptr(), sin as *mut c_void);
            MIR_load_external(ctx, CString::new("cos").unwrap().as_ptr(), cos as *mut c_void);
            MIR_load_external(ctx, CString::new("tan").unwrap().as_ptr(), tan as *mut c_void);
            MIR_load_external(ctx, CString::new("exp").unwrap().as_ptr(), exp as *mut c_void);
            MIR_load_external(ctx, CString::new("log").unwrap().as_ptr(), log as *mut c_void);
            MIR_load_external(ctx, CString::new("pow").unwrap().as_ptr(), pow as *mut c_void);
            MIR_load_external(ctx, CString::new("fmod").unwrap().as_ptr(), fmod as *mut c_void);

            // Compile
            let full_source = format!(
                "#include <math.h>\n{}
", 
                source
            );
            
            // We use a simple struct to feed string to c2mir
            struct StringReader { data: Vec<u8>, cursor: usize }
            unsafe extern "C" fn getc_func(data: *mut c_void) -> c_int {
                let reader = &mut *(data as *mut StringReader);
                if reader.cursor < reader.data.len() {
                    let byte = reader.data[reader.cursor];
                    reader.cursor += 1;
                    byte as c_int
                } else { -1 }
            }

            let mut reader = StringReader {
                data: full_source.bytes().collect(),
                cursor: 0,
            };
            let mut options: c2mir_options = std::mem::zeroed();
            let mut pipe_fds = [0; 2];
            if libc::pipe(pipe_fds.as_mut_ptr()) != 0 {
                return Err("Compilation failed: pipe_failed".to_string());
            }
            let read_fd = pipe_fds[0];
            let write_fd = pipe_fds[1];
            let original_stdout = libc::dup(libc::STDOUT_FILENO);
            let original_stderr = libc::dup(libc::STDERR_FILENO);
            if original_stdout < 0 || original_stderr < 0 {
                libc::close(read_fd);
                libc::close(write_fd);
                return Err("Compilation failed: dup_failed".to_string());
            }
            libc::dup2(write_fd, libc::STDOUT_FILENO);
            libc::dup2(write_fd, libc::STDERR_FILENO);
            libc::close(write_fd);

            let compile_ok = c2mir_compile(
                ctx,
                &mut options,
                Some(getc_func),
                &mut reader as *mut _ as *mut c_void,
                b"synth.c\0".as_ptr() as *const _,
                ptr::null_mut(),
            ) == 1;

            libc::dup2(original_stdout, libc::STDOUT_FILENO);
            libc::dup2(original_stderr, libc::STDERR_FILENO);
            libc::close(original_stdout);
            libc::close(original_stderr);

            let mut compiler_output = String::new();
            {
                let mut file = std::fs::File::from_raw_fd(read_fd);
                let _ = file.read_to_string(&mut compiler_output);
            }

            if !compile_ok {
                let msg = if compiler_output.trim().is_empty() {
                    "Compilation failed".to_string()
                } else {
                    format!("Compilation failed:\n{}", compiler_output.trim())
                };
                return Err(msg);
            }

            // Find 'render' function
            let module_list = MIR_get_module_list(ctx);
            let module = (*module_list).tail;
            if module.is_null() { return Err("No module generated".to_string()); }
            
            MIR_load_module(ctx, module);
            MIR_link(ctx, Some(MIR_set_gen_interface), None);

            let func_name = CString::new("render").unwrap();
            let mut item = (*module).items.head;
            let mut render_ptr = ptr::null_mut();

            while !item.is_null() {
                if (*item).item_type == MIR_item_type_t_MIR_func_item {
                    let name = CStr::from_ptr(MIR_item_name(ctx, item));
                    if name == func_name.as_c_str() {
                        render_ptr = MIR_gen(ctx, item);
                        break;
                    }
                }
                item = (*item).item_link.next;
            }

            if render_ptr.is_null() {
                return Err("Function 'double render(double t, double p1, double p2)' not found".to_string());
            }

            // Swap the function pointer
            self.render_fn.store(render_ptr, std::sync::atomic::Ordering::Relaxed);
            self.refresh_loop_buffer();
            Ok(())
        }
    }

    fn refresh_loop_buffer(&mut self) {
        #[cfg(target_os = "android")]
        {
            if let Some(handle) = &self.stream {
                let loop_data = generate_loop_buffer(
                    &handle.callback_data.render_fn,
                    handle.callback_data.sample_rate,
                    handle.callback_data.channels,
                    LOOP_SECONDS,
                );
                if let Ok(mut buf) = handle.callback_data.buffer.lock() {
                    *buf = LoopBuffer::new(loop_data, handle.callback_data.channels);
                }
            }
        }
    }
}

// Function pointer signature: double (*)(double, double, double)
type RenderFn = extern "C" fn(f64, f64, f64) -> f64;

#[cfg(target_os = "android")]
struct AAudioCallbackData {
    render_fn: Arc<std::sync::atomic::AtomicPtr<c_void>>,
    sample_rate: f64,
    channels: usize,
    buffer: Arc<Mutex<LoopBuffer>>,
}

#[cfg(target_os = "android")]
struct AAudioStreamHandle {
    stream: *mut aaudio::AAudioStream,
    callback_data: Box<AAudioCallbackData>,
}

#[cfg(target_os = "android")]
fn aaudio_result(result: i32) -> Result<(), String> {
    if result == aaudio::AAUDIO_OK {
        return Ok(());
    }
    let text = unsafe {
        let ptr = aaudio::AAudio_convertResultToText(result);
        if ptr.is_null() {
            "unknown error".to_string()
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    };
    Err(format!("AAudio error {}: {}", result, text))
}

#[cfg(target_os = "android")]
unsafe extern "C" fn aaudio_data_callback(
    _stream: *mut aaudio::AAudioStream,
    user_data: *mut c_void,
    audio_data: *mut c_void,
    num_frames: i32,
) -> i32 {
    if user_data.is_null() || audio_data.is_null() || num_frames <= 0 {
        return aaudio::AAUDIO_CALLBACK_RESULT_CONTINUE;
    }

    let data = &mut *(user_data as *mut AAudioCallbackData);
    let channels = data.channels.max(1);
    let frames = num_frames as usize;
    let samples = frames.saturating_mul(channels);
    let output = std::slice::from_raw_parts_mut(audio_data as *mut f32, samples);
    if let Ok(mut buf) = data.buffer.try_lock() {
        buf.fill_output(output);
    } else {
        for sample in output.iter_mut() {
            *sample = 0.0;
        }
    }
    aaudio::AAUDIO_CALLBACK_RESULT_CONTINUE
}

#[cfg(target_os = "android")]
unsafe extern "C" fn aaudio_error_callback(
    _stream: *mut aaudio::AAudioStream,
    _user_data: *mut c_void,
    error: i32,
) {
    let text = {
        let ptr = aaudio::AAudio_convertResultToText(error);
        if ptr.is_null() {
            "unknown error".to_string()
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    };
    eprintln!("AAudio stream error {}: {}", error, text);
}

// Global parameters accessible by audio thread
// Note: In a real robust engine these should be atomic or passed via ringbuf.
// For this MVP, we will use AtomicU64 to store f64 bits
static PARAM1: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PARAM2: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn set_params(p1: f64, p2: f64) {
    PARAM1.store(p1.to_bits(), std::sync::atomic::Ordering::Relaxed);
    PARAM2.store(p2.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

fn get_params() -> (f64, f64) {
    let p1 = f64::from_bits(PARAM1.load(std::sync::atomic::Ordering::Relaxed));
    let p2 = f64::from_bits(PARAM2.load(std::sync::atomic::Ordering::Relaxed));
    (p1, p2)
}

const LOOP_SECONDS: f64 = 2.0;

#[derive(Default)]
struct LoopBuffer {
    data: Vec<f32>,
    cursor: usize,
    channels: usize,
}

impl LoopBuffer {
    fn new(data: Vec<f32>, channels: usize) -> Self {
        Self {
            data,
            cursor: 0,
            channels,
        }
    }

    fn fill_output(&mut self, output: &mut [f32]) {
        if self.data.is_empty() {
            for sample in output.iter_mut() {
                *sample = 0.0;
            }
            return;
        }

        for sample in output.iter_mut() {
            *sample = self.data[self.cursor];
            self.cursor += 1;
            if self.cursor >= self.data.len() {
                self.cursor = 0;
            }
        }
    }
}

fn generate_loop_buffer(
    render_fn_ptr: &Arc<std::sync::atomic::AtomicPtr<c_void>>,
    sample_rate: f64,
    channels: usize,
    seconds: f64,
) -> Vec<f32> {
    if sample_rate <= 0.0 {
        return Vec::new();
    }

    let frames = (sample_rate * seconds).max(1.0) as usize;
    let samples = frames.saturating_mul(channels.max(1));
    let mut data = vec![0.0f32; samples];

    let ptr = render_fn_ptr.load(Ordering::Relaxed);
    if ptr.is_null() {
        return data;
    }

    let render: RenderFn = unsafe { std::mem::transmute(ptr) };
    let (p1, p2) = get_params();
    let dt = 1.0 / sample_rate;
    let mut t = 0.0;

    for frame_idx in 0..frames {
        let value = render(t, p1, p2) as f32;
        let base = frame_idx * channels.max(1);
        for ch in 0..channels.max(1) {
            data[base + ch] = value;
        }
        t += dt;
    }

    data
}

pub fn render_synthesizer_screen(state: &AppState) -> Value {
    let synth = &state.synthesizer;
    
    let mut components = Vec::new();
    
    components.push(json!({
        "type": "Text",
        "text": "Algorithmic Synthesizer",
        "size": 24.0,
        "bold": true,
        "margin_bottom": 16.0
    }));

    components.push(json!({
        "type": "Text",
        "text": "Define 'double render(double t, double p1, double p2)'",
        "size": 12.0,
        "margin_bottom": 8.0
    }));

    components.push(json!({
        "type": "TextInput",
        "bind_key": "synthesizer_source",
        "text": synth.source_code,
        "hint": "double render(double t, double p1, double p2) { return sin(t*440*6.28); }",
        "single_line": false,
        "max_lines": 15,
        "margin_bottom": 12.0
    }));

    components.push(json!({
        "type": "Row",
        "children": [
            {
                "type": "TextInput",
                "bind_key": "synthesizer_p1",
                "text": synth.param1,
                "hint": "Param 1 (e.g. Freq)",
                "single_line": true,
                "margin_right": 8.0
            },
            {
                "type": "TextInput",
                "bind_key": "synthesizer_p2",
                "text": synth.param2,
                "hint": "Param 2 (e.g. Mod)",
                "single_line": true
            }
        ],
        "margin_bottom": 12.0
    }));

    if let Some(status) = &synth.compilation_status {
        components.push(json!({
            "type": "Text",
            "text": status,
            "color": if synth.compilation_error { "red" } else { "green" },
            "size": 12.0,
            "margin_bottom": 12.0
        }));
    }

    components.push(json!({
        "type": "Row",
        "children": [
            {
                "type": "Button",
                "text": if synth.is_playing { "Stop" } else { "Play" },
                "action": if synth.is_playing { "synthesizer_stop" } else { "synthesizer_play" },
                "margin_right": 8.0
            },
            {
                "type": "Button",
                "text": "Apply / Compile",
                "action": "synthesizer_apply",
                "margin_right": 8.0
            },
            {
                "type": "Button",
                "text": "Load Example",
                "action": "synthesizer_example"
            }
        ]
    }));

    json!({
        "type": "Column",
        "children": components,
        "padding": 16
    })
}

pub fn handle_synthesizer_actions(state: &mut AppState, action: crate::router::Action) -> Option<Value> {
    use crate::router::Action::*;
    
    // Update params
    let p1 = state.synthesizer.param1.parse::<f64>().unwrap_or(0.0);
    let p2 = state.synthesizer.param2.parse::<f64>().unwrap_or(0.0);
    set_params(p1, p2);

    match action {
        SynthesizerScreen => {
            state.push_screen(crate::state::Screen::Synthesizer);
            if state.synthesizer.source_code.is_empty() {
                state.synthesizer.source_code = r#"
double render(double t, double p1, double p2) {
    // Simple Sine Wave
    // p1: Frequency (default 440)
    // p2: Amplitude (0.0 to 1.0)
    double freq = p1 > 0 ? p1 : 440.0;
    double amp = p2 > 0 ? p2 : 0.5;
    return sin(t * freq * 6.28318) * amp;
}
"#.trim().to_string();
                state.synthesizer.param1 = "440.0".to_string();
                state.synthesizer.param2 = "0.5".to_string();
            }
            Some(render_synthesizer_screen(state))
        }
        SynthesizerPlay => {
            let engine = SYNTH_ENGINE.get_or_init(|| Mutex::new(SynthesizerEngine::new()));
            if let Ok(mut eng) = engine.lock() {
                // Ensure code is compiled if not already
                if eng.mir_ctx.is_none() {
                     if let Err(e) = eng.compile_and_load(&state.synthesizer.source_code) {
                         state.synthesizer.compilation_status = Some(e);
                         state.synthesizer.compilation_error = true;
                         return Some(render_synthesizer_screen(state));
                     }
                }
                
                match eng.init_audio() {
                    Ok(_) => {
                        state.synthesizer.is_playing = true;
                        state.synthesizer.compilation_status = Some("Playing...".to_string());
                        state.synthesizer.compilation_error = false;
                    },
                    Err(e) => {
                        state.synthesizer.compilation_status = Some(e);
                        state.synthesizer.compilation_error = true;
                    }
                }
            }
            Some(render_synthesizer_screen(state))
        }
        SynthesizerStop => {
            let engine = SYNTH_ENGINE.get_or_init(|| Mutex::new(SynthesizerEngine::new()));
            if let Ok(mut eng) = engine.lock() {
                eng.stop_audio();
            }
            state.synthesizer.is_playing = false;
            state.synthesizer.compilation_status = Some("Stopped.".to_string());
            state.synthesizer.compilation_error = false;
            Some(render_synthesizer_screen(state))
        }
        SynthesizerApply => {
            let engine = SYNTH_ENGINE.get_or_init(|| Mutex::new(SynthesizerEngine::new()));
            if let Ok(mut eng) = engine.lock() {
                match eng.compile_and_load(&state.synthesizer.source_code) {
                    Ok(_) => {
                        state.synthesizer.compilation_status = Some("Compiled & Updated!".to_string());
                        state.synthesizer.compilation_error = false;
                    },
                    Err(e) => {
                        state.synthesizer.compilation_status = Some(e);
                        state.synthesizer.compilation_error = true;
                    }
                }
            }
            Some(render_synthesizer_screen(state))
        }
        SynthesizerUpdateCode { source } => {
            state.synthesizer.source_code = source;
            // Don't auto-compile on every keystroke, wait for Apply
            Some(render_synthesizer_screen(state))
        }
        SynthesizerLoadExample => {
            state.synthesizer.source_code = r#"
double render(double t, double p1, double p2) {
    // FM Synthesis
    // p1: Carrier Freq
    // p2: Mod Index
    double freq = p1 > 0 ? p1 : 220.0;
    double mod = p2 > 0 ? p2 : 5.0;
    
    double modulator = sin(t * freq * 0.5 * 6.28);
    return sin(t * freq * 6.28 + modulator * mod) * 0.5;
}
"#.trim().to_string();
            state.synthesizer.param1 = "220.0".to_string();
            state.synthesizer.param2 = "5.0".to_string();
            Some(render_synthesizer_screen(state))
        }
        _ => None,
    }
}
