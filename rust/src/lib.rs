mod features;
mod state;
mod ui;
use features::archive::{handle_archive_open, render_archive_screen};
use features::color_tools::{handle_color_action, render_color_screen};
use features::file_info::{file_info_from_fd, file_info_from_path};
use features::hashes::{handle_hash_action, handle_hash_verify, HashAlgo};
use features::kotlin_image::{
    handle_output_dir as handle_kotlin_image_output_dir,
    handle_resize_screen as handle_kotlin_image_resize_screen,
    handle_resize_sync as handle_kotlin_image_resize_sync,
    handle_result as handle_kotlin_image_result, handle_screen_entry as handle_kotlin_image_screen,
    parse_image_target, render_kotlin_image_screen, ImageConversionResult, ImageTarget,
};
use features::pdf::{
    handle_pdf_operation, handle_pdf_select, handle_pdf_sign, handle_pdf_title, render_pdf_screen,
    PdfOperation,
};
use features::qr::{handle_qr_action, render_qr_screen};
use features::sensor_logger::{
    apply_status_from_bindings, parse_bindings as parse_sensor_bindings,
};
use features::text_tools::{handle_text_action, render_text_tools_screen, TextAction};
use features::text_viewer::{
    guess_language_from_path, load_more_text, load_prev_text, load_text_from_fd,
    load_text_from_path, load_text_from_path_at_offset,
};
use features::{render_menu, Feature};
use ui::{
    Barometer as UiBarometer, Button as UiButton, CodeView as UiCodeView, Column as UiColumn,
    Compass as UiCompass, DepsList as UiDepsList, Magnetometer as UiMagnetometer,
    Progress as UiProgress, Text as UiText, Warning as UiWarning,
};

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
// TODO: reduce lock hold time or move to a channel/queue; consider parking_lot with timeouts to avoid long UI pauses.

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
    primary_fd: Option<i32>,
    primary_path: Option<String>,
    angle_radians: Option<f64>,
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
        bindings: HashMap<String, String>,
    },
    KotlinImageResizeScreen,
    KotlinImageResizeSync {
        bindings: HashMap<String, String>,
    },
    KotlinImageOutputDir {
        target: Option<ImageTarget>,
        output_dir: Option<String>,
    },
    HashVerifyScreen,
    HashVerify {
        path: Option<String>,
        fd: Option<i32>,
        reference: Option<String>,
    },
    HashVerifyPaste {
        reference: Option<String>,
    },
    QrGenerate {
        input: Option<String>,
    },
    ColorFromHex {
        input: Option<String>,
    },
    ColorFromRgb {
        input: Option<String>,
    },
    ColorCopyHexInput {
        input: Option<String>,
    },
    ColorCopyClipboard,
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
    PdfToolsScreen,
    PdfSelect {
        fd: Option<i32>,
        uri: Option<String>,
        error: Option<String>,
    },
    PdfExtract {
        fd: Option<i32>,
        uri: Option<String>,
        selection: Vec<u32>,
    },
    PdfDelete {
        fd: Option<i32>,
        uri: Option<String>,
        selection: Vec<u32>,
    },
    PdfMerge {
        primary_fd: Option<i32>,
        primary_uri: Option<String>,
        secondary_fd: Option<i32>,
        secondary_uri: Option<String>,
    },
    PdfSign {
        fd: Option<i32>,
        uri: Option<String>,
        signature: Option<String>,
        page: Option<u32>,
        page_x_pct: Option<f64>,
        page_y_pct: Option<f64>,
        pos_x: f64,
        pos_y: f64,
        width: f64,
        height: f64,
        img_width_px: Option<f64>,
        img_height_px: Option<f64>,
        img_dpi: Option<f64>,
    },
    PdfSignGrid {
        page: u32,
        x_pct: f64,
        y_pct: f64,
    },
    PdfSetTitle {
        fd: Option<i32>,
        uri: Option<String>,
        title: Option<String>,
    },
    PdfSignatureStore {
        data: Option<String>,
    },
    PdfSignatureClear,
    About,
    TextViewerScreen,
    TextViewerOpen {
        fd: Option<i32>,
        path: Option<String>,
        error: Option<String>,
    },
    TextViewerToggleTheme,
    TextViewerToggleLineNumbers,
    TextViewerLoadAnyway,
    TextViewerLoadMore,
    TextViewerLoadPrev,
    TextViewerJump {
        offset: Option<u64>,
    },
    TextViewerFind {
        query: Option<String>,
        direction: Option<String>,
    },
    SensorLoggerScreen,
    SensorLoggerStart {
        bindings: HashMap<String, String>,
    },
    SensorLoggerStop,
    SensorLoggerShare,
    SensorLoggerStatus {
        bindings: HashMap<String, String>,
    },
    Increment,
    Snapshot,
    Restore {
        snapshot: String,
    },
    ArchiveToolsScreen,
    ArchiveOpen {
        fd: Option<i32>,
        path: Option<String>,
        error: Option<String>,
    },
    ArchiveOpenText {
        index: usize,
    },
    CompassDemo,
    CompassSet {
        angle_radians: f64,
        error: Option<String>,
    },
    BarometerScreen,
    BarometerSet {
        hpa: f64,
        error: Option<String>,
    },
    MagnetometerScreen,
    MagnetometerSet {
        magnitude_ut: f64,
        error: Option<String>,
    },
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
        primary_fd,
        primary_path,
        angle_radians,
    } = command;

    let bindings = bindings.unwrap_or_default();
    let loading_only = loading_only.unwrap_or(false);

    match action.as_str() {
        "init" => Ok(Action::Init),
        "reset" => Ok(Action::Reset),
        "back" => Ok(Action::Back),
        "pdf_tools_screen" => Ok(Action::PdfToolsScreen),
        "pdf_select" => Ok(Action::PdfSelect {
            fd,
            uri: path,
            error,
        }),
        "pdf_extract" => Ok(Action::PdfExtract {
            fd,
            uri: path,
            selection: parse_pdf_selection(&bindings),
        }),
        "pdf_delete" => Ok(Action::PdfDelete {
            fd,
            uri: path,
            selection: parse_pdf_selection(&bindings),
        }),
        "pdf_set_title" => Ok(Action::PdfSetTitle {
            fd,
            uri: path,
            title: bindings.get("pdf_title").cloned(),
        }),
        "pdf_merge" => Ok(Action::PdfMerge {
            primary_fd,
            primary_uri: primary_path,
            secondary_fd: fd,
            secondary_uri: path,
        }),
        "pdf_sign" => Ok(Action::PdfSign {
            fd,
            uri: path,
            signature: bindings.get("signature_base64").cloned(),
            page: parse_u32_binding(&bindings, "pdf_signature_page"),
            page_x_pct: parse_f64_binding(&bindings, "pdf_signature_x_pct"),
            page_y_pct: parse_f64_binding(&bindings, "pdf_signature_y_pct"),
            pos_x: parse_f64_binding(&bindings, "pdf_signature_x").unwrap_or(32.0),
            pos_y: parse_f64_binding(&bindings, "pdf_signature_y").unwrap_or(32.0),
            width: parse_f64_binding(&bindings, "pdf_signature_width").unwrap_or(180.0),
            height: parse_f64_binding(&bindings, "pdf_signature_height").unwrap_or(60.0),
            img_width_px: parse_f64_binding(&bindings, "signature_width_px"),
            img_height_px: parse_f64_binding(&bindings, "signature_height_px"),
            img_dpi: parse_f64_binding(&bindings, "signature_dpi"),
        }),
        "pdf_sign_grid" => Ok(Action::PdfSignGrid {
            page: parse_u32_binding(&bindings, "pdf_signature_page").unwrap_or(1),
            x_pct: parse_f64_binding(&bindings, "pdf_signature_x_pct").unwrap_or(0.5),
            y_pct: parse_f64_binding(&bindings, "pdf_signature_y_pct").unwrap_or(0.5),
        }),
        "pdf_signature_store" => Ok(Action::PdfSignatureStore {
            data: bindings.get("signature_base64").cloned(),
        }),
        "pdf_signature_clear" => Ok(Action::PdfSignatureClear),
        "about" => Ok(Action::About),
        "text_viewer_screen" => Ok(Action::TextViewerScreen),
        "text_viewer_open" => Ok(Action::TextViewerOpen { fd, path, error }),
        "text_viewer_toggle_theme" => Ok(Action::TextViewerToggleTheme),
        "text_viewer_toggle_line_numbers" => Ok(Action::TextViewerToggleLineNumbers),
        "text_viewer_load_anyway" => Ok(Action::TextViewerLoadAnyway),
        "text_viewer_load_more" => Ok(Action::TextViewerLoadMore),
        "text_viewer_load_prev" => Ok(Action::TextViewerLoadPrev),
        "text_viewer_jump" => Ok(Action::TextViewerJump {
            offset: parse_u64_binding(&bindings, "offset_bytes"),
        }),
        "text_viewer_find" => Ok(Action::TextViewerFind {
            query: bindings.get("find_query").cloned(),
            direction: bindings.get("find_direction").cloned(),
        }),
        "text_viewer_find_submit" => Ok(Action::TextViewerFind {
            query: bindings.get("find_query").cloned(),
            direction: None,
        }),
        "text_viewer_find_next" => Ok(Action::TextViewerFind {
            query: bindings.get("find_query").cloned(),
            direction: Some("next".into()),
        }),
        "text_viewer_find_prev" => Ok(Action::TextViewerFind {
            query: bindings.get("find_query").cloned(),
            direction: Some("prev".into()),
        }),
        "text_viewer_find_clear" => Ok(Action::TextViewerFind {
            query: Some(String::new()),
            direction: None,
        }),
        "sensor_logger_screen" => Ok(Action::SensorLoggerScreen),
        "sensor_logger_start" => Ok(Action::SensorLoggerStart { bindings }),
        "sensor_logger_stop" => Ok(Action::SensorLoggerStop),
        "sensor_logger_share" => Ok(Action::SensorLoggerShare),
        "sensor_logger_status" => Ok(Action::SensorLoggerStatus { bindings }),
        "shader_demo" => Ok(Action::ShaderDemo),
        "load_shader_file" => Ok(Action::LoadShader { path, fd, error }),
        "kotlin_image_screen_webp" => Ok(Action::KotlinImageScreen(ImageTarget::Webp)),
        "kotlin_image_screen_png" => Ok(Action::KotlinImageScreen(ImageTarget::Png)),
        "kotlin_image_resize_screen" => Ok(Action::KotlinImageResizeScreen),
        "kotlin_image_resize_sync" => Ok(Action::KotlinImageResizeSync { bindings }),
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
            bindings,
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
        "hash_verify_screen" => Ok(Action::HashVerifyScreen),
        "hash_verify" => Ok(Action::HashVerify {
            path,
            fd,
            reference: bindings.get("hash_reference").cloned(),
        }),
        "hash_verify_paste" => Ok(Action::HashVerifyPaste {
            reference: bindings
                .get("clipboard")
                .cloned()
                .or_else(|| bindings.get("hash_reference").cloned()),
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
        "qr_generate" => {
            let input = bindings.get("qr_input").cloned().or(path);
            Ok(Action::QrGenerate { input })
        }
        "color_from_hex" => Ok(Action::ColorFromHex {
            input: bindings
                .get("color_input")
                .cloned()
                .or_else(|| path.clone()),
        }),
        "color_from_rgb" => Ok(Action::ColorFromRgb {
            input: bindings
                .get("color_input")
                .cloned()
                .or_else(|| path.clone()),
        }),
        "color_copy_hex_input" => Ok(Action::ColorCopyHexInput {
            input: bindings
                .get("color_input")
                .or_else(|| bindings.get("clipboard"))
                .cloned()
                .or_else(|| path.clone()),
        }),
        "color_copy_clipboard" => Ok(Action::ColorCopyClipboard),
        "archive_tools_screen" => Ok(Action::ArchiveToolsScreen),
        "archive_open" => Ok(Action::ArchiveOpen { fd, path, error }),
        "compass_demo" => Ok(Action::CompassDemo),
        "compass_set" => Ok(Action::CompassSet {
            angle_radians: angle_radians.unwrap_or(0.0),
            error,
        }),
        "barometer_screen" => Ok(Action::BarometerScreen),
        "barometer_set" => Ok(Action::BarometerSet {
            hpa: angle_radians.unwrap_or(0.0),
            error,
        }),
        "magnetometer_screen" => Ok(Action::MagnetometerScreen),
        "magnetometer_set" => Ok(Action::MagnetometerSet {
            magnitude_ut: angle_radians.unwrap_or(0.0),
            error,
        }),
        other => {
            if let Some(idx) = other.strip_prefix("archive_open_text:") {
                let index = idx
                    .parse::<usize>()
                    .map_err(|_| format!("invalid_archive_index:{idx}"))?;
                Ok(Action::ArchiveOpenText { index })
            } else if let Some(text_action) = parse_text_action(other) {
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

fn parse_pdf_selection(bindings: &HashMap<String, String>) -> Vec<u32> {
    let raw = bindings
        .get("pdf_selected_pages")
        .cloned()
        .unwrap_or_default();
    raw.split(',')
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .collect()
}

fn parse_u32_binding(bindings: &HashMap<String, String>, key: &str) -> Option<u32> {
    bindings.get(key).and_then(|v| v.trim().parse::<u32>().ok())
}

fn parse_u64_binding(bindings: &HashMap<String, String>, key: &str) -> Option<u64> {
    bindings.get(key).and_then(|v| v.trim().parse::<u64>().ok())
}

fn parse_f64_binding(bindings: &HashMap<String, String>, key: &str) -> Option<f64> {
    bindings.get(key).and_then(|v| v.trim().parse::<f64>().ok())
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
            primary_fd: None,
            primary_path: None,
            angle_radians: None,
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
            let snap =
                serde_json::to_string(&*state).map_err(|e| format!("snapshot_failed:{e}"))?;
            return Ok(json!({
                "type": "Snapshot",
                "snapshot": snap
            }));
        }
        Action::Restore { snapshot } => match serde_json::from_str::<AppState>(&snapshot) {
            Ok(mut restored) => {
                restored.ensure_navigation();
                *state = restored;
            }
            Err(e) => {
                state.last_error = Some(format!("restore_failed:{e}"));
            }
        },
        Action::Reset => {
            state.reset_runtime();
            state.reset_navigation();
        }
        Action::Back => {
            // Guardrail: never allow empty nav stack.
            state.pop_screen();
            if state.nav_depth() == 0 {
                state.reset_navigation();
            }
            state.loading_message = None;
        }
        Action::ArchiveToolsScreen => {
            state.push_screen(Screen::ArchiveTools);
            state.archive.reset();
        }
        Action::ArchiveOpen { fd, path, error } => {
            state.push_screen(Screen::ArchiveTools);
            let mut fd_handle = FdHandle::new(fd);
            if let Some(err) = error {
                state.archive.error = Some(err);
            } else if let Some(raw_fd) = fd_handle.take() {
                if let Err(e) = handle_archive_open(&mut state, raw_fd, path.as_deref()) {
                    state.archive.error = Some(e);
                }
            } else {
                state.archive.error = Some("missing_fd".into());
            }
        }
        Action::ArchiveOpenText { index } => {
            state.push_screen(Screen::TextViewer);
            match features::archive::read_text_entry(&state, index) {
                Ok((label, text)) => {
                    state.text_view_path = Some(label);
                    state.text_view_content = Some(text);
                    state.text_view_error = None;
                    if let Some(entry) = state.archive.entries.get(index) {
                        state.text_view_language = guess_language_from_path(&entry.name);
                    } else {
                        state.text_view_language = None;
                    }
                }
                Err(e) => {
                    state.text_view_error = Some(e);
                    state.text_view_content = None;
                    state.text_view_language = None;
                    if let Some(entry) = state.archive.entries.get(index) {
                        state.text_view_path = state
                            .archive
                            .path
                            .as_ref()
                            .map(|p| format!("{} ⟂ {}", entry.name, p))
                            .or_else(|| Some(entry.name.clone()));
                    }
                }
            }
        }
        Action::CompassDemo => {
            state.push_screen(Screen::Compass);
        }
        Action::CompassSet {
            angle_radians,
            error,
        } => {
            let mut angle = angle_radians % std::f64::consts::TAU;
            if angle < 0.0 {
                angle += std::f64::consts::TAU;
            }
            state.compass_angle_radians = angle;
            state.compass_error = error;
            if matches!(state.current_screen(), Screen::Compass) {
                state.replace_current(Screen::Compass);
            }
        }
        Action::BarometerScreen => {
            state.push_screen(Screen::Barometer);
        }
        Action::BarometerSet { hpa, error } => {
            state.barometer_hpa = Some(hpa);
            state.barometer_error = error;
            if matches!(state.current_screen(), Screen::Barometer) {
                state.replace_current(Screen::Barometer);
            }
        }
        Action::MagnetometerScreen => {
            state.push_screen(Screen::Magnetometer);
        }
        Action::MagnetometerSet {
            magnitude_ut,
            error,
        } => {
            state.magnetometer_ut = Some(magnitude_ut);
            state.magnetometer_error = error;
            if matches!(state.current_screen(), Screen::Magnetometer) {
                state.replace_current(Screen::Magnetometer);
            }
        }
        Action::PdfToolsScreen => {
            state.push_screen(Screen::PdfTools);
            state.pdf.last_error = None;
            state.pdf.last_output = None;
        }
        Action::PdfSelect { fd, uri, error } => {
            state.push_screen(Screen::PdfTools);
            state.pdf.last_error = error.clone();
            let mut fd_handle = FdHandle::new(fd);
            if error.is_none() {
                if let Some(raw_fd) = fd_handle.take() {
                    if let Err(e) = handle_pdf_select(&mut state, Some(raw_fd), uri.as_deref()) {
                        state.pdf.last_error = Some(e);
                    }
                } else {
                    state.pdf.last_error = Some("missing_fd".into());
                }
            }
        }
        Action::PdfExtract { fd, uri, selection } => {
            state.push_screen(Screen::PdfTools);
            let mut fd_handle = FdHandle::new(fd);
            if selection.is_empty() {
                state.pdf.last_error = Some("no_pages_selected".into());
            } else if let Some(raw_fd) = fd_handle.take() {
                match handle_pdf_operation(
                    &mut state,
                    PdfOperation::Extract,
                    Some(raw_fd),
                    None,
                    uri.as_deref(),
                    None,
                    &selection,
                ) {
                    Ok(_) => {}
                    Err(e) => state.pdf.last_error = Some(e),
                }
            } else {
                state.pdf.last_error = Some("missing_fd".into());
            }
        }
        Action::PdfDelete { fd, uri, selection } => {
            state.push_screen(Screen::PdfTools);
            let mut fd_handle = FdHandle::new(fd);
            if selection.is_empty() {
                state.pdf.last_error = Some("no_pages_selected".into());
            } else if let Some(raw_fd) = fd_handle.take() {
                match handle_pdf_operation(
                    &mut state,
                    PdfOperation::Delete,
                    Some(raw_fd),
                    None,
                    uri.as_deref(),
                    None,
                    &selection,
                ) {
                    Ok(_) => {}
                    Err(e) => state.pdf.last_error = Some(e),
                }
            } else {
                state.pdf.last_error = Some("missing_fd".into());
            }
        }
        Action::PdfMerge {
            primary_fd,
            primary_uri,
            secondary_fd,
            secondary_uri,
        } => {
            state.push_screen(Screen::PdfTools);
            let mut primary = FdHandle::new(primary_fd);
            let mut secondary = FdHandle::new(secondary_fd);
            if let (Some(p_fd), Some(s_fd)) = (primary.take(), secondary.take()) {
                match handle_pdf_operation(
                    &mut state,
                    PdfOperation::Merge,
                    Some(p_fd),
                    Some(s_fd),
                    primary_uri.as_deref(),
                    secondary_uri.as_deref(),
                    &[],
                ) {
                    Ok(_) => {}
                    Err(e) => state.pdf.last_error = Some(e),
                }
            } else {
                state.pdf.last_error = Some("missing_fd".into());
            }
        }
        Action::PdfSetTitle { fd, uri, title } => {
            state.push_screen(Screen::PdfTools);
            let mut fd_handle = FdHandle::new(fd);
            if let Some(raw_fd) = fd_handle.take() {
                if let Err(e) =
                    handle_pdf_title(&mut state, raw_fd, uri.as_deref(), title.as_deref())
                {
                    state.pdf.last_error = Some(e);
                }
            } else {
                state.pdf.last_error = Some("missing_fd".into());
            }
        }
        Action::PdfSign {
            fd,
            uri,
            signature,
            page,
            page_x_pct,
            page_y_pct,
            pos_x,
            pos_y,
            width,
            height,
            img_width_px,
            img_height_px,
            img_dpi,
        } => {
            state.push_screen(Screen::PdfTools);
            let mut fd_handle = FdHandle::new(fd);
            if let Some(sig) = signature.or_else(|| state.pdf.signature_base64.clone()) {
                if let Some(raw_fd) = fd_handle.take() {
                    match handle_pdf_sign(
                        &mut state,
                        raw_fd,
                        uri.as_deref(),
                        &sig,
                        page,
                        page_x_pct,
                        page_y_pct,
                        pos_x,
                        pos_y,
                        width,
                        height,
                        img_width_px,
                        img_height_px,
                        img_dpi,
                    ) {
                        Ok(_) => {}
                        Err(e) => state.pdf.last_error = Some(e),
                    }
                } else {
                    state.pdf.last_error = Some("missing_fd".into());
                }
            } else {
                state.pdf.last_error = Some("missing_signature".into());
            }
        }
        Action::PdfSignGrid { page, x_pct, y_pct } => {
            state.push_screen(Screen::PdfTools);
            state.pdf.signature_target_page = Some(page);
            state.pdf.signature_x_pct = Some(x_pct);
            state.pdf.signature_y_pct = Some(y_pct);
            state.pdf.signature_grid_selection = Some((page, x_pct, y_pct));
        }
        Action::HashVerifyScreen => {
            state.push_screen(Screen::HashVerify);
            state.hash_reference = None;
            state.hash_match = None;
            state.last_hash = None;
            state.last_hash_algo = Some("SHA-256".into());
        }
        Action::HashVerify { path, fd, reference } => {
            let mut fd_handle = FdHandle::new(fd);
            state.push_screen(Screen::HashVerify);
            if let Some(err) = reference.as_ref().filter(|s| s.trim().is_empty()).map(|_| "reference_empty".to_string()) {
                state.last_error = Some(err);
                state.hash_match = None;
            } else {
                let algo = HashAlgo::Sha256;
                if let Some(err) = reference.clone().is_none().then(|| "missing_reference".to_string()) {
                    state.last_error = Some(err);
                } else {
                    handle_hash_verify(
                        &mut state,
                        fd_handle.take(),
                        path.as_deref(),
                        reference.as_deref().unwrap(),
                        algo,
                    );
                }
            }
        }
        Action::HashVerifyPaste { reference } => {
            state.push_screen(Screen::HashVerify);
            if let Some(text) = reference {
                state.hash_reference = Some(text);
                state.hash_match = None;
                state.last_hash = None;
                state.last_error = None;
            } else {
                state.last_error = Some("clipboard_empty".into());
            }
        }
        Action::PdfSignatureStore { data } => {
            state.pdf.signature_base64 = data;
            state.pdf.signature_width_pt = None;
            state.pdf.signature_height_pt = None;
            state.pdf.last_error = None;
            state.push_screen(Screen::PdfTools);
        }
        Action::PdfSignatureClear => {
            state.pdf.signature_base64 = None;
            state.pdf.signature_width_pt = None;
            state.pdf.signature_height_pt = None;
            state.pdf.last_error = None;
            state.push_screen(Screen::PdfTools);
        }
        Action::About => {
            state.push_screen(Screen::About);
        }
        Action::TextViewerScreen => {
            state.push_screen(Screen::TextViewer);
            state.text_view_error = None;
            state.text_view_language = None;
            state.text_view_hex_preview = None;
            state.text_view_loaded_bytes = 0;
            state.text_view_total_bytes = None;
            state.text_view_has_more = false;
            state.text_view_window_offset = 0;
            state.text_view_has_previous = false;
            state.text_view_cached_path = None;
        }
        Action::TextViewerOpen { fd, path, error } => {
            state.push_screen(Screen::TextViewer);
            state.text_view_error = error.clone();
            state.text_view_find_query = None;
            state.text_view_find_match = None;
            state.text_view_loaded_bytes = 0;
            state.text_view_total_bytes = None;
            state.text_view_has_more = false;
            state.text_view_window_offset = 0;
            state.text_view_has_previous = false;
            state.text_view_cached_path = None;
            let mut fd_handle = FdHandle::new(fd);
            if error.is_some() {
                state.text_view_content = None;
                state.text_view_language = None;
                state.text_view_hex_preview = None;
            } else if let Some(raw_fd) = fd_handle.take() {
                load_text_from_fd(&mut state, raw_fd as RawFd, path.as_deref());
            } else if let Some(p) = path.as_deref() {
                load_text_from_path(&mut state, p);
            } else {
                state.text_view_error = Some("missing_source".into());
                state.text_view_content = None;
                state.text_view_language = None;
                state.text_view_hex_preview = None;
            }
        }
        Action::TextViewerToggleTheme => {
            state.text_view_dark = !state.text_view_dark;
            state.replace_current(Screen::TextViewer);
        }
        Action::TextViewerToggleLineNumbers => {
            state.text_view_line_numbers = !state.text_view_line_numbers;
            state.replace_current(Screen::TextViewer);
        }
        Action::TextViewerLoadAnyway => {
            // Clear hex preview and reload last path as text.
            state.text_view_hex_preview = None;
            if let Some(path) = state.text_view_path.clone() {
                let effective = state.text_view_cached_path.clone().unwrap_or(path.clone());
                load_text_from_path_at_offset(&mut state, &effective, 0, true);
            } else {
                state.text_view_error = Some("nothing_to_reload".into());
                state.text_view_content = None;
            }
            state.replace_current(Screen::TextViewer);
        }
        Action::TextViewerLoadMore => {
            load_more_text(&mut state);
            state.replace_current(Screen::TextViewer);
        }
        Action::TextViewerLoadPrev => {
            load_prev_text(&mut state);
            state.replace_current(Screen::TextViewer);
        }
        Action::TextViewerJump { offset } => {
            let target = offset.unwrap_or(0);
            if let Some(path) = state.text_view_path.clone() {
                let effective = state.text_view_cached_path.clone().unwrap_or(path.clone());
                let clamped = state
                    .text_view_total_bytes
                    .map(|total| {
                        let window = features::text_viewer::CHUNK_BYTES as u64;
                        let max_offset = total.saturating_sub(window.min(total));
                        target.min(max_offset)
                    })
                    .unwrap_or(target);
                load_text_from_path_at_offset(&mut state, &effective, clamped, true);
            } else {
                state.text_view_error = Some("missing_path".into());
            }
            state.replace_current(Screen::TextViewer);
        }
        Action::TextViewerFind { query, direction } => {
            if let Some(q) = query {
                let trimmed = q.trim();
                if trimmed.is_empty() {
                    state.text_view_find_query = None;
                    state.text_view_find_match = Some("Cleared search".into());
                } else {
                    state.text_view_find_query = Some(trimmed.to_string());
                    state.text_view_find_match = None;
                }
            }
            if let Some(dir) = direction {
                state.text_view_find_match = Some(
                    match dir.as_str() {
                        "next" => "Searching next match",
                        "prev" => "Searching previous match",
                        _ => "Searching",
                    }
                    .into(),
                );
            }
            state.replace_current(Screen::TextViewer);
        }
        Action::SensorLoggerScreen => {
            state.push_screen(Screen::SensorLogger);
        }
        Action::SensorLoggerStart { bindings } => {
            match parse_sensor_bindings(&bindings) {
                Ok(cfg) => {
                    state.last_error = None;
                    state.sensor_status = Some("logging".into());
                    state.sensor_interval_ms = Some(cfg.interval_ms);
                    state.sensor_selection = Some(cfg.selection);
                }
                Err(e) => {
                    state.last_error = Some(e);
                }
            }
            if matches!(state.current_screen(), Screen::SensorLogger) {
                state.replace_current(Screen::SensorLogger);
            }
        }
        Action::SensorLoggerStop => {
            state.last_error = None;
            state.sensor_status = Some("stopped".into());
            if matches!(state.current_screen(), Screen::SensorLogger) {
                state.replace_current(Screen::SensorLogger);
            }
        }
        Action::SensorLoggerShare => {
            // handled in Kotlin; Rust just keeps screen
            if matches!(state.current_screen(), Screen::SensorLogger) {
                state.replace_current(Screen::SensorLogger);
            }
        }
        Action::SensorLoggerStatus { bindings } => {
            apply_status_from_bindings(&mut state, &bindings);
            if matches!(state.current_screen(), Screen::SensorLogger) {
                state.replace_current(Screen::SensorLogger);
            }
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
        Action::KotlinImageResizeScreen => handle_kotlin_image_resize_screen(&mut state),
        Action::KotlinImageResizeSync { bindings } => {
            handle_kotlin_image_resize_sync(&mut state, &bindings);
        }
        Action::KotlinImageResult {
            target,
            result,
            bindings,
        } => handle_kotlin_image_result(&mut state, target, result, Some(&bindings)),
        Action::KotlinImageOutputDir { target, output_dir } => {
            handle_kotlin_image_output_dir(&mut state, target, output_dir);
        }
        Action::QrGenerate { input } => {
            state.push_screen(Screen::Qr);
            let text = input.unwrap_or_default();
            if let Err(e) = handle_qr_action(&mut state, &text) {
                state.last_error = Some(e);
            }
        }
        Action::ColorFromHex { input } => {
            state.push_screen(Screen::ColorTools);
            let txt = input.unwrap_or_default();
            handle_color_action(&mut state, "color_from_hex", &txt);
        }
        Action::ColorFromRgb { input } => {
            state.push_screen(Screen::ColorTools);
            let txt = input.unwrap_or_default();
            handle_color_action(&mut state, "color_from_rgb", &txt);
        }
        Action::ColorCopyHexInput { input } => {
            state.push_screen(Screen::ColorTools);
            let val = input
                .or_else(|| state.text_input.clone())
                .unwrap_or_default();
            handle_color_action(&mut state, "color_copy_hex_input", &val);
        }
        Action::ColorCopyClipboard => {
            state.push_screen(Screen::ColorTools);
            // no-op in Rust; Kotlin handles clipboard using cached Result text
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
        Screen::HashVerify => render_hash_verify_screen(state),
        Screen::FileInfo => render_file_info_screen(state),
        Screen::TextTools => render_text_tools_screen(state),
        Screen::Loading => render_loading_screen(state),
        Screen::ProgressDemo => render_progress_demo_screen(state),
        Screen::Qr => render_qr_screen(state),
        Screen::ColorTools => render_color_screen(state),
        Screen::PdfTools => render_pdf_screen(state),
        Screen::About => render_about_screen(state),
        Screen::SensorLogger => render_sensor_logger_screen(state),
        Screen::TextViewer => render_text_viewer_screen(state),
        Screen::ArchiveTools => render_archive_screen(state),
        Screen::Compass => render_compass_screen(state),
        Screen::Barometer => render_barometer_screen(state),
        Screen::Magnetometer => render_magnetometer_screen(state),
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
        serde_json::to_value(UiText::new("Select a file to see its size and MIME type").size(14.0))
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

fn render_hash_verify_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("Hash verify (SHA-256)").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Paste a reference hash, then pick a file to verify.").size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Copy last hash", "noop")
                .id("copy_last_hash_btn")
                .copy_text(state.last_hash.as_deref().unwrap_or("")),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Paste from clipboard", "hash_verify_paste")
                .id("hash_verify_paste")
                .content_description("hash_verify_paste"),
        )
        .unwrap(),
        serde_json::to_value(
            crate::ui::TextInput::new("hash_reference")
                .hint("Reference hash")
                .text(state.hash_reference.as_deref().unwrap_or_default())
                .single_line(true),
        )
        .unwrap(),
        serde_json::to_value(
            UiButton::new("Pick file and verify", "hash_verify")
                .requires_file_picker(true)
                .id("hash_verify_btn"),
        )
        .unwrap(),
    ];

    if let Some(matches) = state.hash_match {
        let status = if matches { "Match ✅" } else { "Mismatch ❌" };
        children.push(
            serde_json::to_value(UiText::new(status).size(14.0).content_description("hash_verify_status"))
                .unwrap(),
        );
    }
    if let Some(hash) = &state.last_hash {
        children.push(
            serde_json::to_value(
                UiText::new(&format!(
                    "{}: {}",
                    state
                        .last_hash_algo
                        .clone()
                        .unwrap_or_else(|| "SHA-256".into()),
                    hash
                ))
                .size(12.0),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Copy computed hash", "hash_verify_copy").copy_text(hash),
            )
            .unwrap(),
        );
    }
    if let Some(err) = &state.last_error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {}", err)).size(12.0)).unwrap(),
        );
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
        children.push(
            serde_json::to_value(UiProgress::new().content_description("In progress")).unwrap(),
        );
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

fn render_compass_screen(state: &AppState) -> Value {
    let degrees = state.compass_angle_radians.to_degrees();
    let mut children = vec![
        serde_json::to_value(UiText::new("Compass (AGSL)").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Compass dial driven by device sensors. Heading auto-updates when sensors are available.")
                .size(12.0),
        )
        .unwrap(),
        serde_json::to_value(UiText::new(&format!("Heading: {:.1}°", degrees)).size(14.0)).unwrap(),
        serde_json::to_value(UiCompass::new(state.compass_angle_radians)).unwrap(),
        serde_json::to_value(
            UiText::new(
                state
                    .compass_error
                    .as_deref()
                    .unwrap_or("Sensor updates will appear automatically.")
            )
            .size(12.0),
        )
        .unwrap(),
    ];
    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

fn render_barometer_screen(state: &AppState) -> Value {
    let reading = state.barometer_hpa.map(|v| format!("{:.1} hPa", v));
    let mut children = vec![
        serde_json::to_value(UiText::new("Barometer").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(
                state
                    .barometer_error
                    .as_deref()
                    .unwrap_or("Live pressure readout from the device barometer (if present)."),
            )
            .size(12.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiText::new(reading.as_deref().unwrap_or("Waiting for sensor...")).size(14.0),
        )
        .unwrap(),
        serde_json::to_value(UiBarometer::new(state.barometer_hpa.unwrap_or(0.0))).unwrap(),
    ];
    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

fn render_magnetometer_screen(state: &AppState) -> Value {
    let reading = state
        .magnetometer_ut
        .map(|v| format!("{:.1} µT", v))
        .unwrap_or_else(|| "Waiting for sensor...".into());
    let mut children = vec![
        serde_json::to_value(UiText::new("Magnetometer").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(
                state
                    .magnetometer_error
                    .as_deref()
                    .unwrap_or("Live magnetic field strength (device sensor)."),
            )
            .size(12.0),
        )
        .unwrap(),
        serde_json::to_value(UiText::new(&reading).size(14.0)).unwrap(),
        serde_json::to_value(UiMagnetometer::new(state.magnetometer_ut.unwrap_or(0.0))).unwrap(),
    ];
    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
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

fn render_about_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("About Kistaverk").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(&format!("Version: {}", env!("CARGO_PKG_VERSION"))).size(14.0),
        )
        .unwrap(),
        serde_json::to_value(UiText::new("Copyright © 2025 Kistaverk").size(14.0)).unwrap(),
        serde_json::to_value(UiText::new("License: GPLv3").size(14.0)).unwrap(),
        serde_json::to_value(
            UiText::new("This app is open-source under GPL-3.0; contributions welcome.").size(12.0),
        )
        .unwrap(),
        serde_json::to_value(UiDepsList::new()).unwrap(),
    ];
    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(24)).unwrap()
}

fn render_sensor_logger_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("Sensor Logger").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new("Select sensors and start logging to CSV in app storage.").size(14.0),
        )
        .unwrap(),
        serde_json::to_value(UiText::new("Sensors").size(14.0)).unwrap(),
        serde_json::to_value(
            UiColumn::new(vec![
                serde_json::to_value(
                    ui::Checkbox::new("Accelerometer", "sensor_accel")
                        .checked(state.sensor_selection.map(|s| s.accel).unwrap_or(true)),
                )
                .unwrap(),
                serde_json::to_value(
                    ui::Checkbox::new("Gyroscope", "sensor_gyro")
                        .checked(state.sensor_selection.map(|s| s.gyro).unwrap_or(true)),
                )
                .unwrap(),
                serde_json::to_value(
                    ui::Checkbox::new("Magnetometer", "sensor_mag")
                        .checked(state.sensor_selection.map(|s| s.mag).unwrap_or(true)),
                )
                .unwrap(),
                serde_json::to_value(
                    ui::Checkbox::new("Barometer", "sensor_pressure")
                        .checked(state.sensor_selection.map(|s| s.pressure).unwrap_or(false)),
                )
                .unwrap(),
                serde_json::to_value(
                    ui::Checkbox::new("GPS", "sensor_gps")
                        .checked(state.sensor_selection.map(|s| s.gps).unwrap_or(false)),
                )
                .unwrap(),
                serde_json::to_value(
                    ui::Checkbox::new("Battery", "sensor_battery")
                        .checked(state.sensor_selection.map(|s| s.battery).unwrap_or(true)),
                )
                .unwrap(),
            ])
            .padding(8),
        )
        .unwrap(),
        serde_json::to_value(
            ui::TextInput::new("sensor_interval_ms")
                .hint("Interval ms (50-10000)")
                .text(
                    &state
                        .sensor_interval_ms
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "200".into()),
                )
                .content_description("Sensor interval ms"),
        )
        .unwrap(),
        serde_json::to_value(UiButton::new("Start logging", "sensor_logger_start")).unwrap(),
        serde_json::to_value(UiButton::new("Stop logging", "sensor_logger_stop")).unwrap(),
    ];

    if let Some(status) = &state.sensor_status {
        children.push(
            serde_json::to_value(UiText::new(&format!("Status: {}", status)).size(12.0)).unwrap(),
        );
    }
    if state.sensor_status.as_deref() == Some("logging") {
        children.push(
            serde_json::to_value(
                UiWarning::new("Logging continues in a foreground service.")
                    .content_description("sensor_logger_foreground_status"),
            )
            .unwrap(),
        );
    }
    if let Some(err) = &state.last_error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {}", err)).size(12.0)).unwrap(),
        );
    }
    if let Some(path) = &state.last_sensor_log {
        children.push(
            serde_json::to_value(UiText::new(&format!("Last log: {}", path)).size(12.0)).unwrap(),
        );
        children.push(
            serde_json::to_value(UiButton::new("Share last log", "sensor_logger_share")).unwrap(),
        );
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    if bytes as f64 >= MB {
        format!("{:.1} MB", bytes as f64 / MB)
    } else if bytes as f64 >= KB {
        format!("{:.1} KB", bytes as f64 / KB)
    } else {
        format!("{bytes} B")
    }
}

fn render_text_viewer_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new("Text viewer").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(
                "Open a text/CSV/log file and preview it in 128 KB chunks with syntax highlighting.",
            )
            .size(14.0),
        )
        .unwrap(),
        json!({
            "type": "Button",
            "text": "Pick text file",
            "action": "text_viewer_open",
            "requires_file_picker": true,
            "content_description": "Pick text or CSV file"
        }),
    ];

    if let Some(path) = &state.text_view_path {
        children.push(
            serde_json::to_value(UiText::new(&format!("File: {}", path)).size(12.0)).unwrap(),
        );
    }

    if state.text_view_loaded_bytes > 0 || state.text_view_total_bytes.is_some() {
        let start = state.text_view_window_offset;
        let end = state.text_view_loaded_bytes;
        let window_size = end.saturating_sub(start);
        let status = if let Some(total) = state.text_view_total_bytes {
            let pct = if total > 0 {
                (end as f64 / total as f64 * 100.0).min(100.0)
            } else {
                100.0
            };
            format!(
                "Showing {}–{} of {} ({} window, {:.1}%)",
                format_bytes(start),
                format_bytes(end),
                format_bytes(total),
                format_bytes(window_size),
                pct
            )
        } else {
            format!(
                "Showing {}–{} ({} window, chunked preview)",
                format_bytes(start),
                format_bytes(end),
                format_bytes(window_size)
            )
        };
        children.push(serde_json::to_value(UiText::new(&status).size(12.0)).unwrap());
    }

    if state.text_view_has_previous || state.text_view_has_more {
        children.push(
            serde_json::to_value(
                json!({
                    "type": "Grid",
                    "columns": 2,
                    "padding": 4,
                    "children": [
                        { "type": "Button", "text": "Load previous", "action": "text_viewer_load_prev", "id": "text_viewer_load_prev", "content_description": "text_viewer_load_prev" },
                        { "type": "Button", "text": "Load next", "action": "text_viewer_load_more", "id": "text_viewer_load_more", "content_description": "text_viewer_load_more" }
                    ]
                })
            )
            .unwrap(),
        );
    }

    children.push(
        serde_json::to_value(json!({
            "type": "Grid",
            "columns": 2,
            "padding": 4,
            "children": [
                {
                    "type": "TextInput",
                    "bind_key": "offset_bytes",
                    "hint": "Byte offset (0 = start)",
                    "text": state.text_view_window_offset.to_string(),
                    "single_line": true,
                    "action_on_submit": "text_viewer_jump"
                },
                {
                    "type": "Button",
                    "text": "Jump",
                    "action": "text_viewer_jump",
                    "content_description": "text_viewer_jump"
                }
            ]
        }))
        .unwrap(),
    );

    // Find bar
    children.push(
        serde_json::to_value(
            UiColumn::new(vec![
                serde_json::to_value(UiText::new("Find in text").size(14.0)).unwrap(),
                serde_json::to_value(
                    UiColumn::new(vec![
                        json!({
                            "type": "TextInput",
                            "bind_key": "find_query",
                            "text": state
                                .text_view_find_query
                                .as_deref()
                                .unwrap_or(""),
                            "hint": "Enter search term",
                            "action_on_submit": "text_viewer_find_submit",
                            "single_line": true
                        }),
                        json!({
                            "type": "Grid",
                            "columns": 3,
                            "children": [
                                { "type": "Button", "text": "Prev", "action": "text_viewer_find_prev", "id": "find_prev", "content_description": "find_prev" },
                                { "type": "Button", "text": "Next", "action": "text_viewer_find_next", "id": "find_next", "content_description": "find_next" },
                                { "type": "Button", "text": "Clear", "action": "text_viewer_find_clear", "id": "find_clear", "content_description": "find_clear" }
                            ]
                        }),
                    ])
                    .padding(4),
                )
                .unwrap(),
                serde_json::to_value(
                    UiText::new(
                        state
                            .text_view_find_match
                            .as_deref()
                            .unwrap_or("Type a query and tap next/prev."),
                    )
                    .id("find_status")
                    .size(12.0),
                )
                .unwrap(),
            ])
            .padding(8),
        )
        .unwrap(),
    );

    let theme_label = if state.text_view_dark {
        "Switch to light"
    } else {
        "Switch to dark"
    };
    children.push(
        serde_json::to_value(
            UiButton::new(theme_label, "text_viewer_toggle_theme")
                .content_description("text_viewer_toggle_theme"),
        )
        .unwrap(),
    );
    let ln_label = if state.text_view_line_numbers {
        "Hide line numbers"
    } else {
        "Show line numbers"
    };
    children.push(
        serde_json::to_value(
            UiButton::new(ln_label, "text_viewer_toggle_line_numbers")
                .content_description("text_viewer_toggle_line_numbers"),
        )
        .unwrap(),
    );

    if let Some(err) = &state.text_view_error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {}", err)).size(12.0)).unwrap(),
        );
    }

    if let Some(hex) = &state.text_view_hex_preview {
        children.push(
            serde_json::to_value(
                crate::ui::Warning::new(
                    "Binary or unsupported text detected. Showing hex preview (first 4KB).",
                )
                .content_description("text_viewer_hex_warning"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiCodeView::new(hex)
                    .wrap(false)
                    .theme(if state.text_view_dark {
                        "dark"
                    } else {
                        "light"
                    })
                    .line_numbers(false),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Load anyway (may be slow)", "text_viewer_load_anyway")
                    .content_description("text_viewer_load_anyway"),
            )
            .unwrap(),
        );
    }

    if let Some(content) = &state.text_view_content {
        let mut lang = state.text_view_language.clone();
        if lang.is_none() {
            if let Some(path) = &state.text_view_path {
                lang = guess_language_from_path(path);
            }
        }
        let theme = if state.text_view_dark {
            "dark"
        } else {
            "light"
        };
        let mut code = UiCodeView::new(content)
            .wrap(true)
            .theme(theme)
            .line_numbers(state.text_view_line_numbers);
        if let Some(lang_str) = lang.as_deref() {
            code = code.language(lang_str);
        }
        children.push(serde_json::to_value(code).unwrap());
    }

    if state.text_view_has_more {
        children.push(
            serde_json::to_value(
                UiButton::new("Load more", "text_viewer_load_more")
                    .id("text_viewer_load_more")
                    .content_description("text_viewer_load_more"),
            )
            .unwrap(),
        );
    }

    maybe_push_back(&mut children, state);

    serde_json::to_value(UiColumn::new(children).padding(20).scrollable(false)).unwrap()
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
            id: "hash_verify",
            name: "✅ Verify hash",
            category: "🔐 Hashes",
            action: "hash_verify_screen",
            requires_file_picker: false,
            description: "compare to reference",
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
            id: "text_viewer",
            name: "📜 Text viewer",
            category: "📁 Files",
            action: "text_viewer_screen",
            requires_file_picker: true,
            description: "preview text/CSV",
        },
        Feature {
            id: "archive_tools",
            name: "📦 Archive Viewer",
            category: "📁 Files",
            action: "archive_tools_screen",
            requires_file_picker: false,
            description: "list .zip contents",
        },
        Feature {
            id: "pdf_tools",
            name: "📄 PDF pages",
            category: "📁 Files",
            action: "pdf_tools_screen",
            requires_file_picker: false,
            description: "extract/delete pages",
        },
        Feature {
            id: "image_resize_kotlin",
            name: "📉 Image resize (Kotlin)",
            category: "📸 Media",
            action: "kotlin_image_resize_screen",
            requires_file_picker: false,
            description: "shrink for sharing",
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
            id: "compass_demo",
            name: "🧭 Compass",
            category: "🧪 Experiments",
            action: "compass_demo",
            requires_file_picker: false,
            description: "Sensor-driven dial",
        },
        Feature {
            id: "barometer",
            name: "🌡️ Barometer",
            category: "🧪 Experiments",
            action: "barometer_screen",
            requires_file_picker: false,
            description: "Pressure sensor",
        },
        Feature {
            id: "magnetometer",
            name: "🧲 Magnetometer",
            category: "🧪 Experiments",
            action: "magnetometer_screen",
            requires_file_picker: false,
            description: "Field strength",
        },
        Feature {
            id: "text_tools",
            name: "✍️ Text tools",
            category: "📝 Text",
            action: "text_tools_screen",
            requires_file_picker: false,
            description: "case & counts",
        },
        Feature {
            id: "qr_generator",
            name: "🔳 QR Generator",
            category: "🧪 Experiments",
            action: "qr_generate",
            requires_file_picker: false,
            description: "encode text → QR",
        },
        Feature {
            id: "color_converter",
            name: "🎨 Color Converter",
            category: "🧪 Experiments",
            action: "color_from_hex",
            requires_file_picker: false,
            description: "Hex ↔ RGB/HSL",
        },
        Feature {
            id: "sensor_logger",
            name: "📡 Sensor Logger",
            category: "🧪 Experiments",
            action: "sensor_logger_screen",
            requires_file_picker: false,
            description: "log sensors to CSV",
        },
        Feature {
            id: "about",
            name: "ℹ️ About",
            category: "Info",
            action: "about",
            requires_file_picker: false,
            description: "version & license",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::sensor_logger::parse_bindings as parse_sensor_bindings;
    use crate::ui::{Card as UiCard, Section as UiSection, Text as UiText};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::os::unix::io::IntoRawFd;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;
    use zip::write::FileOptions;

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
            primary_fd: None,
            primary_path: None,
            angle_radians: None,
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
    fn section_and_card_serialize_with_headers() {
        let body = vec![serde_json::to_value(UiText::new("Body")).unwrap()];
        let section = UiSection::new(body.clone())
            .title("📁 Files")
            .subtitle("2 tools")
            .icon("📁")
            .padding(8);
        let card = UiCard::new(body).title("⚡ Quick access").padding(6);

        let section_val = serde_json::to_value(section).expect("section should serialize");
        assert_eq!(
            section_val.get("type"),
            Some(&Value::String("Section".into()))
        );
        assert_eq!(
            section_val.get("title"),
            Some(&Value::String("📁 Files".into()))
        );
        assert_eq!(section_val.get("icon"), Some(&Value::String("📁".into())));
        assert!(section_val
            .get("children")
            .and_then(|c| c.as_array())
            .is_some());

        let card_val = serde_json::to_value(card).expect("card should serialize");
        assert_eq!(card_val.get("type"), Some(&Value::String("Card".into())));
        assert!(card_val
            .get("children")
            .and_then(|c| c.as_array())
            .is_some());
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
    fn sensor_bindings_default_and_interval_clamp() {
        let cfg =
            parse_sensor_bindings(&HashMap::from([("sensor_interval_ms".into(), "5".into())]))
                .expect("parse should succeed with defaults");
        assert!(cfg.selection.accel);
        assert!(cfg.selection.gyro);
        assert!(cfg.selection.mag);
        assert!(cfg.selection.battery);
        assert!(!cfg.selection.gps);
        assert_eq!(cfg.interval_ms, 200);
    }

    #[test]
    fn sensor_bindings_require_selection() {
        let err = parse_sensor_bindings(&HashMap::from([
            ("sensor_accel".into(), "false".into()),
            ("sensor_gyro".into(), "false".into()),
            ("sensor_mag".into(), "false".into()),
            ("sensor_pressure".into(), "false".into()),
            ("sensor_gps".into(), "false".into()),
            ("sensor_battery".into(), "false".into()),
        ]))
        .unwrap_err();
        assert_eq!(err, "no_sensor_selected");
    }

    #[test]
    fn sensor_screen_renders_share_button_when_path_present() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        handle_command(make_command("sensor_logger_screen")).unwrap();

        let mut cmd = make_command("sensor_logger_status");
        cmd.bindings = Some(HashMap::from([
            ("sensor_status".into(), "logging".into()),
            ("sensor_path".into(), "/tmp/sensors.csv".into()),
        ]));

        let ui = handle_command(cmd).expect("status command should succeed");
        assert_contains_text(&ui, "Share last log");
        let state = STATE.lock().unwrap();
        assert_eq!(state.last_sensor_log.as_deref(), Some("/tmp/sensors.csv"));
        assert_eq!(state.sensor_status.as_deref(), Some("logging"));
    }

    #[test]
    fn sensor_logger_foreground_indicator_shown_when_logging() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        handle_command(make_command("sensor_logger_screen")).unwrap();
        let mut status = make_command("sensor_logger_status");
        status.bindings = Some(HashMap::from([
            ("sensor_status".into(), "logging".into()),
            ("sensor_path".into(), "/tmp/sensors.csv".into()),
        ]));

        let ui = handle_command(status).expect("status command should succeed");
        assert_contains_text(&ui, "foreground service");
    }

    #[test]
    fn text_viewer_reads_file_via_fd() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "hello from csv").unwrap();
        let fd = File::open(file.path()).unwrap().into_raw_fd();

        let mut cmd = make_command("text_viewer_open");
        cmd.fd = Some(fd);
        cmd.path = Some(file.path().to_string_lossy().into_owned());

        let ui = handle_command(cmd).expect("text viewer should succeed");
        assert_contains_text(&ui, "hello from csv");

        let state = STATE.lock().unwrap();
        assert_eq!(
            state.text_view_path.as_deref(),
            Some(file.path().to_string_lossy().as_ref())
        );
        assert!(state.text_view_error.is_none());
    }

    #[test]
    fn text_viewer_supports_chunked_loading() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut file = NamedTempFile::new().unwrap();
        let block = vec![b'a'; 140_000];
        file.write_all(&block).unwrap();
        file.write_all(&block).unwrap(); // ~280 KB
        file.flush().unwrap();
        let fd = File::open(file.path()).unwrap().into_raw_fd();

        let mut cmd = make_command("text_viewer_open");
        cmd.fd = Some(fd);
        cmd.path = Some(file.path().to_string_lossy().into_owned());

        handle_command(cmd).expect("text viewer should succeed");
        let state = STATE.lock().unwrap();
        let initial_loaded_end = state.text_view_loaded_bytes;
        let initial_offset = state.text_view_window_offset;
        let total = state.text_view_total_bytes.unwrap();
        assert_eq!(initial_offset, 0);
        let _initial_len = state
            .text_view_content
            .as_ref()
            .map(|c| c.len())
            .unwrap_or(0);
        assert!(state.text_view_has_more);
        assert!(initial_loaded_end < total);
        drop(state);

        handle_command(make_command("text_viewer_load_more")).expect("load more should succeed");
        let state = STATE.lock().unwrap();
        let after_len = state
            .text_view_content
            .as_ref()
            .map(|c| c.len())
            .unwrap_or(0);
        assert!(after_len > 0);
        assert!(after_len <= 150_000);
        assert_eq!(state.text_view_total_bytes, Some(total));
        assert!(state.text_view_window_offset > initial_offset);
        assert!(state.text_view_loaded_bytes > initial_loaded_end);
        assert!(state.text_view_has_previous);
        assert!(state.text_view_loaded_bytes - state.text_view_window_offset > 0);
        assert_eq!(
            state.text_view_content.as_ref().unwrap().chars().next(),
            Some('a')
        );
    }

    #[test]
    fn text_viewer_jump_and_prev_work() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut file = NamedTempFile::new().unwrap();
        let mut block = vec![b'a'; 64_000];
        block.extend(vec![b'b'; 64_000]);
        file.write_all(&block).unwrap();
        file.flush().unwrap();

        let mut cmd = make_command("text_viewer_open");
        cmd.path = Some(file.path().to_string_lossy().into_owned());
        handle_command(cmd).expect("text viewer should succeed");

        // Jump to second half
        let mut jump = make_command("text_viewer_jump");
        jump.bindings = Some(HashMap::from([("offset_bytes".into(), "64000".into())]));
        handle_command(jump).expect("jump should succeed");
        {
            let state = STATE.lock().unwrap();
            // If the file is smaller than a window, jump clamps to 0.
            assert!(state.text_view_window_offset <= 64_000);
            assert!(
                state.text_view_content.as_ref().unwrap().starts_with('b')
                    || state.text_view_content.as_ref().unwrap().contains('b')
            );
            assert!(state.text_view_has_previous || state.text_view_window_offset == 0);
        }

        // Load previous should move window back toward start
        handle_command(make_command("text_viewer_load_prev")).expect("prev should succeed");
        let state = STATE.lock().unwrap();
        assert_eq!(state.text_view_window_offset, 0);
        assert!(state.text_view_content.as_ref().unwrap().starts_with('a'));
    }

    #[test]
    fn archive_text_entry_opens_in_viewer() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut zip_file = NamedTempFile::new().unwrap();
        {
            let mut writer = zip::ZipWriter::new(&mut zip_file);
            let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
            writer.start_file("note.txt", options).unwrap();
            writer.write_all(b"hello from zip").unwrap();
            writer.start_file("data.bin", options).unwrap();
            writer.write_all(&[0u8, 1, 2]).unwrap();
            writer.finish().unwrap();
        }

        let fd = File::open(zip_file.path()).unwrap().into_raw_fd();
        let mut open_cmd = make_command("archive_open");
        open_cmd.fd = Some(fd);
        open_cmd.path = Some(zip_file.path().to_string_lossy().into_owned());
        handle_command(open_cmd).expect("archive open should succeed");

        let ui = handle_command(make_command("archive_open_text:0"))
            .expect("text entry open should succeed");
        assert_contains_text(&ui, "hello from zip");

        let state = STATE.lock().unwrap();
        assert!(state
            .text_view_path
            .as_deref()
            .map(|p| p.contains("note.txt"))
            .unwrap_or(false));
        assert_eq!(state.text_view_content.as_deref(), Some("hello from zip"));
        assert!(matches!(state.current_screen(), Screen::TextViewer));
        assert_eq!(state.nav_depth(), 3);
    }

    #[test]
    fn qr_screen_has_back_button_when_nested() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let mut cmd = make_command("qr_generate");
        cmd.bindings = Some(HashMap::from([("qr_input".into(), "hi".into())]));
        let ui = handle_command(cmd).expect("qr generate should succeed");

        assert_contains_text(&ui, "Back");
        let state = STATE.lock().unwrap();
        assert!(matches!(state.current_screen(), Screen::Qr));
        assert!(state.nav_depth() > 1);
    }

    #[test]
    fn sensor_logger_actions_do_not_stack_nav() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        handle_command(make_command("sensor_logger_screen")).unwrap();
        {
            let state = STATE.lock().unwrap();
            assert_eq!(state.nav_depth(), 2);
            assert!(matches!(state.current_screen(), Screen::SensorLogger));
        }

        let mut start = make_command("sensor_logger_start");
        start.bindings = Some(HashMap::from([("sensor_accel".into(), "true".into())]));
        handle_command(start).unwrap();
        {
            let state = STATE.lock().unwrap();
            assert_eq!(state.nav_depth(), 2);
            assert!(matches!(state.current_screen(), Screen::SensorLogger));
        }

        handle_command(make_command("back")).unwrap();
        let state = STATE.lock().unwrap();
        assert_eq!(state.nav_depth(), 1);
        assert!(matches!(state.current_screen(), Screen::Home));
    }

    #[test]
    fn text_viewer_missing_source_sets_error() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let ui = handle_command(make_command("text_viewer_open")).expect("should return UI");
        assert_contains_text(&ui, "missing_source");
        let state = STATE.lock().unwrap();
        assert_eq!(state.text_view_error.as_deref(), Some("missing_source"));
        assert!(state.text_view_content.is_none());
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
