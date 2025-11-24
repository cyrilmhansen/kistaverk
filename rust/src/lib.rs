mod features;
mod state;
mod ui;
use features::file_info::{file_info_from_fd, file_info_from_path};
use features::hashes::{handle_hash_action, HashAlgo};
use features::kotlin_image::{
    handle_output_dir as handle_kotlin_image_output_dir,
    handle_result as handle_kotlin_image_result, handle_screen_entry as handle_kotlin_image_screen,
    parse_image_target, render_kotlin_image_screen, ImageConversionResult, ImageTarget,
};
use features::text_tools::{handle_text_action, render_text_tools_screen, TextAction};
use features::{render_menu, Feature};
use ui::{Button as UiButton, Column as UiColumn, Grid as UiGrid, Progress as UiProgress, Text as UiText};

use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use serde::Deserialize;
use serde_json::{json, Value};
use state::{AppState, Screen};
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    os::unix::io::{FromRawFd, RawFd},
    ptr,
    sync::Mutex,
};

static STATE: Mutex<AppState> = Mutex::new(AppState::new());

#[derive(Deserialize)]
struct Command {
    action: String,
    path: Option<String>,
    fd: Option<i32>,
    error: Option<String>,
    target: Option<String>,
    result_path: Option<String>,
    result_size: Option<String>,
    result_format: Option<String>,
    output_dir: Option<String>,
    bindings: Option<HashMap<String, String>>,
    loading_only: Option<bool>,
    snapshot: Option<String>,
}

#[derive(Debug)]
enum Action {
    Init,
    Reset,
    Back,
    ShaderDemo,
    LoadShader {
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
    },
    KotlinImageScreen(ImageTarget),
    KotlinImageResult {
        target: Option<ImageTarget>,
        result: ImageConversionResult,
    },
    KotlinImageOutputDir {
        target: Option<ImageTarget>,
        output_dir: Option<String>,
    },
    Hash {
        algo: HashAlgo,
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
        loading_only: bool,
    },
    FileInfo {
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
    },
    FileInfoScreen,
    TextToolsScreen {
        bindings: HashMap<String, String>,
    },
    TextTools {
        action: TextAction,
        bindings: HashMap<String, String>,
    },
    ProgressDemoScreen,
    ProgressDemoStart {
        loading_only: bool,
    },
    ProgressDemoFinish,
    Increment,
    Snapshot,
    Restore { snapshot: String },
}

struct FdHandle(Option<i32>);

impl FdHandle {
    fn new(fd: Option<i32>) -> Self {
        Self(fd)
    }

    fn take(&mut self) -> Option<i32> {
        self.0.take()
    }
}

impl Drop for FdHandle {
    fn drop(&mut self) {
        if let Some(fd) = self.0.take() {
            unsafe { File::from_raw_fd(fd as RawFd) };
        }
    }
}

fn parse_action(command: Command) -> Result<Action, String> {
    let Command {
        action,
        path,
        fd,
        error,
        target,
        result_path,
        result_size,
        result_format,
        output_dir,
        bindings,
        loading_only,
        snapshot,
    } = command;

    let bindings = bindings.unwrap_or_default();
    let loading_only = loading_only.unwrap_or(false);

    match action.as_str() {
        "init" => Ok(Action::Init),
        "reset" => Ok(Action::Reset),
        "back" => Ok(Action::Back),
        "shader_demo" => Ok(Action::ShaderDemo),
        "load_shader_file" => Ok(Action::LoadShader { path, fd, error }),
        "kotlin_image_screen_webp" => Ok(Action::KotlinImageScreen(ImageTarget::Webp)),
        "kotlin_image_screen_png" => Ok(Action::KotlinImageScreen(ImageTarget::Png)),
        "kotlin_image_result" => Ok(Action::KotlinImageResult {
            target: target.as_deref().and_then(parse_image_target),
            result: if let Some(err) = error {
                ImageConversionResult {
                    path: None,
                    size: None,
                    format: None,
                    error: Some(err),
                }
            } else {
                ImageConversionResult {
                    path: result_path,
                    size: result_size,
                    format: result_format,
                    error: None,
                }
            },
        }),
        "kotlin_image_output_dir" => Ok(Action::KotlinImageOutputDir {
            target: target.as_deref().and_then(parse_image_target),
            output_dir,
        }),
        "hash_file_sha256" => Ok(Action::Hash {
            algo: HashAlgo::Sha256,
            path,
            fd,
            error,
            loading_only,
        }),
        "hash_file_sha1" => Ok(Action::Hash {
            algo: HashAlgo::Sha1,
            path,
            fd,
            error,
            loading_only,
        }),
        "hash_file_md5" => Ok(Action::Hash {
            algo: HashAlgo::Md5,
            path,
            fd,
            error,
            loading_only,
        }),
        "hash_file_md4" => Ok(Action::Hash {
            algo: HashAlgo::Md4,
            path,
            fd,
            error,
            loading_only,
        }),
        "hash_file_crc32" => Ok(Action::Hash {
            algo: HashAlgo::Crc32,
            path,
            fd,
            error,
            loading_only,
        }),
        "hash_file_blake3" => Ok(Action::Hash {
            algo: HashAlgo::Blake3,
            path,
            fd,
            error,
            loading_only,
        }),
        "progress_demo_screen" => Ok(Action::ProgressDemoScreen),
        "progress_demo_start" => Ok(Action::ProgressDemoStart { loading_only }),
        "progress_demo_finish" => Ok(Action::ProgressDemoFinish),
        "file_info_screen" => Ok(Action::FileInfoScreen),
        "file_info" => Ok(Action::FileInfo { path, fd, error }),
        "text_tools_screen" => Ok(Action::TextToolsScreen { bindings }),
        "increment" => Ok(Action::Increment),
        "snapshot" => Ok(Action::Snapshot),
        "restore_state" => snapshot
            .ok_or_else(|| "missing_snapshot".to_string())
            .map(|snap| Action::Restore { snapshot: snap }),
        other => {
            if let Some(text_action) = parse_text_action(other) {
                Ok(Action::TextTools {
                    action: text_action,
                    bindings,
                })
            } else {
                Err(error.unwrap_or_else(|| format!("unknown_action:{other}")))
            }
        }
    }
}

fn parse_text_action(name: &str) -> Option<TextAction> {
    match name {
        "text_tools_upper" => Some(TextAction::Upper),
        "text_tools_lower" => Some(TextAction::Lower),
        "text_tools_title" => Some(TextAction::Title),
        "text_tools_word_count" => Some(TextAction::WordCount),
        "text_tools_char_count" => Some(TextAction::CharCount),
        "text_tools_trim" => Some(TextAction::Trim),
        "text_tools_wrap" => Some(TextAction::Wrap),
        "text_tools_base64_encode" => Some(TextAction::Base64Encode),
        "text_tools_base64_decode" => Some(TextAction::Base64Decode),
        "text_tools_url_encode" => Some(TextAction::UrlEncode),
        "text_tools_url_decode" => Some(TextAction::UrlDecode),
        "text_tools_hex_encode" => Some(TextAction::HexEncode),
        "text_tools_hex_decode" => Some(TextAction::HexDecode),
        "text_tools_copy_to_input" => Some(TextAction::CopyToInput),
        "text_tools_share_result" => Some(TextAction::ShareResult),
        "text_tools_clear" => Some(TextAction::Clear),
        "text_tools_refresh" => Some(TextAction::Refresh),
        _ => None,
    }
}

fn hash_label(algo: HashAlgo) -> &'static str {
    match algo {
        HashAlgo::Sha256 => "SHA-256",
        HashAlgo::Sha1 => "SHA-1",
        HashAlgo::Md5 => "MD5",
        HashAlgo::Md4 => "MD4",
        HashAlgo::Crc32 => "CRC32",
        HashAlgo::Blake3 => "BLAKE3",
    }
}

fn hash_loading_message(algo: HashAlgo) -> &'static str {
    match algo {
        HashAlgo::Sha256 => "Computing SHA-256...",
        HashAlgo::Sha1 => "Computing SHA-1...",
        HashAlgo::Md5 => "Computing MD5...",
        HashAlgo::Md4 => "Computing MD4...",
        HashAlgo::Crc32 => "Computing CRC32...",
        HashAlgo::Blake3 => "Computing BLAKE3...",
    }
}

#[no_mangle]
pub extern "system" fn Java_aeska_kistaverk_MainActivity_dispatch(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let input_str: String = env
            .get_string(&input)
            .map(|s| s.into())
            .unwrap_or_else(|_| "{}".to_string());

        let command: Command = serde_json::from_str(&input_str).unwrap_or(Command {
            action: "error".into(),
            path: None,
            fd: None,
            error: Some("invalid_json".into()),
            target: None,
            result_path: None,
            result_size: None,
        result_format: None,
        output_dir: None,
        bindings: None,
        loading_only: None,
        snapshot: None,
    });

        handle_command(command)
    }));

    let json_value = match response {
        Ok(Ok(value)) => value,
        Ok(Err(err)) => error_ui(&err),
        Err(_) => error_ui("panic"),
    };

    let output_string = json_value.to_string();
    match env.new_string(output_string) {
        Ok(java_str) => java_str.into_raw(),
        Err(_) => {
            let fallback = error_ui("jni_new_string_failed").to_string();
            env.new_string(fallback)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
    }
}

fn handle_command(command: Command) -> Result<Value, String> {
    let mut lock_poisoned = false;
    let mut state = match STATE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            lock_poisoned = true;
            poisoned.into_inner()
        }
    };

    state.ensure_navigation();

    let action = match parse_action(command) {
        Ok(action) => action,
        Err(err) => {
            state.last_error = Some(err);
            return Ok(render_ui(&state));
        }
    };

    match action {
        Action::Init => {
            // Keep current state; ensure navigation is initialized.
            state.ensure_navigation();
        }
        Action::Snapshot => {
            state.ensure_navigation();
            let snap = serde_json::to_string(&*state)
                .map_err(|e| format!("snapshot_failed:{e}"))?;
            return Ok(json!({
                "type": "Snapshot",
                "snapshot": snap
            }));
        }
        Action::Restore { snapshot } => {
            match serde_json::from_str::<AppState>(&snapshot) {
                Ok(mut restored) => {
                    restored.ensure_navigation();
                    *state = restored;
                }
                Err(e) => {
                    state.last_error = Some(format!("restore_failed:{e}"));
                }
            }
        }
        Action::Reset => {
            state.reset_runtime();
            state.reset_navigation();
        }
        Action::Back => {
            state.pop_screen();
            state.loading_message = None;
        }
        Action::ShaderDemo => state.push_screen(Screen::ShaderDemo),
        Action::LoadShader { path, fd, error } => {
            let mut fd_handle = FdHandle::new(fd);
            if let Some(err) = error {
                state.last_error = Some(err);
            } else if let Some(fd) = fd_handle.take() {
                match read_text_from_fd(fd as RawFd) {
                    Ok(src) => {
                        state.last_shader = Some(src);
                        state.last_error = None;
                        state.replace_current(Screen::ShaderDemo);
                    }
                    Err(e) => state.last_error = Some(format!("shader_read_failed:{e}")),
                }
            } else if let Some(path) = path.as_deref() {
                match std::fs::read_to_string(path) {
                    Ok(src) => {
                        state.last_shader = Some(src);
                        state.last_error = None;
                        state.replace_current(Screen::ShaderDemo);
                    }
                    Err(e) => state.last_error = Some(format!("shader_read_failed:{e}")),
                }
            } else {
                state.last_error = Some("missing_shader_path".into());
            }
        }
        Action::KotlinImageScreen(target) => handle_kotlin_image_screen(&mut state, target),
        Action::KotlinImageResult { target, result } => {
            handle_kotlin_image_result(&mut state, target, result)
        }
        Action::KotlinImageOutputDir { target, output_dir } => {
            handle_kotlin_image_output_dir(&mut state, target, output_dir);
        }
        Action::Hash {
            algo,
            path,
            fd,
            error,
            loading_only,
        } => {
            let mut fd_handle = FdHandle::new(fd);
            if loading_only {
                state.loading_with_spinner = false;
                state.replace_current(Screen::Loading);
                state.loading_message = Some(hash_loading_message(algo).into());
                return Ok(render_ui(&state));
            }
            state.reset_navigation();
            state.last_hash_algo = Some(hash_label(algo).into());
            if let Some(err) = error {
                state.last_error = Some(err);
                state.last_hash = None;
            } else {
                handle_hash_action(&mut state, fd_handle.take(), path.as_deref(), algo);
            }
            state.loading_message = None;
            state.loading_with_spinner = true;
        }
        Action::ProgressDemoScreen => {
            state.push_screen(Screen::ProgressDemo);
            state.progress_status = None;
            state.loading_message = None;
        }
        Action::ProgressDemoStart { loading_only } => {
            if loading_only {
                state.replace_current(Screen::Loading);
                state.loading_message = Some("Working...".into());
                return Ok(render_ui(&state));
            } else {
                state.replace_current(Screen::ProgressDemo);
                state.progress_status = Some("Starting...".into());
            }
        }
        Action::ProgressDemoFinish => {
            state.replace_current(Screen::ProgressDemo);
            state.progress_status = Some("Done after simulated delay.".into());
            state.loading_message = None;
        }
        Action::FileInfoScreen => {
            state.push_screen(Screen::FileInfo);
            state.last_file_info = None;
            state.last_error = None;
        }
        Action::FileInfo { path, fd, error } => {
            let mut fd_handle = FdHandle::new(fd);
            state.replace_current(Screen::FileInfo);
            let info = if let Some(err) = error {
                features::file_info::FileInfoResult {
                    path: path.map(|p| p.to_string()),
                    size_bytes: None,
                    mime: None,
                    error: Some(err),
                }
            } else if let Some(fd) = fd_handle.take() {
                file_info_from_fd(fd as RawFd)
            } else if let Some(path) = path.as_deref() {
                file_info_from_path(path)
            } else {
                features::file_info::FileInfoResult {
                    path: None,
                    size_bytes: None,
                    mime: None,
                    error: Some("missing_path".into()),
                }
            };
            state.last_file_info = Some(serde_json::to_string(&info).unwrap_or_default());
        }
        Action::TextToolsScreen { bindings } => {
            state.push_screen(Screen::TextTools);
            state.text_output = None;
            state.text_operation = None;
            if let Some(input) = bindings.get("text_input") {
                state.text_input = Some(input.clone());
            }
        }
        Action::TextTools { action, bindings } => {
            handle_text_action(&mut state, action, &bindings);
        }
        Action::Increment => state.counter += 1,
    }

    if lock_poisoned && state.last_error.is_none() {
        state.last_error = Some("state_poisoned".into());
    }

    Ok(render_ui(&state))
}

fn read_text_from_fd(fd: RawFd) -> Result<String, String> {
    if fd < 0 {
        return Err("invalid_fd".into());
    }

    let mut file = unsafe { File::from_raw_fd(fd) };
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("read_failed:{e}"))?;
    Ok(contents)
}

fn error_ui(message: &str) -> Value {
    json!({
        "type": "Column",
        "padding": 24,
        "children": [
            { "type": "Text", "text": "Error", "size": 18.0 },
            { "type": "Text", "text": message }
        ]
    })
}

fn render_ui(state: &AppState) -> Value {
    match state.current_screen() {
        Screen::Home => render_menu(state, &feature_catalog()),
        Screen::ShaderDemo => render_shader_screen(state),
        Screen::KotlinImage => render_kotlin_image_screen(state),
        Screen::FileInfo => render_file_info_screen(state),
        Screen::TextTools => render_text_tools_screen(state),
        Screen::Loading => render_loading_screen(state),
        Screen::ProgressDemo => render_progress_demo_screen(state),
    }
}

fn maybe_push_back(children: &mut Vec<Value>, state: &AppState) {
    if state.nav_depth() > 1 {
        children.push(json!({
            "type": "Button",
            "text": "Back",
            "action": "back"
        }));
    }
}

fn render_file_info_screen(state: &AppState) -> Value {
        let mut children = vec![
        serde_json::to_value(UiText::new("File info").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Select a file to see its size and MIME type").size(14.0),
        )
        .unwrap(),
        json!({
            "type": "Button",
            "text": "Pick file",
            "action": "file_info",
            "requires_file_picker": true
        }),
    ];

    if let Some(info_json) = &state.last_file_info {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(info_json) {
            if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                children.push(json!({
                    "type": "Text",
                    "text": format!("Error: {err}"),
                    "size": 14.0
                }));
            } else {
                if let Some(path) = parsed.get("path").and_then(|p| p.as_str()) {
                    children.push(json!({
                        "type": "Text",
                        "text": format!("Path: {path}"),
                    }));
                }
                if let Some(size) = parsed.get("size_bytes").and_then(|s| s.as_u64()) {
                    children.push(json!({
                        "type": "Text",
                        "text": format!("Size: {} bytes", size),
                    }));
                }
                if let Some(mime) = parsed.get("mime").and_then(|m| m.as_str()) {
                    children.push(json!({
                        "type": "Text",
                        "text": format!("MIME: {mime}"),
                    }));
                }
            }
        }
    }

    maybe_push_back(&mut children, state);

    json!({
        "type": "Column",
        "padding": 24,
        "children": children
    })
}

fn render_loading_screen(state: &AppState) -> Value {
    let message = state.loading_message.as_deref().unwrap_or("Working...");
    let mut children = vec![serde_json::to_value(UiText::new(message).size(16.0)).unwrap()];
    if state.loading_with_spinner {
        children.push(serde_json::to_value(
            UiProgress::new().content_description("In progress"),
        )
        .unwrap());
    }
    serde_json::to_value(UiColumn::new(children).padding(24)).unwrap()
}

fn render_shader_screen(state: &AppState) -> Value {
    let fragment = state
        .last_shader
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(SAMPLE_SHADER);

    let mut children = vec![
        serde_json::to_value(UiText::new("Shader toy demo").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Simple fragment shader with time and resolution uniforms."),
        )
        .unwrap(),
        json!({
            "type": "ShaderToy",
            "fragment": fragment
        }),
        serde_json::to_value(
            UiButton::new("Load shader from file", "load_shader_file").requires_file_picker(true),
        )
        .unwrap(),
        serde_json::to_value(
            UiText::new("Sample syntax:\nprecision mediump float;\nuniform float u_time;\nuniform vec2 u_resolution;\nvoid main(){ vec2 uv=gl_FragCoord.xy/u_resolution.xy; vec3 col=0.5+0.5*cos(u_time*0.2+uv.xyx+vec3(0.,2.,4.)); gl_FragColor=vec4(col,1.0); }").size(12.0),
        )
        .unwrap(),
    ];
    maybe_push_back(&mut children, state);

    serde_json::to_value(UiColumn::new(children).padding(16)).unwrap()
}

fn render_progress_demo_screen(state: &AppState) -> Value {
    let mut children = vec![
        json!({
            "type": "Text",
            "text": "Progress demo",
            "size": 20.0
        }),
        json!({
            "type": "Text",
            "text": "Tap start to show a 10 second simulated progress and return here when done.",
            "size": 14.0
        }),
        json!({
            "type": "Button",
            "text": "Start 10s work",
            "action": "progress_demo_start"
        }),
    ];

    if let Some(status) = &state.progress_status {
        children.push(json!({
            "type": "Text",
            "text": format!("Status: {}", status),
            "size": 14.0
        }));
    }

    maybe_push_back(&mut children, state);

    json!({
        "type": "Column",
        "padding": 24,
        "children": children
    })
}

const SAMPLE_SHADER: &str = r#"
precision mediump float;
uniform float u_time;
uniform vec2 u_resolution;
void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution.xy;
    float t = u_time * 0.2;
    vec3 col = 0.5 + 0.5*cos(t + uv.xyx + vec3(0.0,2.0,4.0));
    gl_FragColor = vec4(col, 1.0);
}
"#;

fn feature_catalog() -> Vec<Feature> {
    vec![
        Feature {
            id: "hash_sha256",
            name: "🔒 SHA-256",
            category: "🔐 Hashes",
            action: "hash_file_sha256",
            requires_file_picker: true,
            description: "secure hash",
        },
        Feature {
            id: "hash_sha1",
            name: "🛡️ SHA-1",
            category: "🔐 Hashes",
            action: "hash_file_sha1",
            requires_file_picker: true,
            description: "legacy hash",
        },
        Feature {
            id: "hash_md5",
            name: "📦 MD5",
            category: "🔐 Hashes",
            action: "hash_file_md5",
            requires_file_picker: true,
            description: "legacy hash",
        },
        Feature {
            id: "hash_md4",
            name: "📜 MD4",
            category: "🔐 Hashes",
            action: "hash_file_md4",
            requires_file_picker: true,
            description: "legacy hash",
        },
        Feature {
            id: "file_info",
            name: "📂 File info",
            category: "📁 Files",
            action: "file_info_screen",
            requires_file_picker: false,
            description: "size & MIME",
        },
        Feature {
            id: "image_to_webp_kotlin",
            name: "🖼️ Image → WebP (Kotlin)",
            category: "📸 Media",
            action: "kotlin_image_screen_webp",
            requires_file_picker: false,
            description: "Kotlin conversion with Rust UI",
        },
        Feature {
            id: "image_to_png_kotlin",
            name: "🖼️ Image → PNG (Kotlin)",
            category: "📸 Media",
            action: "kotlin_image_screen_png",
            requires_file_picker: false,
            description: "Kotlin conversion with Rust UI",
        },
        Feature {
            id: "shader_demo",
            name: "Shader demo",
            category: "Graphics",
            action: "shader_demo",
            requires_file_picker: false,
            description: "GLSL sample",
        },
        Feature {
            id: "hash_crc32",
            name: "📏 CRC32",
            category: "🔐 Hashes",
            action: "hash_file_crc32",
            requires_file_picker: true,
            description: "checksum",
        },
        Feature {
            id: "hash_blake3",
            name: "⚡ BLAKE3",
            category: "🔐 Hashes",
            action: "hash_file_blake3",
            requires_file_picker: true,
            description: "fast hash",
        },
        Feature {
            id: "progress_demo",
            name: "⏳ Progress demo",
            category: "🧪 Experiments",
            action: "progress_demo_screen",
            requires_file_picker: false,
            description: "10s simulated work",
        },
        Feature {
            id: "text_tools",
            name: "✍️ Text tools",
            category: "📝 Text",
            action: "text_tools_screen",
            requires_file_picker: false,
            description: "case & counts",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::os::unix::io::IntoRawFd;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    const SAMPLE_CONTENT: &str = "abc";
    const SHA256_ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    const SHA1_ABC: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";
    const MD5_ABC: &str = "900150983cd24fb0d6963f7d28e17f72";
    const SAMPLE_WRAP: &str = "rust keeps your memory safe by design and gives you fearless concurrency without data races";

    fn make_command(action: &str) -> Command {
        Command {
            action: action.into(),
            path: None,
            fd: None,
            error: None,
            target: None,
            result_path: None,
            result_size: None,
            result_format: None,
            output_dir: None,
            bindings: None,
            loading_only: None,
            snapshot: None,
        }
    }

    fn reset_state() {
        handle_command(make_command("reset")).expect("reset command should succeed");
    }

    fn extract_texts(ui: &Value) -> Vec<String> {
        fn walk(node: &Value, acc: &mut Vec<String>) {
            if let Some(text) = node.get("text").and_then(|t| t.as_str()) {
                acc.push(text.to_string());
            }
            if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    walk(child, acc);
                }
            }
        }

        let mut out = Vec::new();
        walk(ui, &mut out);
        out
    }

    fn assert_contains_text(ui: &Value, needle: &str) {
        let texts = extract_texts(ui);
        assert!(
            texts.iter().any(|t| t.contains(needle)),
            "expected UI to contain text with `{needle}`, found: {texts:?}"
        );
    }

    #[test]
    fn hash_file_sha256_via_path_updates_ui_and_state() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(SAMPLE_CONTENT.as_bytes()).unwrap();
        file.flush().unwrap();

        let mut command = make_command("hash_file_sha256");
        command.path = Some(file.path().to_string_lossy().into_owned());

        let ui = handle_command(command).expect("hash command should succeed");

        assert_contains_text(&ui, &format!("SHA-256: {SHA256_ABC}"));

        let state = STATE.lock().unwrap();
        assert_eq!(state.last_hash.as_deref(), Some(SHA256_ABC));
        assert_eq!(state.last_hash_algo.as_deref(), Some("SHA-256"));
        assert!(state.last_error.is_none());
    }

    #[test]
    fn hash_file_loading_then_result() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(SAMPLE_CONTENT.as_bytes()).unwrap();
        file.flush().unwrap();

        let mut loading_cmd = make_command("hash_file_sha256");
        loading_cmd.loading_only = Some(true);
        let loading_ui = handle_command(loading_cmd).expect("loading command should succeed");
        assert_contains_text(&loading_ui, "Computing SHA-256");

        let mut command = make_command("hash_file_sha256");
        command.path = Some(file.path().to_string_lossy().into_owned());

        let ui = handle_command(command).expect("hash command should succeed");

        assert_contains_text(&ui, &format!("SHA-256: {SHA256_ABC}"));

        let state = STATE.lock().unwrap();
        assert_eq!(state.last_hash.as_deref(), Some(SHA256_ABC));
        assert_eq!(state.last_hash_algo.as_deref(), Some("SHA-256"));
        assert!(state.last_error.is_none());
        assert!(matches!(state.current_screen(), Screen::Home));
    }

    #[test]
    fn hash_file_sha1_via_fd_updates_ui_and_state() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(SAMPLE_CONTENT.as_bytes()).unwrap();
        file.flush().unwrap();

        let fd = File::open(file.path()).unwrap().into_raw_fd();

        let mut command = make_command("hash_file_sha1");
        command.fd = Some(fd);

        let ui = handle_command(command).expect("hash command should succeed");

        assert_contains_text(&ui, &format!("SHA-1: {SHA1_ABC}"));

        let state = STATE.lock().unwrap();
        assert_eq!(state.last_hash.as_deref(), Some(SHA1_ABC));
        assert_eq!(state.last_hash_algo.as_deref(), Some("SHA-1"));
        assert!(state.last_error.is_none());
    }

    #[test]
    fn text_tools_uppercase_consumes_binding_and_updates_state() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut command = make_command("text_tools_upper");
        command.bindings = Some(HashMap::from([("text_input".into(), "Hello rust".into())]));

        let ui = handle_command(command).expect("text command should succeed");

        assert_contains_text(&ui, "HELLO RUST");

        let state = STATE.lock().unwrap();
        assert_eq!(state.text_input.as_deref(), Some("Hello rust"));
        assert_eq!(state.text_output.as_deref(), Some("HELLO RUST"));
        assert!(matches!(state.current_screen(), Screen::TextTools));
    }

    #[test]
    fn text_tools_word_count_reports_count() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut command = make_command("text_tools_word_count");
        command.bindings = Some(HashMap::from([(
            "text_input".into(),
            "one two  three".into(),
        )]));

        let ui = handle_command(command).expect("text command should succeed");

        assert_contains_text(&ui, "Word count: 3");

        let state = STATE.lock().unwrap();
        assert_eq!(state.text_output.as_deref(), Some("Word count: 3"));
    }

    #[test]
    fn text_tools_wrap_splits_lines() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut command = make_command("text_tools_wrap");
        command.bindings = Some(HashMap::from([("text_input".into(), SAMPLE_WRAP.into())]));

        let ui = handle_command(command).expect("text command should succeed");
        let texts = extract_texts(&ui);
        // Wrapped text should introduce a line break (result block shows raw text)
        let result = texts
            .into_iter()
            .find(|t| t == "Result")
            .and_then(|_| {
                // result text is next entry after label in traversal
                STATE.lock().ok().and_then(|s| s.text_output.clone())
            })
            .unwrap_or_default();

        assert!(
            result.contains('\n'),
            "expected wrapped text to contain newline, got {result:?}"
        );
        let state = STATE.lock().unwrap();
        assert!(state
            .text_output
            .as_deref()
            .unwrap_or_default()
            .contains('\n'));
    }

    #[test]
    fn text_tools_trim_respects_aggressive_flag() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut command = make_command("text_tools_trim");
        command.bindings = Some(HashMap::from([
            ("text_input".into(), "  a   b  ".into()),
            ("aggressive_trim".into(), "true".into()),
        ]));

        let ui = handle_command(command).expect("trim command should succeed");
        assert_contains_text(&ui, "Trim spacing (collapse)");

        let state = STATE.lock().unwrap();
        assert_eq!(state.text_output.as_deref(), Some("a b"));

        drop(state);
        reset_state();

        let mut command2 = make_command("text_tools_trim");
        command2.bindings = Some(HashMap::from([
            ("text_input".into(), "  a   b  ".into()),
            ("aggressive_trim".into(), "false".into()),
        ]));

        let ui2 = handle_command(command2).expect("trim command should succeed");
        assert_contains_text(&ui2, "Trim edges");
        let state2 = STATE.lock().unwrap();
        assert_eq!(state2.text_output.as_deref(), Some("a   b"));
    }

    #[test]
    fn back_from_home_does_not_underflow_stack() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let ui = handle_command(make_command("back")).expect("back should succeed");
        assert_contains_text(&ui, "Tool menu");

        let state = STATE.lock().unwrap();
        assert_eq!(state.nav_depth(), 1);
        assert!(matches!(state.current_screen(), Screen::Home));
    }

    #[test]
    fn back_pops_to_previous_screen() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        handle_command(make_command("text_tools_screen")).expect("screen switch should work");
        {
            let state = STATE.lock().unwrap();
            assert_eq!(state.nav_depth(), 2);
            assert!(matches!(state.current_screen(), Screen::TextTools));
        }

        handle_command(make_command("back")).expect("back should succeed");
        let state = STATE.lock().unwrap();
        assert_eq!(state.nav_depth(), 1);
        assert!(matches!(state.current_screen(), Screen::Home));
    }

    #[test]
    fn snapshot_and_restore_round_trip() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        // Move into text tools and set some state
        let mut cmd = make_command("text_tools_upper");
        cmd.bindings = Some(HashMap::from([("text_input".into(), "hi".into())]));
        handle_command(cmd).expect("text action should succeed");

        let snap_value = handle_command(make_command("snapshot")).expect("snapshot should succeed");
        let snap_str = snap_value
            .get("snapshot")
            .and_then(|v| v.as_str())
            .expect("snapshot missing");

        // Reset state and ensure we go back to home
        reset_state();
        {
            let state = STATE.lock().unwrap();
            assert!(matches!(state.current_screen(), Screen::Home));
            assert!(state.text_output.is_none());
        }

        let mut restore_cmd = make_command("restore_state");
        restore_cmd.snapshot = Some(snap_str.to_string());
        let ui_after_restore =
            handle_command(restore_cmd).expect("restore should succeed and return UI");
        assert_contains_text(&ui_after_restore, "Result");

        let state = STATE.lock().unwrap();
        assert!(matches!(state.current_screen(), Screen::TextTools));
        assert_eq!(state.text_output.as_deref(), Some("HI"));
        assert_eq!(state.text_input.as_deref(), Some("hi"));
    }

    #[test]
    fn text_tools_base64_roundtrip() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut enc = make_command("text_tools_base64_encode");
        enc.bindings = Some(HashMap::from([("text_input".into(), "hi".into())]));
        handle_command(enc).expect("encode should work");
        {
            let state = STATE.lock().unwrap();
            assert_eq!(state.text_output.as_deref(), Some("aGk="));
        }

        let mut dec = make_command("text_tools_base64_decode");
        dec.bindings = Some(HashMap::from([("text_input".into(), "aGk=".into())]));
        let ui = handle_command(dec).expect("decode should work");
        assert_contains_text(&ui, "hi");
        let state = STATE.lock().unwrap();
        assert_eq!(state.text_output.as_deref(), Some("hi"));
    }

    #[test]
    fn text_tools_hex_roundtrip() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut enc = make_command("text_tools_hex_encode");
        enc.bindings = Some(HashMap::from([("text_input".into(), "hi".into())]));
        handle_command(enc).expect("encode should work");
        {
            let state = STATE.lock().unwrap();
            assert_eq!(state.text_output.as_deref(), Some("6869"));
        }

        let mut dec = make_command("text_tools_hex_decode");
        dec.bindings = Some(HashMap::from([("text_input".into(), "6869".into())]));
        let ui = handle_command(dec).expect("decode should work");
        assert_contains_text(&ui, "hi");
        let state = STATE.lock().unwrap();
        assert_eq!(state.text_output.as_deref(), Some("hi"));
    }

    #[test]
    fn missing_source_sets_error_and_clears_previous_hash() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(SAMPLE_CONTENT.as_bytes()).unwrap();
        file.flush().unwrap();

        let mut initial = make_command("hash_file_md5");
        initial.path = Some(file.path().to_string_lossy().into_owned());

        handle_command(initial).expect("initial hash should succeed");

        {
            let state = STATE.lock().unwrap();
            assert_eq!(state.last_hash.as_deref(), Some(MD5_ABC));
            assert_eq!(state.last_hash_algo.as_deref(), Some("MD5"));
        }

        let ui = handle_command(make_command("hash_file_md4"))
            .expect("hash command should still return UI even when failing");

        assert_contains_text(&ui, "missing_path");

        let state = STATE.lock().unwrap();
        assert_eq!(state.last_hash, None);
        assert_eq!(state.last_error.as_deref(), Some("missing_path"));
        assert_eq!(state.last_hash_algo.as_deref(), Some("MD4"));
    }
}
