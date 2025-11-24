mod features;
mod state;
use features::hashes::{handle_hash_action, HashAlgo};
use features::kotlin_image::{
    handle_result as handle_kotlin_image_result, handle_screen_entry as handle_kotlin_image_screen,
    handle_output_dir as handle_kotlin_image_output_dir, parse_image_target,
    render_kotlin_image_screen, ImageConversionResult, ImageTarget,
};
use features::{render_menu, Feature};

use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use serde::Deserialize;
use serde_json::{json, Value};
use state::{AppState, Screen};
use std::{
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
    } = command;

    let mut fd_handle = FdHandle::new(fd);

    let mut state = STATE.lock().map_err(|_| "state_lock_failed".to_string())?;

    let command_error = error;

    match action.as_str() {
        "reset" => {
            state.last_hash = None;
            state.last_error = None;
            state.current_screen = Screen::Home;
            state.last_shader = None;
            state.last_hash_algo = None;
            state.image.reset();
        }
        "shader_demo" => state.current_screen = Screen::ShaderDemo,
        "load_shader_file" => {
            if let Some(err) = command_error.as_ref() {
                state.last_error = Some(err.clone());
            } else if let Some(fd) = fd_handle.take() {
                match read_text_from_fd(fd as RawFd) {
                    Ok(src) => {
                        state.last_shader = Some(src);
                        state.last_error = None;
                        state.current_screen = Screen::ShaderDemo;
                    }
                    Err(e) => state.last_error = Some(format!("shader_read_failed:{e}")),
                }
            } else if let Some(path) = path.as_deref() {
                match std::fs::read_to_string(path) {
                    Ok(src) => {
                        state.last_shader = Some(src);
                        state.last_error = None;
                        state.current_screen = Screen::ShaderDemo;
                    }
                    Err(e) => state.last_error = Some(format!("shader_read_failed:{e}")),
                }
            } else {
                state.last_error = Some("missing_shader_path".into());
            }
        }
        "kotlin_image_screen_webp" => handle_kotlin_image_screen(&mut state, ImageTarget::Webp),
        "kotlin_image_screen_png" => handle_kotlin_image_screen(&mut state, ImageTarget::Png),
        "kotlin_image_result" => handle_kotlin_image_result(
            &mut state,
            target.as_deref().and_then(parse_image_target),
            if let Some(err) = command_error {
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
        ),
        "kotlin_image_output_dir" => {
            handle_kotlin_image_output_dir(
                &mut state,
                target.as_deref().and_then(parse_image_target),
                output_dir,
            );
        }
        "hash_file_sha256" => {
            state.current_screen = Screen::Home;
            state.last_hash_algo = Some("SHA-256".into());
            if let Some(err) = command_error.as_ref() {
                state.last_error = Some(err.clone());
                state.last_hash = None;
            } else {
                handle_hash_action(
                    &mut state,
                    fd_handle.take(),
                    path.as_deref(),
                    HashAlgo::Sha256,
                );
            }
        }
        "hash_file_sha1" => {
            state.current_screen = Screen::Home;
            state.last_hash_algo = Some("SHA-1".into());
            if let Some(err) = command_error.as_ref() {
                state.last_error = Some(err.clone());
                state.last_hash = None;
            } else {
                handle_hash_action(
                    &mut state,
                    fd_handle.take(),
                    path.as_deref(),
                    HashAlgo::Sha1,
                );
            }
        }
        "hash_file_md5" => {
            state.current_screen = Screen::Home;
            state.last_hash_algo = Some("MD5".into());
            if let Some(err) = command_error.as_ref() {
                state.last_error = Some(err.clone());
                state.last_hash = None;
            } else {
                handle_hash_action(&mut state, fd_handle.take(), path.as_deref(), HashAlgo::Md5);
            }
        }
        "hash_file_md4" => {
            state.current_screen = Screen::Home;
            state.last_hash_algo = Some("MD4".into());
            if let Some(err) = command_error.as_ref() {
                state.last_error = Some(err.clone());
                state.last_hash = None;
            } else {
                handle_hash_action(&mut state, fd_handle.take(), path.as_deref(), HashAlgo::Md4);
            }
        }
        "increment" => state.counter += 1,
        _ => {
            if let Some(err) = command_error {
                state.last_error = Some(err);
                state.last_hash = None;
            }
        }
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
    match state.current_screen {
        Screen::Home => render_menu(state, &feature_catalog()),
        Screen::ShaderDemo => {
            let fragment = state
                .last_shader
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(SAMPLE_SHADER);

            json!({
                "type": "Column",
                "padding": 16,
                "children": [
                    { "type": "Text", "text": "Shader toy demo", "size": 20.0 },
                    { "type": "Text", "text": "Simple fragment shader with time and resolution uniforms." },
                    {
                        "type": "ShaderToy",
                        "fragment": fragment
                    },
                    {
                        "type": "Button",
                        "text": "Load shader from file",
                        "action": "load_shader_file",
                        "requires_file_picker": true
                    },
                    {
                        "type": "Text",
                        "text": "Sample syntax:\nprecision mediump float;\nuniform float u_time;\nuniform vec2 u_resolution;\nvoid main(){ vec2 uv=gl_FragCoord.xy/u_resolution.xy; vec3 col=0.5+0.5*cos(u_time*0.2+uv.xyx+vec3(0.,2.,4.)); gl_FragColor=vec4(col,1.0); }",
                        "size": 12.0
                    },
                    { "type": "Button", "text": "Back", "action": "reset" }
                ]
            })
        }
        Screen::KotlinImage => render_kotlin_image_screen(state),
    }
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
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
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
        }
    }

    fn reset_state() {
        handle_command(make_command("reset")).expect("reset command should succeed");
    }

    fn extract_texts(ui: &Value) -> Vec<String> {
        ui.get("children")
            .and_then(|c| c.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|child| child.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()))
            .collect()
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
