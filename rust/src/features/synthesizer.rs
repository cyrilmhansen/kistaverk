use crate::state::AppState;
use serde_json::{json, Value};
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, Sample, SampleFormat};
use mir_sys::*;
use libc::{c_void, c_int};

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
    stream: Option<Stream>,
    sample_rate: f64,
    // The JIT context must be kept alive while the stream is running
    mir_ctx: Option<MIR_context_t>,
    // Pointer to the current render function: double (*render)(double, double, double)
    render_fn: Arc<std::sync::atomic::AtomicPtr<c_void>>,
    start_time: Instant,
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
            start_time: Instant::now(),
        }
    }

    fn init_audio(&mut self) -> Result<(), String> {
        if self.stream.is_some() {
            return Ok(());
        }

        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or("No output device available")?;
        
        let config = device.default_output_config()
            .map_err(|e| format!("Failed to get default config: {}", e))?;
        
        self.sample_rate = config.sample_rate().0 as f64;
        let channels = config.channels() as usize;

        let render_fn_clone = self.render_fn.clone();
        let start_time = self.start_time;
        let sample_rate = self.sample_rate;

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &render_fn_clone, start_time, sample_rate)
                },
                err_fn,
                None
            ),
            SampleFormat::I16 => device.build_output_stream(
                &config.into(),
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &render_fn_clone, start_time, sample_rate)
                },
                err_fn,
                None
            ),
            SampleFormat::U16 => device.build_output_stream(
                &config.into(),
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &render_fn_clone, start_time, sample_rate)
                },
                err_fn,
                None
            ),
            _ => return Err("Unsupported sample format".to_string()),
        }.map_err(|e| format!("Failed to build stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;
        self.stream = Some(stream);
        Ok(())
    }

    fn stop_audio(&mut self) {
        self.stream = None;
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

            if c2mir_compile(
                ctx,
                &mut options,
                Some(getc_func),
                &mut reader as *mut _ as *mut c_void,
                b"synth.c\0".as_ptr() as *const _,
                ptr::null_mut(),
            ) != 1 {
                return Err("Compilation failed".to_string());
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
            Ok(())
        }
    }
}

// Function pointer signature: double (*)(double, double, double)
type RenderFn = extern "C" fn(f64, f64, f64) -> f64;

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

fn write_data<T>(output: &mut [T], channels: usize, render_fn_ptr: &Arc<std::sync::atomic::AtomicPtr<c_void>>, start_time: Instant, sample_rate: f64)
where
    T: Sample + cpal::FromSample<f32>,
{
    let ptr = render_fn_ptr.load(std::sync::atomic::Ordering::Relaxed);
    if ptr.is_null() {
        for frame in output.chunks_mut(channels) {
            for sample in frame.iter_mut() {
                *sample = cpal::Sample::from_sample(0.0f32);
            }
        }
        return;
    }

    let render: RenderFn = unsafe { std::mem::transmute(ptr) };
    let (p1, p2) = get_params();
    
    // We calculate time relative to start to avoid precision loss over very long periods? 
    // Actually f64 is fine for audio time.
    let now_elapsed = start_time.elapsed().as_secs_f64();
    let dt = 1.0 / sample_rate;
    
    let mut t = now_elapsed;

    for frame in output.chunks_mut(channels) {
        let value = render(t, p1, p2);
        let sample_val_f32 = value as f32; // Clip? cpal handles some casting, but clipping is good practice.
        let sample_val = cpal::Sample::from_sample(sample_val_f32);
        for sample in frame.iter_mut() {
            *sample = sample_val;
        }
        t += dt;
    }
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
