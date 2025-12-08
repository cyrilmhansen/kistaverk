use crate::features;
use crate::features::archive::{self, render_archive_screen, ArchiveOpenResult};
use crate::features::color_tools::{handle_color_action, render_color_screen};
use crate::features::compression::{gzip_compress, gzip_decompress, render_compression_screen};
use crate::features::dithering::{process_dithering, render_dithering_screen, save_fd_to_temp};
use crate::features::file_info::{file_info_from_fd, file_info_from_path, render_file_info_screen};
use crate::features::hashes::{
    compute_all_hashes, compute_hash, render_hash_verify_screen, HashAlgo,
};
use crate::features::kotlin_image::{
    handle_output_dir as handle_kotlin_image_output_dir,
    handle_resize_screen as handle_kotlin_image_resize_screen,
    handle_resize_sync as handle_kotlin_image_resize_sync,
    handle_result as handle_kotlin_image_result, handle_screen_entry as handle_kotlin_image_screen,
    parse_image_target, render_kotlin_image_screen, ImageConversionResult, ImageTarget,
};
use crate::features::misc_screens::{
    render_about_screen, render_barometer_screen, render_compass_screen, render_loading_screen,
    render_magnetometer_screen, render_progress_demo_screen, render_shader_screen,
};
use crate::features::math_tool::{handle_math_action, render_math_tool_screen};
use crate::features::pdf::{
    handle_pdf_sign, perform_pdf_operation, perform_pdf_set_title, render_pdf_preview_screen,
    render_pdf_screen, PdfOperation, PdfSetTitleResult,
};
use crate::features::pixel_art::{
    process_pixel_art, render_pixel_art_screen, reset_pixel_art, save_fd_to_temp as save_pixel_fd,
};
use crate::features::presets::{
    apply_preset_to_state, delete_preset, load_presets, preset_payload_for_tool,
    render_preset_manager, render_save_preset_dialog, save_preset, tool_id_for_screen,
};
use crate::features::qr::{handle_qr_action, render_qr_screen};
use crate::features::qr_transfer::{
    advance_frame as qr_slideshow_advance, decode_qr_frame_luma, handle_receive_scan,
    load_slideshow_from_fd, load_slideshow_from_path, render_qr_receive_screen,
    render_qr_slideshow_screen, save_received_file,
};
use crate::features::regex_tester::{handle_regex_action, render_regex_tester_screen};
use crate::features::sensor_utils::{low_pass_angle, low_pass_scalar};
use crate::features::sensor_logger::{
    apply_status_from_bindings, parse_bindings as parse_sensor_bindings,
    render_sensor_logger_screen,
};
use crate::features::text_viewer::{apply_text_view_result, load_text_for_worker, TextViewLoadResult, TextViewSource};
use crate::features::text_viewer::guess_language_from_path;
use crate::features::text_tools::{handle_text_action, render_text_tools_screen, TextAction};
use crate::features::text_viewer::render_text_viewer_screen;
use crate::features::uuid_gen::{handle_uuid_action, render_uuid_screen};
use crate::ui::render_multi_hash_screen;

use crate::state::{AppState, DitheringMode, DitheringPalette, MultiHashResults, Screen};
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::Read,
    os::unix::io::{FromRawFd, RawFd},
    ptr,
    sync::{mpsc, Mutex, MutexGuard, OnceLock},
    thread,
};

#[cfg(test)]
use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};

struct GlobalState {
    ui: Mutex<AppState>,
    worker: OnceLock<WorkerRuntime>,
    notifications: Mutex<Vec<WorkerResult>>,
}

impl GlobalState {
    const fn new() -> Self {
        Self {
            ui: Mutex::new(AppState::new()),
            worker: OnceLock::new(),
            notifications: Mutex::new(Vec::new()),
        }
    }

    fn ui_lock(&self) -> MutexGuard<'_, AppState> {
        self.ui.lock().expect("ui mutex poisoned")
    }

    #[cfg(test)]
    fn ui_try_lock(&self) -> Option<MutexGuard<'_, AppState>> {
        self.ui.lock().ok()
    }

    fn worker(&self) -> &WorkerRuntime {
        self.worker.get_or_init(WorkerRuntime::new)
    }

    fn push_worker_result(&self, result: WorkerResult) {
        if let Ok(mut guard) = self.notifications.lock() {
            guard.push(result);
        }
    }

    fn drain_worker_results(&self) -> Vec<WorkerResult> {
        self.notifications
            .lock()
            .map(|mut q| q.drain(..).collect())
            .unwrap_or_default()
    }
}

struct WorkerRuntime {
    #[cfg_attr(test, allow(dead_code))]
    sender: mpsc::Sender<WorkerJob>,
}

impl WorkerRuntime {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel::<WorkerJob>();
        thread::Builder::new()
            .name("kistaverk-worker".into())
            .spawn(move || {
                while let Ok(job) = rx.recv() {
                    let result = run_worker_job(job);
                    STATE.push_worker_result(result);
                }
            })
            .expect("failed to spawn worker thread");

        Self { sender: tx }
    }

    #[cfg(not(test))]
    fn enqueue(&self, job: WorkerJob) -> Result<(), String> {
        self.sender
            .send(job)
            .map_err(|e| format!("worker_send_failed:{e}"))
    }

    #[cfg(test)]
    fn enqueue(&self, job: WorkerJob) -> Result<(), String> {
        if TEST_FORCE_ASYNC_WORKER.load(Ordering::SeqCst) {
            self.sender
                .send(job)
                .map_err(|e| format!("worker_send_failed:{e}"))
        } else {
            let result = run_worker_job(job);
            STATE.push_worker_result(result);
            Ok(())
        }
    }
}

#[derive(Clone)]
enum HashSourceInput {
    Fd(i32),
    Path(String),
}

#[derive(Clone, Copy)]
enum CompressionOp {
    Compress,
    Decompress,
}

#[derive(Clone)]
struct PdfWorkerArgs {
    op: PdfOperation,
    primary_fd: i32,
    secondary_fd: Option<i32>,
    primary_uri: Option<String>,
    secondary_uri: Option<String>,
    selected_pages: Vec<u32>,
}

#[derive(Clone)]
struct HashVerifyJob {
    source: HashSourceInput,
    reference: String,
    algo: HashAlgo,
}

#[derive(Clone)]
struct HashVerifyResult {
    computed: String,
    reference: String,
    algo: HashAlgo,
}

#[derive(Clone)]
struct PdfWorkerResult {
    out_path: String,
    page_count: u32,
    title: Option<String>,
    selected_pages: Vec<u32>,
    source_uri: Option<String>,
}

#[derive(Clone)]
struct PdfSelectResult {
    page_count: u32,
    title: Option<String>,
    source_uri: Option<String>,
}

#[derive(Clone)]
struct ArchiveCompressResult {
    open: ArchiveOpenResult,
    status: String,
}

enum WorkerJob {
    Hash {
        source: HashSourceInput,
        algo: HashAlgo,
    },
    MultiHash {
        source: HashSourceInput,
        display_path: String,
    },
    HashVerify(HashVerifyJob),
    Compression {
        op: CompressionOp,
        path: String,
    },
    Dithering {
        source_path: String,
        mode: DitheringMode,
        palette: DitheringPalette,
        output_dir: Option<String>,
    },
    PixelArt {
        source_path: String,
        scale: u32,
    },
    PdfOperation(PdfWorkerArgs),
    ArchiveOpen {
        fd: i32,
        path: Option<String>,
    },
    ArchiveCompress {
        source_path: String,
    },
    ArchiveExtractAll {
        archive_path: String,
    },
    ArchiveExtractEntry {
        archive_path: String,
        index: u32,
    },
    FileInfo {
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
    },
    PdfSelect {
        fd: i32,
        uri: Option<String>,
    },
    TextViewerLoad {
        source: TextViewSource,
        offset: u64,
        force_text: bool,
        can_page: bool,
    },
    PdfSetTitle {
        fd: i32,
        uri: Option<String>,
        title: Option<String>,
    },
}

enum WorkerResult {
    Hash {
        value: Result<String, String>,
    },
    MultiHash {
        value: Result<MultiHashResults, String>,
    },
    HashVerify {
        value: Result<HashVerifyResult, String>,
    },
    Compression {
        value: Result<String, String>,
    },
    Dithering {
        value: Result<String, String>,
    },
    PixelArt {
        value: Result<String, String>,
    },
    PdfOperation {
        value: Result<PdfWorkerResult, String>,
    },
    ArchiveOpen {
        value: Result<ArchiveOpenResult, String>,
    },
    ArchiveCompress {
        value: Result<ArchiveCompressResult, String>,
    },
    ArchiveExtract {
        archive_path: String,
        value: Result<String, String>,
    },
    FileInfo {
        value: Result<features::file_info::FileInfoResult, String>,
    },
    PdfSelect {
        value: Result<PdfSelectResult, String>,
    },
    TextViewer {
        value: Result<TextViewLoadResult, String>,
    },
    PdfSetTitle {
        value: Result<PdfSetTitleResult, String>,
    },
}

const COMPASS_SMOOTH_ALPHA: f64 = 0.2;
const BAROMETER_SMOOTH_ALPHA: f64 = 0.2;
const MAGNETOMETER_SMOOTH_ALPHA: f64 = 0.2;

fn run_worker_job(job: WorkerJob) -> WorkerResult {
    match job {
        WorkerJob::Hash { source, algo } => {
            test_worker_delay();
            let value = match source {
                HashSourceInput::Fd(fd) => {
                    compute_hash(features::hashes::HashSource::RawFd(fd as RawFd), algo)
                }
                HashSourceInput::Path(p) => {
                    compute_hash(features::hashes::HashSource::Path(&p), algo)
                }
            };
            WorkerResult::Hash { value }
        }
        WorkerJob::MultiHash {
            source,
            display_path,
        } => {
            test_worker_delay();
            let value = match source {
                HashSourceInput::Fd(fd) => compute_all_hashes(
                    features::hashes::HashSource::RawFd(fd as RawFd),
                    display_path,
                ),
                HashSourceInput::Path(p) => {
                    compute_all_hashes(features::hashes::HashSource::Path(&p), display_path)
                }
            };
            WorkerResult::MultiHash { value }
        }
        WorkerJob::HashVerify(job) => {
            test_worker_delay();
            let value = match job.source {
                HashSourceInput::Fd(fd) => {
                    compute_hash(features::hashes::HashSource::RawFd(fd as RawFd), job.algo)
                }
                HashSourceInput::Path(p) => {
                    compute_hash(features::hashes::HashSource::Path(&p), job.algo)
                }
            }
            .map(|computed| HashVerifyResult {
                computed,
                reference: job.reference,
                algo: job.algo,
            });
            WorkerResult::HashVerify { value }
        }
        WorkerJob::Compression { op, path } => {
            test_worker_delay();
            let value = match op {
                CompressionOp::Compress => {
                    gzip_compress(&path).map(|out| format!("Compressed to {}", out.display()))
                }
                CompressionOp::Decompress => {
                    gzip_decompress(&path).map(|out| format!("Decompressed to {}", out.display()))
                }
            };
            WorkerResult::Compression { value }
        }
        WorkerJob::Dithering {
            source_path,
            mode,
            palette,
            output_dir,
        } => {
            test_worker_delay();
            let value = process_dithering(&source_path, mode, palette, output_dir.as_deref());
            WorkerResult::Dithering { value }
        }
        WorkerJob::PixelArt { source_path, scale } => {
            test_worker_delay();
            let value = process_pixel_art(&source_path, scale);
            WorkerResult::PixelArt { value }
        }
        WorkerJob::PdfOperation(args) => {
            test_worker_delay();
            let value = perform_pdf_operation(
                args.op,
                args.primary_fd,
                args.secondary_fd,
                args.primary_uri.as_deref(),
                args.secondary_uri.as_deref(),
                &args.selected_pages,
            )
            .map(|pdf_out| PdfWorkerResult {
                out_path: pdf_out.out_path,
                page_count: pdf_out.page_count,
                title: pdf_out.title,
                selected_pages: args.selected_pages.clone(),
                source_uri: args.primary_uri.clone(),
            });
            WorkerResult::PdfOperation { value }
        }
        WorkerJob::ArchiveOpen { fd, path } => {
            test_worker_delay();
            let value = archive::open_archive_from_fd(fd as RawFd, path.as_deref());
            WorkerResult::ArchiveOpen { value }
        }
        WorkerJob::ArchiveCompress { source_path } => {
            test_worker_delay();
            let value = archive::create_archive(&source_path).and_then(|out| {
                let open_res = archive::open_archive_from_path(
                    out.to_string_lossy().as_ref(),
                )?;
                Ok(ArchiveCompressResult {
                    status: format!("Archive created at {}", out.display()),
                    open: open_res,
                })
            });
            WorkerResult::ArchiveCompress { value }
        }
        WorkerJob::ArchiveExtractAll { archive_path } => {
            test_worker_delay();
            let value = {
                let dest = archive::archive_output_root(&archive_path);
                archive::extract_all(&archive_path, &dest).map(|count| {
                    format!("Extracted {count} entries to {}", dest.display())
                })
            };
            WorkerResult::ArchiveExtract {
                archive_path,
                value,
            }
        }
        WorkerJob::ArchiveExtractEntry {
            archive_path,
            index,
        } => {
            test_worker_delay();
            let value = {
                let dest = archive::archive_output_root(&archive_path);
                archive::extract_entry(&archive_path, &dest, index)
                    .map(|out| format!("Extracted to {}", out.display()))
            };
            WorkerResult::ArchiveExtract {
                archive_path,
                value,
            }
        }
        WorkerJob::FileInfo { path, fd, error } => {
            test_worker_delay();
            let value = if let Some(err) = error {
                Err(err)
            } else if let Some(fd) = fd {
                Ok(file_info_from_fd(fd as RawFd))
            } else if let Some(p) = path {
                Ok(file_info_from_path(&p))
            } else {
                Err("missing_path".into())
            };
            WorkerResult::FileInfo { value }
        }
        WorkerJob::PdfSelect { fd, uri } => {
            test_worker_delay();
            let value = match features::pdf::load_pdf_metadata(fd as RawFd) {
                Ok((count, title)) => Ok(PdfSelectResult {
                    page_count: count,
                    title,
                    source_uri: uri,
                }),
                Err(e) => Err(e),
            };
            WorkerResult::PdfSelect { value }
        }
        WorkerJob::TextViewerLoad {
            source,
            offset,
            force_text,
            can_page,
        } => {
            test_worker_delay();
            let value = load_text_for_worker(source, offset, force_text, can_page);
            WorkerResult::TextViewer { value }
        }
        WorkerJob::PdfSetTitle { fd, uri, title } => {
            test_worker_delay();
            let value = perform_pdf_set_title(fd as RawFd, uri.as_deref(), title.as_deref());
            WorkerResult::PdfSetTitle { value }
        }
    }
}

static STATE: GlobalState = GlobalState::new();
// TODO: reduce lock hold time or move to a channel/queue; consider parking_lot with timeouts to avoid long UI pauses.

#[cfg(test)]
static TEST_FORCE_ASYNC_WORKER: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
static TEST_WORKER_DELAY_MS: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
fn test_worker_delay() {
    let delay = TEST_WORKER_DELAY_MS.load(Ordering::SeqCst);
    if delay > 0 {
        thread::sleep(Duration::from_millis(delay));
    }
}

#[cfg(not(test))]
fn test_worker_delay() {}

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
    KotlinImagePick {
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
    },
    DitheringScreen,
    DitheringPickImage {
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
    },
    DitheringSetMode {
        mode: DitheringMode,
    },
    DitheringSetPalette {
        palette: DitheringPalette,
    },
    DitheringApply {
        loading_only: bool,
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
    HashQrFromLast,
    HashPasteReference {
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
    QrSlideshowScreen,
    QrSlideshowPick {
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
    },
    QrSlideshowPlay,
    QrSlideshowNext,
    QrSlideshowPrev,
    QrSlideshowTick,
    QrSlideshowSetSpeed {
        interval_ms: u64,
    },
    QrReceiveScreen,
    QrReceiveScan {
        data: Option<String>,
    },
    QrReceiveSave,
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
    PdfReorder {
        fd: Option<i32>,
        uri: Option<String>,
        order: Vec<u32>,
    },
    PdfMerge {
        primary_fd: Option<i32>,
        primary_uri: Option<String>,
        secondary_fd: Option<i32>,
        secondary_uri: Option<String>,
    },
    PdfPreviewScreen,
    PdfPageOpen {
        page: u32,
    },
    PdfPageClose,
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
        index: u32,
    },
    ArchiveExtractAll,
    ArchiveExtractEntry {
        index: u32,
    },
    CompressionScreen,
    GzipCompress {
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
    },
    GzipDecompress {
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
    },
    SystemInfoScreen,
    SystemInfoUpdate {
        bindings: HashMap<String, String>,
    },
    ArchiveCompress {
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
    },
    MultiHashScreen,
    HashAll {
        path: Option<String>,
        fd: Option<i32>,
        loading_only: bool,
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
    PresetsList {
        tool_id: Option<String>,
    },
    PresetSaveDialog {
        tool_id: Option<String>,
    },
    PresetSave {
        name: Option<String>,
    },
    PresetLoad {
        id: String,
    },
    PresetDelete {
        id: String,
    },
    PixelArtScreen,
    PixelArtPick {
        path: Option<String>,
        fd: Option<i32>,
        error: Option<String>,
    },
    PixelArtSetScale {
        scale: u32,
    },
    PixelArtApply {
        loading_only: bool,
    },
    RegexTesterScreen,
    RegexTest {
        bindings: HashMap<String, String>,
    },
    MathToolScreen,
    MathCalculate {
        bindings: HashMap<String, String>,
    },
    MathClearHistory,
    UuidScreen,
    UuidGenerate,
    RandomStringGenerate {
        bindings: HashMap<String, String>,
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
        "pdf_reorder" => Ok(Action::PdfReorder {
            fd,
            uri: path,
            order: parse_pdf_order(&bindings),
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
        "pdf_preview_screen" => Ok(Action::PdfPreviewScreen),
        "pdf_page_open" => Ok(Action::PdfPageOpen {
            page: parse_u32_binding(&bindings, "page").unwrap_or(1),
        }),
        "pdf_page_close" => Ok(Action::PdfPageClose),
        "pixel_art_screen" => Ok(Action::PixelArtScreen),
        "pixel_art_pick" => Ok(Action::PixelArtPick { path, fd, error }),
        "pixel_art_set_scale" => Ok(Action::PixelArtSetScale {
            scale: parse_u32_binding(&bindings, "scale").unwrap_or(4),
        }),
        "pixel_art_apply" => Ok(Action::PixelArtApply { loading_only }),
        "regex_tester_screen" => Ok(Action::RegexTesterScreen),
        "regex_test" => Ok(Action::RegexTest { bindings }),
        "math_tool_screen" => Ok(Action::MathToolScreen),
        "math_calculate" => Ok(Action::MathCalculate { bindings }),
        "math_clear_history" => Ok(Action::MathClearHistory),
        "uuid_screen" => Ok(Action::UuidScreen),
        "uuid_generate" => Ok(Action::UuidGenerate),
        "random_string_generate" => Ok(Action::RandomStringGenerate { bindings }),
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
        "kotlin_image_screen_jpg" => Ok(Action::KotlinImageScreen(ImageTarget::Jpeg)),
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
        "kotlin_image_pick" => Ok(Action::KotlinImagePick { path, fd, error }),
        "dithering_screen" => Ok(Action::DitheringScreen),
        "dithering_pick_image" => Ok(Action::DitheringPickImage { path, fd, error }),
        "dithering_mode_fs" => Ok(Action::DitheringSetMode {
            mode: DitheringMode::FloydSteinberg,
        }),
        "dithering_mode_sierra" => Ok(Action::DitheringSetMode {
            mode: DitheringMode::Sierra,
        }),
        "dithering_mode_atkinson" => Ok(Action::DitheringSetMode {
            mode: DitheringMode::Atkinson,
        }),
        "dithering_mode_bayer4" => Ok(Action::DitheringSetMode {
            mode: DitheringMode::Bayer4x4,
        }),
        "dithering_mode_bayer8" => Ok(Action::DitheringSetMode {
            mode: DitheringMode::Bayer8x8,
        }),
        "dithering_palette_mono" => Ok(Action::DitheringSetPalette {
            palette: DitheringPalette::Monochrome,
        }),
        "dithering_palette_cga" => Ok(Action::DitheringSetPalette {
            palette: DitheringPalette::Cga,
        }),
        "dithering_palette_gb" => Ok(Action::DitheringSetPalette {
            palette: DitheringPalette::GameBoy,
        }),
        "dithering_apply" => Ok(Action::DitheringApply { loading_only }),
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
        "hash_paste_reference" => Ok(Action::HashPasteReference {
            reference: bindings
                .get("clipboard")
                .cloned()
                .or_else(|| bindings.get("hash_reference").cloned()),
        }),
        "hash_qr_last" => Ok(Action::HashQrFromLast),
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
        "qr_slideshow_screen" => Ok(Action::QrSlideshowScreen),
        "qr_slideshow_pick" => Ok(Action::QrSlideshowPick { path, fd, error }),
        "qr_slideshow_play" => Ok(Action::QrSlideshowPlay),
        "qr_slideshow_next" => Ok(Action::QrSlideshowNext),
        "qr_slideshow_prev" => Ok(Action::QrSlideshowPrev),
        "qr_slideshow_tick" => Ok(Action::QrSlideshowTick),
        "qr_slideshow_set_speed" => Ok(Action::QrSlideshowSetSpeed {
            interval_ms: parse_u64_binding(&bindings, "interval_ms").unwrap_or(200),
        }),
        "qr_receive_screen" => Ok(Action::QrReceiveScreen),
        "qr_receive_scan" => Ok(Action::QrReceiveScan {
            data: bindings
                .get("qr_scan_input")
                .cloned()
                .or_else(|| bindings.get("clipboard").cloned()),
        }),
        "qr_receive_save" => Ok(Action::QrReceiveSave),
        "archive_tools_screen" => Ok(Action::ArchiveToolsScreen),
        "archive_open" => Ok(Action::ArchiveOpen { fd, path, error }),
        "archive_compress" => Ok(Action::ArchiveCompress { path, fd, error }),
        "gzip_screen" => Ok(Action::CompressionScreen),
        "gzip_compress" => Ok(Action::GzipCompress { path, fd, error }),
        "gzip_decompress" => Ok(Action::GzipDecompress { path, fd, error }),
        "system_info_screen" => Ok(Action::SystemInfoScreen),
        "system_info_update" => Ok(Action::SystemInfoUpdate { bindings }),
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
        "presets_list" => Ok(Action::PresetsList {
            tool_id: bindings.get("tool_id").cloned(),
        }),
        "preset_save_dialog" => Ok(Action::PresetSaveDialog {
            tool_id: bindings.get("tool_id").cloned(),
        }),
        "preset_save" => Ok(Action::PresetSave {
            name: bindings.get("preset_name").cloned(),
        }),
        "preset_load" => bindings
            .get("id")
            .cloned()
            .ok_or_else(|| "missing_preset_id".to_string())
            .map(|id| Action::PresetLoad { id }),
        "preset_delete" => bindings
            .get("id")
            .cloned()
            .ok_or_else(|| "missing_preset_id".to_string())
            .map(|id| Action::PresetDelete { id }),
        other => {
            if let Some(idx) = other.strip_prefix("archive_open_text:") {
                let index = idx
                    .parse::<u32>()
                    .map_err(|_| format!("invalid_archive_index:{idx}"))?;
                Ok(Action::ArchiveOpenText { index })
            } else if other == "archive_extract_all" {
                Ok(Action::ArchiveExtractAll)
            } else if let Some(idx) = other.strip_prefix("archive_extract_entry:") {
                let index = idx
                    .parse::<u32>()
                    .map_err(|_| format!("invalid_archive_index:{idx}"))?;
                Ok(Action::ArchiveExtractEntry { index })
            } else if other == "multi_hash_screen" {
                Ok(Action::MultiHashScreen)
            } else if other == "hash_all" {
                Ok(Action::HashAll {
                    path,
                    fd,
                    loading_only,
                })
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

fn parse_pdf_order(bindings: &HashMap<String, String>) -> Vec<u32> {
    bindings
        .get("pdf_reorder_pages")
        .cloned()
        .unwrap_or_default()
        .split(',')
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

// JNI function to decode QR code from camera frame
#[no_mangle]
pub extern "system" fn Java_aeska_kistaverk_MainActivity_processQrCameraFrame(
    env: JNIEnv,
    _class: JClass,
    luma_array: jni::objects::JByteArray,
    width: jni::sys::jint,
    height: jni::sys::jint,
    row_stride: jni::sys::jint,
    rotation_deg: jni::sys::jint,
) -> jstring {
    let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let luma_data = env
            .convert_byte_array(&luma_array)
            .map_err(|e| format!("jni_luma_array_err:{e}"))?;
        let width_u = width as u32;
        let height_u = height as u32;
        let row_stride_u = row_stride as u32;
        let rotation_u = rotation_deg as u16;

        match decode_qr_frame_luma(&luma_data, width_u, height_u, row_stride_u, rotation_u) {
            Ok(Some(decoded_text)) => env
                .new_string(decoded_text)
                .map(|s| s.into_raw())
                .map_err(|e| format!("jni_new_string_err:{e}")),
            Ok(None) => Ok(ptr::null_mut()), // No QR code found
            Err(e) => {
                // Log the error and return null, or potentially a special error string
                eprintln!("QR decoding error: {}", e);
                Ok(ptr::null_mut())
            }
        }
    }));

    match response {
        Ok(Ok(res)) => res,
        Ok(Err(_)) | Err(_) => ptr::null_mut(), // Return null on any error or panic
    }
}

fn handle_command(command: Command) -> Result<Value, String> {
    let mut lock_poisoned = false;
    let mut state = match STATE.ui.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            lock_poisoned = true;
            poisoned.into_inner()
        }
    };

    apply_worker_results(&mut state);
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
        a @ Action::ArchiveToolsScreen
        | a @ Action::ArchiveOpen { .. }
        | a @ Action::ArchiveCompress { .. }
        | a @ Action::ArchiveOpenText { .. }
        | a @ Action::ArchiveExtractAll
        | a @ Action::ArchiveExtractEntry { .. } => {
            if let Some(ui) = handle_archive_actions(&mut state, a) {
                return Ok(ui);
            }
        }
        a @ Action::CompressionScreen
        | a @ Action::GzipCompress { .. }
        | a @ Action::GzipDecompress { .. } => {
            handle_compression_actions(&mut state, a);
        }
        Action::SystemInfoScreen => {
            state.push_screen(Screen::SystemInfo);
            state.system_info.error = None;
        }
        Action::SystemInfoUpdate { bindings } => {
            state.push_screen(Screen::SystemInfo);
            match features::system_info::apply_system_info_bindings(&mut state, &bindings) {
                Ok(_) => {} // No-op
                Err(e) => state.system_info.error = Some(e),
            }
        }
        Action::MultiHashScreen => {
            if let Some(ui) = handle_multi_hash_actions(&mut state, Action::MultiHashScreen) {
                return Ok(ui);
            }
        }
        Action::HashAll {
            path,
            fd,
            loading_only,
        } => {
            return handle_multi_hash_job(state, path, fd, loading_only);
        }
        Action::PresetsList { tool_id } => {
            if matches!(state.current_screen(), Screen::PresetManager) {
                state.replace_current(Screen::PresetManager);
            } else {
                state.push_screen(Screen::PresetManager);
            }
            state.preset_state.error = None;
            state.preset_state.last_message = None;
            state.preset_state.is_saving = false;
            if let Some(tool) = tool_id
                .or_else(|| tool_id_for_screen(state.current_screen()).map(|t| t.to_string()))
            {
                state.preset_state.current_tool_id = Some(tool);
            }
            match load_presets() {
                Ok(list) => state.preset_state.presets = list,
                Err(e) => {
                    state.preset_state.error = Some(e);
                    state.preset_state.presets.clear();
                }
            }
        }
        Action::PresetSaveDialog { tool_id } => {
            state.preset_state.error = None;
            state.preset_state.last_message = None;
            state.preset_state.is_saving = false;
            if let Some(tool) = tool_id
                .or_else(|| tool_id_for_screen(state.current_screen()).map(|t| t.to_string()))
            {
                state.preset_state.current_tool_id = Some(tool);
            }
            state.preset_state.name_input.clear();
            state.push_screen(Screen::PresetSave);
        }
        Action::PresetSave { name } => {
            state.preset_state.is_saving = true;
            let tool_id = state
                .preset_state
                .current_tool_id
                .clone()
                .or_else(|| tool_id_for_screen(state.current_screen()).map(|t| t.to_string()));
            let Some(tool_id) = tool_id else {
                state.preset_state.error = Some("preset_missing_tool".into());
                state.preset_state.is_saving = false;
                state.replace_current(Screen::PresetSave);
                return Ok(render_ui(&state));
            };
            state.preset_state.current_tool_id = Some(tool_id.clone());

            let provided = name
                .or_else(|| {
                    if state.preset_state.name_input.is_empty() {
                        None
                    } else {
                        Some(state.preset_state.name_input.clone())
                    }
                })
                .unwrap_or_default();
            let trimmed = provided.trim();
            if trimmed.is_empty() {
                state.preset_state.error = Some("preset_name_empty".into());
                state.preset_state.is_saving = false;
                state.replace_current(Screen::PresetSave);
                return Ok(render_ui(&state));
            }
            state.preset_state.name_input = trimmed.to_string();

            let payload = match preset_payload_for_tool(&state, &tool_id) {
                Ok(p) => p,
                Err(e) => {
                    state.preset_state.error = Some(e);
                    state.preset_state.is_saving = false;
                    state.replace_current(Screen::PresetSave);
                    return Ok(render_ui(&state));
                }
            };

            match save_preset(&tool_id, trimmed, payload) {
                Ok(saved) => {
                    state.preset_state.is_saving = false;
                    state.preset_state.error = None;
                    state.preset_state.last_message =
                        Some(format!("Saved preset \"{}\"", saved.name));
                    if !state.preset_state.presets.iter().any(|p| p.id == saved.id) {
                        state.preset_state.presets.insert(0, saved);
                        state
                            .preset_state
                            .presets
                            .sort_by(|a, b| b.created_at.cmp(&a.created_at));
                    }
                    state.replace_current(Screen::PresetManager);
                }
                Err(e) => {
                    state.preset_state.error = Some(e);
                    state.preset_state.is_saving = false;
                    state.replace_current(Screen::PresetSave);
                }
            }
        }
        Action::PresetLoad { id } => {
            let preset = state
                .preset_state
                .presets
                .iter()
                .find(|p| p.id == id)
                .cloned();
            let preset = if let Some(p) = preset {
                Some(p)
            } else {
                match load_presets() {
                    Ok(list) => {
                        state.preset_state.presets = list;
                        state
                            .preset_state
                            .presets
                            .iter()
                            .find(|p| p.id == id)
                            .cloned()
                    }
                    Err(e) => {
                        state.preset_state.error = Some(e);
                        None
                    }
                }
            };

            if let Some(preset) = preset {
                state.preset_state.current_tool_id = Some(preset.tool_id.clone());
                match apply_preset_to_state(&mut state, &preset) {
                    Ok(_) => {
                        state.preset_state.error = None;
                        state.preset_state.last_message =
                            Some(format!("Applied \"{}\"", preset.name));
                    }
                    Err(e) => {
                        state.preset_state.error = Some(e);
                    }
                }
            } else if state.preset_state.error.is_none() {
                state.preset_state.error = Some("preset_not_found".into());
            }

            if matches!(state.current_screen(), Screen::PresetManager) {
                state.replace_current(Screen::PresetManager);
            }
        }
        Action::PresetDelete { id } => {
            if let Err(e) = delete_preset(&id) {
                state.preset_state.error = Some(e);
            } else {
                state.preset_state.presets.retain(|p| p.id != id);
                state.preset_state.last_message = Some("Preset deleted".into());
            }
            if matches!(state.current_screen(), Screen::PresetManager) {
                state.replace_current(Screen::PresetManager);
            }
        }
        a @ Action::PixelArtScreen
        | a @ Action::PixelArtPick { .. }
        | a @ Action::PixelArtSetScale { .. }
        | a @ Action::PixelArtApply { .. }
        | a @ Action::KotlinImageScreen(_)
        | a @ Action::KotlinImageResizeScreen
        | a @ Action::KotlinImageResizeSync { .. }
        | a @ Action::KotlinImageResult { .. }
        | a @ Action::KotlinImageOutputDir { .. }
        | a @ Action::KotlinImagePick { .. }
        | a @ Action::DitheringScreen
        | a @ Action::DitheringPickImage { .. }
        | a @ Action::DitheringSetMode { .. }
        | a @ Action::DitheringSetPalette { .. }
        | a @ Action::DitheringApply { .. } => {
            if let Some(ui) = handle_media_actions(&mut state, a) {
                return Ok(ui);
            }
        }
        Action::RegexTesterScreen => {
            state.push_screen(Screen::RegexTester);
            state.regex_tester.error = None;
            state.regex_tester.match_result = None;
        }
        Action::RegexTest { bindings } => {
            state.push_screen(Screen::RegexTester);
            handle_regex_action(&mut state, &bindings);
            if matches!(state.current_screen(), Screen::RegexTester) {
                state.replace_current(Screen::RegexTester);
            }
        }
        Action::MathToolScreen => {
            state.push_screen(Screen::MathTool);
            state.math_tool.error = None;
        }
        Action::MathCalculate { bindings } => {
            state.push_screen(Screen::MathTool);
            handle_math_action(&mut state, "math_calculate", &bindings);
            if matches!(state.current_screen(), Screen::MathTool) {
                state.replace_current(Screen::MathTool);
            }
        }
        Action::MathClearHistory => {
            state.push_screen(Screen::MathTool);
            handle_math_action(&mut state, "math_clear_history", &HashMap::new());
            if matches!(state.current_screen(), Screen::MathTool) {
                state.replace_current(Screen::MathTool);
            }
        }
        Action::UuidScreen => {
            state.push_screen(Screen::UuidGenerator);
        }
        Action::UuidGenerate => {
            state.push_screen(Screen::UuidGenerator);
            handle_uuid_action(&mut state, "uuid_generate", &HashMap::new());
            if matches!(state.current_screen(), Screen::UuidGenerator) {
                state.replace_current(Screen::UuidGenerator);
            }
        }
        Action::RandomStringGenerate { bindings } => {
            state.push_screen(Screen::UuidGenerator);
            handle_uuid_action(&mut state, "random_string_generate", &bindings);
            if matches!(state.current_screen(), Screen::UuidGenerator) {
                state.replace_current(Screen::UuidGenerator);
            }
        }
        a @ Action::QrSlideshowScreen
        | a @ Action::QrSlideshowPick { .. }
        | a @ Action::QrSlideshowPlay
        | a @ Action::QrSlideshowNext
        | a @ Action::QrSlideshowPrev
        | a @ Action::QrSlideshowTick
        | a @ Action::QrSlideshowSetSpeed { .. }
        | a @ Action::QrReceiveScreen
        | a @ Action::QrReceiveScan { .. }
        | a @ Action::QrReceiveSave
        | a @ Action::QrGenerate { .. } => {
            handle_qr_actions(&mut state, a);
        }
        a @ Action::PdfToolsScreen
        | a @ Action::PdfSelect { .. }
        | a @ Action::PdfExtract { .. }
        | a @ Action::PdfDelete { .. }
        | a @ Action::PdfReorder { .. }
        | a @ Action::PdfMerge { .. }
        | a @ Action::PdfSetTitle { .. }
        | a @ Action::PdfPreviewScreen
        | a @ Action::PdfPageOpen { .. }
        | a @ Action::PdfPageClose
        | a @ Action::PdfSign { .. }
        | a @ Action::PdfSignGrid { .. } => {
            handle_pdf_actions(&mut state, a);
        }
        a @ Action::HashVerifyScreen
        | a @ Action::HashVerify { .. }
        | a @ Action::HashVerifyPaste { .. }
        | a @ Action::HashPasteReference { .. }
        | a @ Action::HashQrFromLast => {
            if let Some(ui) = handle_hash_actions(&mut state, a) {
                return Ok(ui);
            }
        }
        a @ Action::PdfSignatureStore { .. } | a @ Action::PdfSignatureClear => {
            handle_pdf_actions(&mut state, a);
        }
        Action::About => {
            state.push_screen(Screen::About);
        }
        a @ Action::TextViewerScreen
        | a @ Action::TextViewerOpen { .. }
        | a @ Action::TextViewerToggleTheme
        | a @ Action::TextViewerToggleLineNumbers
        | a @ Action::TextViewerLoadAnyway
        | a @ Action::TextViewerLoadMore
        | a @ Action::TextViewerLoadPrev
        | a @ Action::TextViewerJump { .. }
        | a @ Action::TextViewerFind { .. } => {
            handle_text_viewer_actions(&mut state, a);
        }
        a @ Action::SensorLoggerScreen
        | a @ Action::SensorLoggerStart { .. }
        | a @ Action::SensorLoggerStop
        | a @ Action::SensorLoggerShare
        | a @ Action::SensorLoggerStatus { .. }
        | a @ Action::CompassDemo
        | a @ Action::CompassSet { .. }
        | a @ Action::BarometerScreen
        | a @ Action::BarometerSet { .. }
        | a @ Action::MagnetometerScreen
        | a @ Action::MagnetometerSet { .. } => {
            handle_sensor_actions(&mut state, a);
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
            return handle_hash_job(state, algo, path, fd, error, loading_only);
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
            state.replace_current(Screen::Loading);
            state.loading_message = Some("Reading file info...".into());
            state.loading_with_spinner = true;
            let job = WorkerJob::FileInfo { path, fd, error };
            if let Err(e) = STATE.worker().enqueue(job) {
                state.last_error = Some(e);
                state.loading_message = None;
                state.loading_with_spinner = false;
                state.replace_current(Screen::FileInfo);
            }
            #[cfg(test)]
            {
                apply_worker_results(&mut state);
            }
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

fn handle_qr_actions(state: &mut AppState, action: Action) {
    match action {
        Action::QrGenerate { input } => {
            state.push_screen(Screen::Qr);
            let text = input.unwrap_or_default();
            if let Err(e) = handle_qr_action(state, &text) {
                state.last_error = Some(e);
            }
        }
        Action::QrSlideshowScreen => {
            state.push_screen(Screen::QrSlideshow);
            state.qr_slideshow.error = None;
        }
        Action::QrSlideshowPick { path, fd, error } => {
            state.push_screen(Screen::QrSlideshow);
            state.qr_slideshow.error = error.clone();
            let mut fd_handle = FdHandle::new(fd);
            if error.is_none() {
                if let Some(raw_fd) = fd_handle.take() {
                    if let Err(e) = load_slideshow_from_fd(state, raw_fd as RawFd, path.as_deref())
                    {
                        state.qr_slideshow.error = Some(e);
                    }
                } else if let Some(p) = path.as_deref() {
                    if let Err(e) = load_slideshow_from_path(state, p) {
                        state.qr_slideshow.error = Some(e);
                    }
                } else {
                    state.qr_slideshow.error = Some("missing_source".into());
                }
            }
        }
        Action::QrSlideshowPlay => {
            state.qr_slideshow.is_playing = !state.qr_slideshow.is_playing;
            if matches!(state.current_screen(), Screen::QrSlideshow) {
                state.replace_current(Screen::QrSlideshow);
            }
        }
        Action::QrSlideshowNext => {
            state.qr_slideshow.is_playing = false;
            qr_slideshow_advance(state, 1).unwrap_or(());
            if matches!(state.current_screen(), Screen::QrSlideshow) {
                state.replace_current(Screen::QrSlideshow);
            }
        }
        Action::QrSlideshowPrev => {
            state.qr_slideshow.is_playing = false;
            qr_slideshow_advance(state, -1).unwrap_or(());
            if matches!(state.current_screen(), Screen::QrSlideshow) {
                state.replace_current(Screen::QrSlideshow);
            }
        }
        Action::QrSlideshowTick => {
            if state.qr_slideshow.is_playing {
                qr_slideshow_advance(state, 1).unwrap_or(());
                if matches!(state.current_screen(), Screen::QrSlideshow) {
                    state.replace_current(Screen::QrSlideshow);
                }
            }
        }
        Action::QrSlideshowSetSpeed { interval_ms } => {
            state.qr_slideshow.interval_ms = interval_ms.max(50);
            if matches!(state.current_screen(), Screen::QrSlideshow) {
                state.replace_current(Screen::QrSlideshow);
            }
        }
        Action::QrReceiveScreen => {
            state.push_screen(Screen::QrReceive);
            state.qr_receive.reset();
        }
        Action::QrReceiveScan { data } => {
            if let Some(payload) = data {
                if !payload.trim().is_empty() {
                    if let Err(e) = handle_receive_scan(state, &payload) {
                        state.qr_receive.error = Some(e);
                    }
                }
            }
            if matches!(state.current_screen(), Screen::QrReceive) {
                state.replace_current(Screen::QrReceive);
            }
        }
        Action::QrReceiveSave => {
            match save_received_file(state) {
                Ok(_) => state.qr_receive.error = None,
                Err(e) => state.qr_receive.error = Some(e),
            }
            if matches!(state.current_screen(), Screen::QrReceive) {
                state.replace_current(Screen::QrReceive);
            }
        }
        _ => {}
    }
}

fn handle_archive_actions(state: &mut AppState, action: Action) -> Option<Value> {
    match action {
        Action::ArchiveToolsScreen => {
            state.push_screen(Screen::ArchiveTools);
            state.archive.reset();
            None
        }
        Action::ArchiveOpen { fd, path, error } => {
            state.push_screen(Screen::ArchiveTools);
            state.archive.error = error.clone();
            state.archive.last_output = None;
            state.archive.entries.clear();
            state.archive.truncated = false;
            state.archive.path = path.clone();
            let mut fd_handle = FdHandle::new(fd);
            if let Some(err) = error {
                state.archive.error = Some(err);
            } else if let Some(raw_fd) = fd_handle.take() {
                state.loading_with_spinner = true;
                state.loading_message = Some("Opening archive...".into());
                state.replace_current(Screen::Loading);
                let job = WorkerJob::ArchiveOpen { fd: raw_fd, path };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.archive.error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.archive.error = Some("missing_fd".into());
            }
            None
        }
        Action::ArchiveCompress { path, fd, error } => {
            state.push_screen(Screen::ArchiveTools);
            state.archive.error = None;
            state.archive.last_output = None;
            state.archive.entries.clear();
            state.archive.truncated = false;
            state.archive.path = None;
            if let Some(err) = error {
                state.archive.error = Some(err);
            } else if let Some(path) = path {
                if fd.is_some() {
                    state.archive.error = Some("archive_compress_requires_path".into());
                } else {
                    state.loading_with_spinner = true;
                    state.loading_message = Some("Compressing...".into());
                    state.replace_current(Screen::Loading);
                    let job = WorkerJob::ArchiveCompress { source_path: path };
                    if let Err(e) = STATE.worker().enqueue(job) {
                        state.archive.error = Some(e);
                    }
                    #[cfg(test)]
                    {
                        apply_worker_results(state);
                    }
                }
            } else if fd.is_some() {
                state.archive.error = Some("archive_compress_requires_path".into());
            } else {
                state.archive.error = Some("missing_path".into());
            }
            None
        }
        Action::ArchiveOpenText { index } => {
            state.push_screen(Screen::TextViewer);
            match features::archive::read_text_entry(state, index) {
                Ok((label, text)) => {
                    state.text_view_path = Some(label);
                    state.text_view_content = Some(text);
                    state.text_view_error = None;
                    if let Some(entry) = state.archive.entries.get(index as usize) {
                        state.text_view_language = guess_language_from_path(&entry.name);
                    } else {
                        state.text_view_language = None;
                    }
                }
                Err(e) => {
                    state.text_view_error = Some(e);
                    state.text_view_content = None;
                    state.text_view_language = None;
                    if let Some(entry) = state.archive.entries.get(index as usize) {
                        state.text_view_path = state
                            .archive
                            .path
                            .as_ref()
                            .map(|p| format!("{} ⟂ {}", entry.name, p))
                            .or_else(|| Some(entry.name.clone()));
                    }
                }
            }
            None
        }
        Action::ArchiveExtractAll => {
            state.replace_current(Screen::ArchiveTools);
            state.archive.last_output = None;
            if let Some(path) = state.archive.path.clone() {
                state.archive.error = None;
                state.loading_with_spinner = true;
                state.loading_message = Some("Extracting...".into());
                state.replace_current(Screen::Loading);
                let job = WorkerJob::ArchiveExtractAll {
                    archive_path: path,
                };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.archive.error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.archive.error = Some("archive_missing_path".into());
            }
            None
        }
        Action::ArchiveExtractEntry { index } => {
            state.replace_current(Screen::ArchiveTools);
            state.archive.last_output = None;
            if let Some(path) = state.archive.path.clone() {
                state.archive.error = None;
                state.loading_with_spinner = true;
                state.loading_message = Some("Extracting...".into());
                state.replace_current(Screen::Loading);
                let job = WorkerJob::ArchiveExtractEntry {
                    archive_path: path,
                    index,
                };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.archive.error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.archive.error = Some("archive_missing_path".into());
            }
            None
        }
        _ => None,
    }
}

fn handle_compression_actions(state: &mut AppState, action: Action) {
    match action {
        Action::CompressionScreen => {
            state.push_screen(Screen::Compression);
            state.compression_error = None;
            state.compression_status = None;
        }
        Action::GzipCompress { path, fd, error } => {
            state.push_screen(Screen::Compression);
            state.compression_error = None;
            state.compression_status = None;
            if let Some(err) = error {
                state.compression_error = Some(err);
            } else if let Some(p) = path {
                state.loading_with_spinner = true;
                state.loading_message = Some("Compressing...".into());
                if fd.is_some() {
                    state.compression_error = Some("gzip_requires_path".into());
                } else {
                    let job = WorkerJob::Compression {
                        op: CompressionOp::Compress,
                        path: p,
                    };
                    if let Err(e) = STATE.worker().enqueue(job) {
                        state.compression_error = Some(e);
                    }
                    #[cfg(test)]
                    {
                        apply_worker_results(state);
                    }
                }
            } else if fd.is_some() {
                state.compression_error = Some("gzip_requires_path".into());
            } else {
                state.compression_error = Some("missing_path".into());
            }
        }
        Action::GzipDecompress { path, fd, error } => {
            state.push_screen(Screen::Compression);
            state.compression_error = None;
            state.compression_status = None;
            if let Some(err) = error {
                state.compression_error = Some(err);
            } else if let Some(p) = path {
                state.loading_with_spinner = true;
                state.loading_message = Some("Decompressing...".into());
                if fd.is_some() {
                    state.compression_error = Some("gzip_requires_path".into());
                } else {
                    let job = WorkerJob::Compression {
                        op: CompressionOp::Decompress,
                        path: p,
                    };
                    if let Err(e) = STATE.worker().enqueue(job) {
                        state.compression_error = Some(e);
                    }
                    #[cfg(test)]
                    {
                        apply_worker_results(state);
                    }
                }
            } else if fd.is_some() {
                state.compression_error = Some("gzip_requires_path".into());
            } else {
                state.compression_error = Some("missing_path".into());
            }
        }
        _ => {}
    }
}

fn handle_multi_hash_actions(state: &mut AppState, action: Action) -> Option<Value> {
    match action {
        Action::MultiHashScreen => {
            state.push_screen(Screen::MultiHash);
            state.multi_hash_results = None;
            state.multi_hash_error = None;
            None
        }
        _ => None,
    }
}

fn handle_hash_actions(state: &mut AppState, action: Action) -> Option<Value> {
    match action {
        Action::HashVerifyScreen => {
            state.push_screen(Screen::HashVerify);
            state.hash_reference = None;
            state.hash_match = None;
            state.last_hash = None;
            state.last_hash_algo = Some("SHA-256".into());
            None
        }
        Action::HashVerify {
            path,
            fd,
            reference,
        } => {
            let mut fd_handle = FdHandle::new(fd);
            state.push_screen(Screen::HashVerify);
            if let Some(err) = reference
                .as_ref()
                .filter(|s| s.trim().is_empty())
                .map(|_| "reference_empty".to_string())
            {
                state.last_error = Some(err);
                state.hash_match = None;
            } else {
                let algo = HashAlgo::Sha256;
                if let Some(err) = reference
                    .clone()
                    .is_none()
                    .then(|| "missing_reference".to_string())
                {
                    state.last_error = Some(err);
                } else {
                    let source = hash_job_source(fd_handle.take(), path.as_deref());
                    if let Some(src) = source {
                        let reference = reference.unwrap();
                        let job = WorkerJob::HashVerify(HashVerifyJob {
                            source: src,
                            reference: reference.clone(),
                            algo,
                        });
                        state.hash_reference = Some(reference);
                        state.hash_match = None;
                        state.last_hash = None;
                        state.last_error = None;
                        state.loading_with_spinner = true;
                        state.loading_message = Some(hash_loading_message(algo).into());
                        state.replace_current(Screen::Loading);
                        if let Err(e) = STATE.worker().enqueue(job) {
                            state.last_error = Some(e);
                        }
                        #[cfg(test)]
                        {
                            apply_worker_results(state);
                        }
                    } else {
                        state.last_error = Some("missing_path".into());
                        state.hash_match = None;
                        state.last_hash = None;
                    }
                }
            }
            None
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
            None
        }
        Action::HashPasteReference { reference } => {
            state.push_screen(Screen::Home);
            if let Some(text) = reference {
                state.hash_reference = Some(text.clone());
                state.hash_match = None;
                state.last_error = None;
                if let Some(hash) = state.last_hash.clone() {
                    let cleaned_ref = text.trim().to_ascii_lowercase();
                    let cleaned_hash = hash.trim().to_ascii_lowercase();
                    state.hash_match = Some(cleaned_ref == cleaned_hash);
                }
            } else {
                state.last_error = Some("clipboard_empty".into());
            }
            None
        }
        Action::HashQrFromLast => {
            if let Some(hash) = state.last_hash.clone() {
                state.push_screen(Screen::Qr);
                if let Err(e) = handle_qr_action(state, &hash) {
                    state.last_error = Some(e);
                }
            } else {
                state.last_error = Some("no_hash_available".into());
            }
            None
        }
        _ => None,
    }
}

fn handle_hash_job(
    mut state: MutexGuard<'_, AppState>,
    algo: HashAlgo,
    path: Option<String>,
    fd: Option<i32>,
    error: Option<String>,
    loading_only: bool,
) -> Result<Value, String> {
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
        state.loading_message = None;
        state.loading_with_spinner = true;
        return Ok(render_ui(&state));
    }

    let source = hash_job_source(fd_handle.take(), path.as_deref());
    if source.is_none() {
        state.last_error = Some("missing_path".into());
        state.last_hash = None;
        state.loading_message = None;
        state.loading_with_spinner = true;
        return Ok(render_ui(&state));
    }

    drop(fd_handle);
    let job = WorkerJob::Hash {
        source: source.unwrap(),
        algo,
    };
    if let Err(e) = STATE.worker().enqueue(job) {
        state.last_error = Some(e);
        state.last_hash = None;
    }
    #[cfg(test)]
    {
        apply_worker_results(&mut state);
    }
    state.loading_message = None;
    state.loading_with_spinner = true;
    Ok(render_ui(&state))
}

fn handle_multi_hash_job(
    mut state: MutexGuard<'_, AppState>,
    path: Option<String>,
    fd: Option<i32>,
    loading_only: bool,
) -> Result<Value, String> {
    let mut fd_handle = FdHandle::new(fd);
    if loading_only {
        state.loading_with_spinner = false;
        state.replace_current(Screen::Loading);
        state.loading_message = Some("Computing all hashes...".into());
        state.multi_hash_results = None;
        state.multi_hash_error = None;
        return Ok(render_ui(&state));
    }
    let source = hash_job_source(fd_handle.take(), path.as_deref());
    state.reset_navigation();
    state.push_screen(Screen::MultiHash);
    state.multi_hash_results = None;
    state.multi_hash_error = None;

    match source {
        Some(src) => {
            let display = path.clone().unwrap_or_else(|| "Selected file".to_string());
            drop(fd_handle);
            let job = WorkerJob::MultiHash {
                source: src,
                display_path: display,
            };
            if let Err(e) = STATE.worker().enqueue(job) {
                state.multi_hash_error = Some(e);
                state.multi_hash_results = None;
            }
            #[cfg(test)]
            {
                apply_worker_results(&mut state);
            }
            state.loading_message = None;
            state.loading_with_spinner = true;
            return Ok(render_ui(&state));
        }
        None => {
            state.multi_hash_error = Some("missing_path".into());
            state.multi_hash_results = None;
            state.loading_message = None;
            state.loading_with_spinner = true;
            return Ok(render_ui(&state));
        }
    }
}

fn hash_job_source(fd: Option<i32>, path: Option<&str>) -> Option<HashSourceInput> {
    if let Some(fd) = fd {
        Some(HashSourceInput::Fd(fd))
    } else {
        path.map(|p| HashSourceInput::Path(p.to_string()))
    }
}

fn handle_pdf_actions(state: &mut AppState, action: Action) {
    match action {
        Action::PdfToolsScreen => {
            state.push_screen(Screen::PdfTools);
            state.pdf.last_error = None;
            state.pdf.last_output = None;
        }
        Action::PdfSelect { fd, uri, error } => {
            state.push_screen(Screen::PdfTools);
            state.pdf.last_error = error.clone();
            state.pdf.preview_page = None;
            state.pdf.page_count = None;
            state.pdf.selected_pages.clear();
            state.pdf.last_output = None;
            state.pdf.current_title = None;
            state.pdf.signature_target_page = None;
            state.pdf.signature_x_pct = None;
            state.pdf.signature_y_pct = None;
            if let Some(err) = error {
                state.pdf.last_error = Some(err);
            } else if let Some(raw_fd) = fd {
                state.loading_message = Some("Loading PDF...".into());
                state.loading_with_spinner = true;
                state.replace_current(Screen::Loading);
                let job = WorkerJob::PdfSelect {
                    fd: raw_fd,
                    uri: uri.clone(),
                };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.pdf.last_error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.pdf.last_error = Some("missing_fd".into());
            }
        }
        Action::PdfExtract { fd, uri, selection } => {
            state.push_screen(Screen::PdfTools);
            state.pdf.last_error = None;
            state.pdf.last_output = None;
            let mut fd_handle = FdHandle::new(fd);
            if selection.is_empty() {
                state.pdf.last_error = Some("no_pages_selected".into());
            } else if let Some(raw_fd) = fd_handle.take() {
                state.loading_with_spinner = true;
                state.loading_message = Some("Processing PDF...".into());
                let job = WorkerJob::PdfOperation(PdfWorkerArgs {
                    op: PdfOperation::Extract,
                    primary_fd: raw_fd,
                    secondary_fd: None,
                    primary_uri: uri.clone(),
                    secondary_uri: None,
                    selected_pages: selection.clone(),
                });
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.pdf.last_error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.pdf.last_error = Some("missing_fd".into());
            }
        }
        Action::PdfDelete { fd, uri, selection } => {
            state.push_screen(Screen::PdfTools);
            state.pdf.last_error = None;
            state.pdf.last_output = None;
            let mut fd_handle = FdHandle::new(fd);
            if selection.is_empty() {
                state.pdf.last_error = Some("no_pages_selected".into());
            } else if let Some(raw_fd) = fd_handle.take() {
                state.loading_with_spinner = true;
                state.loading_message = Some("Processing PDF...".into());
                let job = WorkerJob::PdfOperation(PdfWorkerArgs {
                    op: PdfOperation::Delete,
                    primary_fd: raw_fd,
                    secondary_fd: None,
                    primary_uri: uri.clone(),
                    secondary_uri: None,
                    selected_pages: selection.clone(),
                });
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.pdf.last_error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.pdf.last_error = Some("missing_fd".into());
            }
        }
        Action::PdfReorder { fd, uri, order } => {
            state.push_screen(Screen::PdfTools);
            state.pdf.last_error = None;
            state.pdf.last_output = None;
            let mut fd_handle = FdHandle::new(fd);
            if order.is_empty() {
                state.pdf.last_error = Some("no_pages_selected".into());
            } else if let Some(raw_fd) = fd_handle.take() {
                state.loading_with_spinner = true;
                state.loading_message = Some("Processing PDF...".into());
                let job = WorkerJob::PdfOperation(PdfWorkerArgs {
                    op: PdfOperation::Reorder,
                    primary_fd: raw_fd,
                    secondary_fd: None,
                    primary_uri: uri.clone(),
                    secondary_uri: None,
                    selected_pages: order.clone(),
                });
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.pdf.last_error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
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
            state.pdf.last_error = None;
            state.pdf.last_output = None;
            let mut primary = FdHandle::new(primary_fd);
            let mut secondary = FdHandle::new(secondary_fd);
            if let (Some(p_fd), Some(s_fd)) = (primary.take(), secondary.take()) {
                state.loading_with_spinner = true;
                state.loading_message = Some("Processing PDF...".into());
                let job = WorkerJob::PdfOperation(PdfWorkerArgs {
                    op: PdfOperation::Merge,
                    primary_fd: p_fd,
                    secondary_fd: Some(s_fd),
                    primary_uri: primary_uri.clone(),
                    secondary_uri: secondary_uri.clone(),
                    selected_pages: Vec::new(),
                });
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.pdf.last_error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.pdf.last_error = Some("missing_fd".into());
            }
        }
        Action::PdfSetTitle { fd, uri, title } => {
            state.push_screen(Screen::PdfTools);
            if let Some(raw_fd) = fd {
                state.loading_message = Some("Updating title...".into());
                state.loading_with_spinner = true;
                let job = WorkerJob::PdfSetTitle {
                    fd: raw_fd,
                    uri: uri.clone(),
                    title,
                };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.pdf.last_error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.pdf.last_error = Some("missing_fd".into());
            }
        }
        Action::PdfPreviewScreen => {
            if matches!(state.current_screen(), Screen::PdfPreview) {
                state.replace_current(Screen::PdfPreview);
            } else {
                state.push_screen(Screen::PdfPreview);
            }
            state.pdf.preview_page = None;
        }
        Action::PdfPageOpen { page } => {
            if matches!(state.current_screen(), Screen::PdfPreview) {
                state.replace_current(Screen::PdfPreview);
            } else {
                state.push_screen(Screen::PdfPreview);
            }
            let mut target_page = page.max(1);
            if let Some(count) = state.pdf.page_count {
                if target_page > count {
                    target_page = count;
                }
            }
            state.pdf.preview_page = Some(target_page);
        }
        Action::PdfPageClose => {
            if matches!(state.current_screen(), Screen::PdfPreview) {
                state.replace_current(Screen::PdfPreview);
            } else {
                state.push_screen(Screen::PdfPreview);
            }
            state.pdf.preview_page = None;
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
                    if let Err(e) = handle_pdf_sign(
                        state,
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
                        state.pdf.last_error = Some(e);
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
        _ => {}
    }
}

fn handle_text_viewer_actions(state: &mut AppState, action: Action) {
    match action {
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
            if error.is_some() {
                state.text_view_content = None;
                state.text_view_language = None;
                state.text_view_hex_preview = None;
            } else if let Some(raw_fd) = fd {
                state.loading_message = Some("Loading text...".into());
                state.loading_with_spinner = true;
                state.replace_current(Screen::Loading);
                let source = TextViewSource::Fd {
                    fd: raw_fd,
                    display_path: path.clone(),
                };
                let job = WorkerJob::TextViewerLoad {
                    source,
                    offset: 0,
                    force_text: false,
                    can_page: true,
                };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.text_view_error = Some(e);
                    state.replace_current(Screen::TextViewer);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else if let Some(p) = path.clone() {
                state.loading_message = Some("Loading text...".into());
                state.loading_with_spinner = true;
                state.replace_current(Screen::Loading);
                let source = TextViewSource::Path {
                    read_path: p.clone(),
                    display_path: Some(p),
                };
                let job = WorkerJob::TextViewerLoad {
                    source,
                    offset: 0,
                    force_text: false,
                    can_page: true,
                };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.text_view_error = Some(e);
                    state.replace_current(Screen::TextViewer);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
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
            state.text_view_hex_preview = None;
            if let Some(path) = state.text_view_path.clone() {
                let effective = state.text_view_cached_path.clone().unwrap_or(path.clone());
                state.loading_message = Some("Loading text...".into());
                state.loading_with_spinner = true;
                state.replace_current(Screen::Loading);
                let source = TextViewSource::Path {
                    read_path: effective,
                    display_path: Some(path),
                };
                let job = WorkerJob::TextViewerLoad {
                    source,
                    offset: 0,
                    force_text: true,
                    can_page: true,
                };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.text_view_error = Some(e);
                    state.replace_current(Screen::TextViewer);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.text_view_error = Some("nothing_to_reload".into());
                state.text_view_content = None;
                state.replace_current(Screen::TextViewer);
            }
        }
        Action::TextViewerLoadMore => {
            let path = match state.text_view_path.clone() {
                Some(p) => p,
                None => {
                    state.text_view_error = Some("missing_path".into());
                    state.replace_current(Screen::TextViewer);
                    return;
                }
            };
            let offset = state
                .text_view_window_offset
                .saturating_add(features::text_viewer::CHUNK_BYTES as u64);
            let effective = state.text_view_cached_path.clone().unwrap_or(path.clone());
            state.loading_message = Some("Loading text...".into());
            state.loading_with_spinner = true;
            state.replace_current(Screen::Loading);
            let source = TextViewSource::Path {
                read_path: effective,
                display_path: Some(path),
            };
            let job = WorkerJob::TextViewerLoad {
                source,
                offset,
                force_text: true,
                can_page: true,
            };
            if let Err(e) = STATE.worker().enqueue(job) {
                state.text_view_error = Some(e);
                state.replace_current(Screen::TextViewer);
            }
            #[cfg(test)]
            {
                apply_worker_results(state);
            }
        }
        Action::TextViewerLoadPrev => {
            let path = match state.text_view_path.clone() {
                Some(p) => p,
                None => {
                    state.text_view_error = Some("missing_path".into());
                    state.replace_current(Screen::TextViewer);
                    return;
                }
            };
            let offset = state
                .text_view_window_offset
                .saturating_sub(features::text_viewer::CHUNK_BYTES as u64);
            let effective = state.text_view_cached_path.clone().unwrap_or(path.clone());
            state.loading_message = Some("Loading text...".into());
            state.loading_with_spinner = true;
            state.replace_current(Screen::Loading);
            let source = TextViewSource::Path {
                read_path: effective,
                display_path: Some(path),
            };
            let job = WorkerJob::TextViewerLoad {
                source,
                offset,
                force_text: true,
                can_page: true,
            };
            if let Err(e) = STATE.worker().enqueue(job) {
                state.text_view_error = Some(e);
                state.replace_current(Screen::TextViewer);
            }
            #[cfg(test)]
            {
                apply_worker_results(state);
            }
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
                state.loading_message = Some("Loading text...".into());
                state.loading_with_spinner = true;
                state.replace_current(Screen::Loading);
                let source = TextViewSource::Path {
                    read_path: effective,
                    display_path: Some(path),
                };
                let job = WorkerJob::TextViewerLoad {
                    source,
                    offset: clamped,
                    force_text: true,
                    can_page: true,
                };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.text_view_error = Some(e);
                    state.replace_current(Screen::TextViewer);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.text_view_error = Some("missing_path".into());
                state.replace_current(Screen::TextViewer);
            }
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
        _ => {}
    }
}

fn handle_sensor_actions(state: &mut AppState, action: Action) {
    match action {
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
            if matches!(state.current_screen(), Screen::SensorLogger) {
                state.replace_current(Screen::SensorLogger);
            }
        }
        Action::SensorLoggerStatus { bindings } => {
            apply_status_from_bindings(state, &bindings);
            if matches!(state.current_screen(), Screen::SensorLogger) {
                state.replace_current(Screen::SensorLogger);
            }
        }
        Action::CompassDemo => {
            state.push_screen(Screen::Compass);
        }
        Action::CompassSet {
            angle_radians,
            error,
        } => {
            if let Some(err) = error {
                state.compass_error = Some(err);
            } else if let Some(filtered) =
                low_pass_angle(state.compass_filter_angle, angle_radians, COMPASS_SMOOTH_ALPHA)
            {
                state.compass_filter_angle = Some(filtered);
                state.compass_angle_radians = filtered;
                state.compass_error = None;
            } else {
                state.compass_error = Some("invalid_angle".into());
            }
            if matches!(state.current_screen(), Screen::Compass) {
                state.replace_current(Screen::Compass);
            }
        }
        Action::BarometerScreen => {
            state.push_screen(Screen::Barometer);
        }
        Action::BarometerSet { hpa, error } => {
            if let Some(err) = error {
                state.barometer_error = Some(err);
            } else if let Some(filtered) =
                low_pass_scalar(state.barometer_filter_value, hpa, BAROMETER_SMOOTH_ALPHA)
            {
                state.barometer_filter_value = Some(filtered);
                state.barometer_hpa = Some(filtered);
                state.barometer_error = None;
            } else {
                state.barometer_error = Some("invalid_pressure".into());
            }
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
            if let Some(err) = error {
                state.magnetometer_error = Some(err);
            } else if let Some(filtered) = low_pass_scalar(
                state.magnetometer_filter_value,
                magnitude_ut,
                MAGNETOMETER_SMOOTH_ALPHA,
            ) {
                state.magnetometer_filter_value = Some(filtered);
                state.magnetometer_ut = Some(filtered);
                state.magnetometer_error = None;
            } else {
                state.magnetometer_error = Some("invalid_magnetometer".into());
            }
            if matches!(state.current_screen(), Screen::Magnetometer) {
                state.replace_current(Screen::Magnetometer);
            }
        }
        _ => {}
    }
}

fn handle_media_actions(state: &mut AppState, action: Action) -> Option<Value> {
    match action {
        Action::PixelArtScreen => {
            state.push_screen(Screen::PixelArt);
            reset_pixel_art(&mut state.pixel_art);
            None
        }
        Action::PixelArtPick { path, fd, error } => {
            state.push_screen(Screen::PixelArt);
            state.pixel_art.error = error.clone();
            state.pixel_art.result_path = None;
            let mut fd_handle = FdHandle::new(fd);
            if error.is_none() {
                if let Some(raw_fd) = fd_handle.take() {
                    match save_pixel_fd(raw_fd as RawFd, path.as_deref()) {
                        Ok(saved) => {
                            state.pixel_art.source_path = Some(saved);
                            state.pixel_art.error = None;
                        }
                        Err(e) => state.pixel_art.error = Some(e),
                    }
                } else if let Some(p) = path {
                    state.pixel_art.source_path = Some(p);
                    state.pixel_art.error = None;
                } else {
                    state.pixel_art.error = Some("missing_source".into());
                }
            }
            None
        }
        Action::PixelArtSetScale { scale } => {
            state.pixel_art.scale_factor = scale.max(2);
            if matches!(state.current_screen(), Screen::PixelArt) {
                state.replace_current(Screen::PixelArt);
            }
            None
        }
        Action::PixelArtApply { loading_only } => {
            if loading_only {
                state.loading_with_spinner = false;
                state.loading_message = Some("Pixelating...".into());
                state.replace_current(Screen::Loading);
                return Some(render_ui(&state));
            }
            state.loading_message = Some("Pixelating...".into());
            state.loading_with_spinner = true;
            state.replace_current(Screen::PixelArt);
            if let Some(path) = state.pixel_art.source_path.clone() {
                let job = WorkerJob::PixelArt {
                    source_path: path,
                    scale: state.pixel_art.scale_factor,
                };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.pixel_art.error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.pixel_art.error = Some("no_image_selected".into());
            }
            None
        }
        Action::KotlinImagePick { path, fd, error } => {
            // Ensure we stay on the image screen
            if !matches!(state.current_screen(), Screen::KotlinImage) {
                state.push_screen(Screen::KotlinImage);
            }
            state.image.result = None;
            let mut fd_handle = FdHandle::new(fd);
            if let Some(err) = error {
                state.image.result = Some(features::kotlin_image::ImageConversionResult {
                    path: None,
                    size: None,
                    format: None,
                    error: Some(err),
                });
            } else {
                if let Some(raw_fd) = fd_handle.take() {
                    // Reuse save_fd_to_temp from dithering module as it is a generic helper
                    match save_fd_to_temp(raw_fd as RawFd, path.as_deref()) {
                        Ok(saved) => {
                            state.image.source_path = Some(saved);
                        }
                        Err(e) => {
                            state.image.result =
                                Some(features::kotlin_image::ImageConversionResult {
                                    path: None,
                                    size: None,
                                    format: None,
                                    error: Some(e),
                                });
                        }
                    }
                } else if let Some(p) = path {
                    state.image.source_path = Some(p);
                } else {
                    state.image.result = Some(features::kotlin_image::ImageConversionResult {
                        path: None,
                        size: None,
                        format: None,
                        error: Some("missing_source".into()),
                    });
                }
            }
            None
        }
        Action::KotlinImageScreen(target) => {
            handle_kotlin_image_screen(state, target);
            None
        }
        Action::KotlinImageResizeScreen => {
            handle_kotlin_image_resize_screen(state);
            None
        }
        Action::KotlinImageResizeSync { bindings } => {
            handle_kotlin_image_resize_sync(state, &bindings);
            None
        }
        Action::KotlinImageResult {
            target,
            result,
            bindings,
        } => {
            handle_kotlin_image_result(state, target, result, Some(&bindings));
            None
        }
        Action::KotlinImageOutputDir { target, output_dir } => {
            handle_kotlin_image_output_dir(state, target, output_dir);
            None
        }
        Action::DitheringScreen => {
            state.push_screen(Screen::Dithering);
            state.dithering_error = None;
            state.dithering_result_path = None;
            None
        }
        Action::DitheringPickImage { path, fd, error } => {
            state.push_screen(Screen::Dithering);
            state.dithering_error = error.clone();
            state.dithering_result_path = None;
            let output_dir = path
                .as_deref()
                .map(|p| {
                    features::storage::output_dir_for(Some(p))
                        .to_string_lossy()
                        .into_owned()
                })
                .unwrap_or_else(|| {
                    features::storage::preferred_temp_dir()
                        .to_string_lossy()
                        .into_owned()
                });
            state.dithering_output_dir = Some(output_dir);
            let mut fd_handle = FdHandle::new(fd);
            if error.is_none() {
                if let Some(raw_fd) = fd_handle.take() {
                    match save_fd_to_temp(raw_fd as RawFd, path.as_deref()) {
                        Ok(saved) => {
                            state.dithering_source_path = Some(saved);
                            state.dithering_error = None;
                        }
                        Err(e) => state.dithering_error = Some(e),
                    }
                } else if let Some(p) = path {
                    state.dithering_source_path = Some(p);
                    state.dithering_error = None;
                } else {
                    state.dithering_error = Some("missing_source".into());
                }
            }
            None
        }
        Action::DitheringSetMode { mode } => {
            state.dithering_mode = mode;
            if matches!(state.current_screen(), Screen::Dithering) {
                state.replace_current(Screen::Dithering);
            }
            None
        }
        Action::DitheringSetPalette { palette } => {
            state.dithering_palette = palette;
            if matches!(state.current_screen(), Screen::Dithering) {
                state.replace_current(Screen::Dithering);
            }
            None
        }
        Action::DitheringApply { loading_only } => {
            if loading_only {
                state.loading_with_spinner = false;
                state.loading_message = Some("Applying dithering...".into());
                state.replace_current(Screen::Loading);
                return Some(render_ui(&state));
            }
            state.loading_message = Some("Applying dithering...".into());
            state.loading_with_spinner = true;
            state.replace_current(Screen::Dithering);
            if let Some(path) = state.dithering_source_path.clone() {
                let output_dir = state.dithering_output_dir.clone();
                let job = WorkerJob::Dithering {
                    source_path: path,
                    mode: state.dithering_mode,
                    palette: state.dithering_palette,
                    output_dir,
                };
                if let Err(e) = STATE.worker().enqueue(job) {
                    state.dithering_error = Some(e);
                }
                #[cfg(test)]
                {
                    apply_worker_results(state);
                }
            } else {
                state.dithering_error = Some("no_image_selected".into());
            }
            None
        }
        _ => None,
    }
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
        Screen::PdfPreview => render_pdf_preview_screen(state),
        Screen::About => render_about_screen(state),
        Screen::SensorLogger => render_sensor_logger_screen(state),
        Screen::TextViewer => render_text_viewer_screen(state),
        Screen::Dithering => render_dithering_screen(state),
        Screen::ArchiveTools => render_archive_screen(state),
        Screen::Compression => render_compression_screen(state),
        Screen::SystemInfo => features::system_info::render_system_info_screen(state),
        Screen::Compass => render_compass_screen(state),
        Screen::Barometer => render_barometer_screen(state),
        Screen::Magnetometer => render_magnetometer_screen(state),
        Screen::MultiHash => render_multi_hash_screen(state),
        Screen::PixelArt => render_pixel_art_screen(state),
        Screen::RegexTester => render_regex_tester_screen(state),
        Screen::MathTool => render_math_tool_screen(state),
        Screen::UuidGenerator => render_uuid_screen(state),
        Screen::PresetManager => render_preset_manager(state),
        Screen::PresetSave => render_save_preset_dialog(state),
        Screen::QrSlideshow => render_qr_slideshow_screen(state),
        Screen::QrReceive => render_qr_receive_screen(state),
    }
}

/// A feature entry for the home menu.
pub struct Feature {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub action: &'static str,
    pub requires_file_picker: bool,
    pub description: &'static str,
}

/// Render the home screen using a catalog of features.
pub fn render_menu(state: &AppState, catalog: &[Feature]) -> Value {
    use crate::ui::{
        Button as UiButton, Card as UiCard, Column as UiColumn, Section as UiSection,
        Text as UiText,
    };

    let mut children = vec![
        serde_json::to_value(UiText::new("🧰 Tool menu").size(22.0)).unwrap(),
        serde_json::to_value(
            UiText::new("✨ Select a tool. Hash tools prompt for a file.").size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiText::new("Legacy notice: MD5 and SHA-1 are not suitable for security; prefer SHA-256 or BLAKE3.")
                .size(12.0),
        )
        .unwrap(),
    ];

    // Quick access row (static for now; prefer high-traffic tools).
    let quick_ids = ["pdf_tools", "text_tools", "text_viewer", "hash_sha256"];
    let quick_buttons: Vec<Value> = catalog
        .iter()
        .filter(|f| quick_ids.contains(&f.id))
        .map(|f| {
            serde_json::to_value(
                UiButton::new(f.name, f.action)
                    .id(f.id)
                    .requires_file_picker(f.requires_file_picker),
            )
            .unwrap()
        })
        .collect();
    if !quick_buttons.is_empty() {
        let quick = UiCard::new(vec![
            serde_json::to_value(UiColumn::new(quick_buttons)).unwrap()
        ])
        .title("⚡ Quick access")
        .padding(12);
        children.push(serde_json::to_value(quick).unwrap());
    }

    let mut grouped: BTreeMap<&str, Vec<&Feature>> = BTreeMap::new();
    for feature in catalog.iter() {
        grouped.entry(feature.category).or_default().push(feature);
    }

    for (category, feats) in grouped {
        let mut section_children: Vec<Value> = Vec::new();
        if category.contains("Hash") {
            section_children.push(
                serde_json::to_value(
                    UiText::new("MD5/SHA-1 are legacy. Prefer SHA-256 or BLAKE3.").size(12.0),
                )
                .unwrap(),
            );
        }
        let list: Vec<Value> = feats
            .iter()
            .map(|f| {
                serde_json::to_value(
                    UiButton::new(&format!("{} – {}", f.name, f.description), f.action)
                        .id(f.id)
                        .requires_file_picker(f.requires_file_picker),
                )
                .unwrap()
            })
            .collect();
        section_children.push(
            serde_json::to_value(UiColumn::new(list).padding(4).content_description(category))
                .unwrap(),
        );

        let subtitle = format!("{} tools", feats.len());
        let mut section = UiSection::new(section_children)
            .title(category)
            .subtitle(&subtitle)
            .padding(12);
        if let Some(first) = category.split_whitespace().next() {
            if first.chars().all(|c| !c.is_ascii_alphanumeric()) {
                section = section.icon(first);
            }
        }
        children.push(serde_json::to_value(section).unwrap());
    }

    if let Some(hash) = &state.last_hash {
        children.push(
            serde_json::to_value(
                UiText::new(&format!(
                    "{}: {}",
                    state
                        .last_hash_algo
                        .clone()
                        .unwrap_or_else(|| "Hash".into()),
                    hash
                ))
                .size(14.0),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Copy last hash", "noop")
                    .copy_text(hash)
                    .id("copy_last_hash_home"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Paste reference (clipboard)", "hash_paste_reference")
                    .id("hash_paste_reference_btn"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new("Show QR for last hash", "hash_qr_last").id("hash_qr_last_btn"),
            )
            .unwrap(),
        );
        if let Some(matches) = state.hash_match {
            let status = if matches {
                "Reference match ✅"
            } else {
                "Reference mismatch ❌"
            };
            children.push(
                serde_json::to_value(
                    UiText::new(status)
                        .size(12.0)
                        .content_description("hash_ref_status"),
                )
                .unwrap(),
            );
        }
    }

    if let Some(status) = &state.sensor_status {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Sensor logger: {}", status))
                    .size(12.0)
                    .content_description("sensor_logger_status_home"),
            )
            .unwrap(),
        );
    }
    if let Some(path) = &state.last_sensor_log {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Last log: {}", path))
                    .size(12.0)
                    .content_description("sensor_logger_path_home"),
            )
            .unwrap(),
        );
    }

    if let Some(err) = &state.last_error {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("Error: {}", err))
                    .size(14.0)
                    .content_description("error_text"),
            )
            .unwrap(),
        );
    }

    serde_json::to_value(UiColumn::new(children).padding(32)).unwrap()
}

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
            id: "multi_hash",
            name: "Multi-hash",
            category: "🔐 Hashes",
            action: "multi_hash_screen",
            requires_file_picker: false,
            description: "Compute MD5, SHA-1, SHA-256, BLAKE3",
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
            id: "pixel_art",
            name: "🟫 Pixel artifier",
            category: "📸 Media",
            action: "pixel_art_screen",
            requires_file_picker: false,
            description: "downscale+nearest upscale",
        },
        Feature {
            id: "regex_tester",
            name: "🔎 Regex tester",
            category: "🧰 Utilities",
            action: "regex_tester_screen",
            requires_file_picker: false,
            description: "test patterns & captures",
        },
        Feature {
            id: "math_tool",
            name: "➗ Math evaluator",
            category: "🧰 Utilities",
            action: "math_tool_screen",
            requires_file_picker: false,
            description: "evaluate expressions & functions",
        },
        Feature {
            id: "uuid_generator",
            name: "🆔 UUID & random string",
            category: "🧰 Utilities",
            action: "uuid_screen",
            requires_file_picker: false,
            description: "uuid v4 + configurable strings",
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
            id: "qr_transfer_sender",
            name: "📡 QR Transfer (sender)",
            category: "🧰 Utilities",
            action: "qr_slideshow_screen",
            requires_file_picker: false,
            description: "slideshow of QR chunks",
        },
        Feature {
            id: "qr_transfer_receiver",
            name: "📥 QR Transfer (receiver)",
            category: "🧰 Utilities",
            action: "qr_receive_screen",
            requires_file_picker: false,
            description: "reassemble pasted QR chunks",
        },
        Feature {
            id: "file_info",
            name: "📂 File Inspector",
            category: "📁 Files",
            action: "file_info_screen",
            requires_file_picker: false,
            description: "size, MIME, and header preview",
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
            id: "archive_compress",
            name: "📦 ZIP Creator",
            category: "📁 Files",
            action: "archive_compress",
            requires_file_picker: true,
            description: "compress file or folder",
        },
        Feature {
            id: "gzip_tools",
            name: "🌀 GZIP",
            category: "📁 Files",
            action: "gzip_screen",
            requires_file_picker: false,
            description: "single-file .gz compress/decompress",
        },
        Feature {
            id: "system_info",
            name: "📊 System panels",
            category: "🧰 Utilities",
            action: "system_info_screen",
            requires_file_picker: false,
            description: "device storage/network/battery snapshot",
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
            id: "pdf_preview",
            name: "📑 PDF viewer",
            category: "📁 Files",
            action: "pdf_preview_screen",
            requires_file_picker: false,
            description: "thumbnails & single-page view",
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
            id: "image_dithering",
            name: "🟪 Retro dithering",
            category: "📸 Media",
            action: "dithering_screen",
            requires_file_picker: false,
            description: "Floyd-Steinberg, Bayer, retro palettes",
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
    use std::thread;
    use std::sync::{atomic::Ordering, Mutex};
    use std::time::{Duration, Instant};
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
        TEST_FORCE_ASYNC_WORKER.store(false, Ordering::SeqCst);
        TEST_WORKER_DELAY_MS.store(0, Ordering::SeqCst);
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

        let state = STATE.ui_lock();
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

        let state = STATE.ui_lock();
        assert_eq!(state.last_hash.as_deref(), Some(SHA256_ABC));
        assert_eq!(state.last_hash_algo.as_deref(), Some("SHA-256"));
        assert!(state.last_error.is_none());
        assert!(matches!(state.current_screen(), Screen::Home));
    }

    #[test]
    fn hash_verify_enqueues_and_releases_mutex() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        TEST_FORCE_ASYNC_WORKER.store(true, Ordering::SeqCst);
        TEST_WORKER_DELAY_MS.store(200, Ordering::SeqCst);

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(SAMPLE_CONTENT.as_bytes()).unwrap();
        file.flush().unwrap();

        let mut command = make_command("hash_verify");
        command.path = Some(file.path().to_string_lossy().into_owned());
        command.bindings = Some(HashMap::from([(
            "hash_reference".into(),
            SHA256_ABC.into(),
        )]));

        let start = Instant::now();
        let ui = handle_command(command).expect("hash verify dispatch should succeed");
        assert!(
            start.elapsed() < Duration::from_millis(100),
            "dispatch held the UI mutex for too long"
        );
        assert_contains_text(&ui, "Computing SHA-256");
        assert!(
            STATE.ui_try_lock().is_some(),
            "state mutex should be free while worker runs"
        );

        std::thread::sleep(Duration::from_millis(250));
        let refreshed =
            handle_command(make_command("init")).expect("refresh after worker should succeed");
        assert_contains_text(&refreshed, SHA256_ABC);

        let state = STATE.ui_lock();
        assert_eq!(state.last_hash.as_deref(), Some(SHA256_ABC));
        assert_eq!(state.hash_match, Some(true));

        TEST_FORCE_ASYNC_WORKER.store(false, Ordering::SeqCst);
        TEST_WORKER_DELAY_MS.store(0, Ordering::SeqCst);
    }

    #[test]
    fn concurrent_jni_call_proceeds_while_worker_runs() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        TEST_FORCE_ASYNC_WORKER.store(true, Ordering::SeqCst);
        TEST_WORKER_DELAY_MS.store(300, Ordering::SeqCst);

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(SAMPLE_CONTENT.as_bytes()).unwrap();
        file.flush().unwrap();

        let mut hash_cmd = make_command("hash_file_sha256");
        hash_cmd.path = Some(file.path().to_string_lossy().into_owned());
        handle_command(hash_cmd).expect("hash dispatch should succeed");

        let start = Instant::now();
        let inc_ui = handle_command(make_command("increment"))
            .expect("increment should not be blocked by worker");
        assert!(
            start.elapsed() < Duration::from_millis(100),
            "increment waited too long for state mutex"
        );
        assert_contains_text(&inc_ui, "Tool menu");

        let state = STATE.ui_lock();
        assert_eq!(state.counter, 1);
        drop(state);

        std::thread::sleep(Duration::from_millis(350));
        handle_command(make_command("init")).expect("refresh should apply worker result");
        let state = STATE.ui_lock();
        assert_eq!(state.last_hash.as_deref(), Some(SHA256_ABC));
        assert!(state.last_error.is_none());

        TEST_FORCE_ASYNC_WORKER.store(false, Ordering::SeqCst);
        TEST_WORKER_DELAY_MS.store(0, Ordering::SeqCst);
    }

    #[test]
    fn archive_open_enqueues_and_releases_mutex() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        TEST_FORCE_ASYNC_WORKER.store(true, Ordering::SeqCst);
        TEST_WORKER_DELAY_MS.store(200, Ordering::SeqCst);

        let mut zip_file = NamedTempFile::new().unwrap();
        {
            let mut writer = zip::ZipWriter::new(&mut zip_file);
            let options =
                FileOptions::default().compression_method(zip::CompressionMethod::Stored);
            writer.start_file("note.txt", options).unwrap();
            writer.write_all(b"hello from zip").unwrap();
            writer.finish().unwrap();
        }

        let fd = File::open(zip_file.path()).unwrap().into_raw_fd();
        let mut open_cmd = make_command("archive_open");
        open_cmd.fd = Some(fd);
        open_cmd.path = Some(zip_file.path().to_string_lossy().into_owned());

        let start = Instant::now();
        let ui = handle_command(open_cmd).expect("archive open dispatch should succeed");
        assert!(
            start.elapsed() < Duration::from_millis(100),
            "dispatch held the UI mutex for too long"
        );
        assert_contains_text(&ui, "Opening archive");
        assert!(
            STATE.ui_try_lock().is_some(),
            "state mutex should be free while archive worker runs"
        );

        std::thread::sleep(Duration::from_millis(250));
        handle_command(make_command("init")).expect("refresh after worker should succeed");
        let state = STATE.ui_lock();
        assert!(state
            .archive
            .entries
            .iter()
            .any(|e| e.name == "note.txt"));
        assert!(matches!(state.current_screen(), Screen::ArchiveTools));

        TEST_FORCE_ASYNC_WORKER.store(false, Ordering::SeqCst);
        TEST_WORKER_DELAY_MS.store(0, Ordering::SeqCst);
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

        let state = STATE.ui_lock();
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

        let state = STATE.ui_lock();
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

        let state = STATE.ui_lock();
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
                STATE.ui_try_lock().and_then(|s| s.text_output.clone())
            })
            .unwrap_or_default();

        assert!(
            result.contains('\n'),
            "expected wrapped text to contain newline, got {result:?}"
        );
        let state = STATE.ui_lock();
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

        let state = STATE.ui_lock();
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
        let state2 = STATE.ui_lock();
        assert_eq!(state2.text_output.as_deref(), Some("a   b"));
    }

    #[test]
    fn back_from_home_does_not_underflow_stack() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let ui = handle_command(make_command("back")).expect("back should succeed");
        assert_contains_text(&ui, "Tool menu");

        let state = STATE.ui_lock();
        assert_eq!(state.nav_depth(), 1);
        assert!(matches!(state.current_screen(), Screen::Home));
    }

    #[test]
    fn back_pops_to_previous_screen() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        handle_command(make_command("text_tools_screen")).expect("screen switch should work");
        {
            let state = STATE.ui_lock();
            assert_eq!(state.nav_depth(), 2);
            assert!(matches!(state.current_screen(), Screen::TextTools));
        }

        handle_command(make_command("back")).expect("back should succeed");
        let state = STATE.ui_lock();
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
            let state = STATE.ui_lock();
            assert!(matches!(state.current_screen(), Screen::Home));
            assert!(state.text_output.is_none());
        }

        let mut restore_cmd = make_command("restore_state");
        restore_cmd.snapshot = Some(snap_str.to_string());
        let ui_after_restore =
            handle_command(restore_cmd).expect("restore should succeed and return UI");
        assert_contains_text(&ui_after_restore, "Result");

        let state = STATE.ui_lock();
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
            let state = STATE.ui_lock();
            assert_eq!(state.text_output.as_deref(), Some("aGk="));
        }

        let mut dec = make_command("text_tools_base64_decode");
        dec.bindings = Some(HashMap::from([("text_input".into(), "aGk=".into())]));
        let ui = handle_command(dec).expect("decode should work");
        assert_contains_text(&ui, "hi");
        let state = STATE.ui_lock();
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
            let state = STATE.ui_lock();
            assert_eq!(state.text_output.as_deref(), Some("6869"));
        }

        let mut dec = make_command("text_tools_hex_decode");
        dec.bindings = Some(HashMap::from([("text_input".into(), "6869".into())]));
        let ui = handle_command(dec).expect("decode should work");
        assert_contains_text(&ui, "hi");
        let state = STATE.ui_lock();
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
        let state = STATE.ui_lock();
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

        let state = STATE.ui_lock();
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
        let state = STATE.ui_lock();
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
        let state = STATE.ui_lock();
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
    fn pdf_preview_screen_sets_state() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        {
            let mut state = STATE.ui_lock();
            state.pdf.source_uri = Some("file://dummy.pdf".into());
            state.pdf.page_count = Some(3);
        }
        let ui = handle_command(make_command("pdf_preview_screen")).expect("preview screen");
        assert_contains_text(&ui, "PDF viewer");
        let state = STATE.ui_lock();
        assert!(matches!(state.current_screen(), Screen::PdfPreview));
        assert!(state.pdf.preview_page.is_none());
    }

    #[test]
    fn pdf_page_open_sets_page() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        {
            let mut state = STATE.ui_lock();
            state.pdf.source_uri = Some("file://dummy.pdf".into());
            state.pdf.page_count = Some(5);
        }
        let mut cmd = make_command("pdf_page_open");
        let mut bindings = HashMap::new();
        bindings.insert("page".to_string(), "2".to_string());
        cmd.bindings = Some(bindings);
        handle_command(cmd).expect("open page");

        let state = STATE.ui_lock();
        assert_eq!(state.pdf.preview_page, Some(2));
        assert!(matches!(state.current_screen(), Screen::PdfPreview));
    }

    fn write_test_image(w: u32, h: u32, color: [u8; 3]) -> NamedTempFile {
        let mut img = image::RgbaImage::new(w, h);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([color[0], color[1], color[2], 255]);
        }
        let file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        img.save(file.path()).unwrap();
        file
    }

    #[test]
    fn pixel_art_screen_sets_defaults() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        handle_command(make_command("pixel_art_screen")).expect("screen");
        let state = STATE.ui_lock();
        assert!(matches!(state.current_screen(), Screen::PixelArt));
        assert_eq!(state.pixel_art.scale_factor, 4);
    }

    #[test]
    fn pixel_art_apply_produces_result() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        let img = write_test_image(8, 8, [10, 20, 30]);
        {
            let mut cmd = make_command("pixel_art_pick");
            cmd.path = Some(img.path().to_string_lossy().into_owned());
            handle_command(cmd).expect("pick");
        }
        let mut apply = make_command("pixel_art_apply");
        apply.loading_only = Some(false);
        let ui = handle_command(apply).expect("apply");
        assert_contains_text(&ui, "Result:");
        let state = STATE.ui_lock();
        assert!(state.pixel_art.result_path.is_some());
        assert!(state.pixel_art.error.is_none());
    }

    #[test]
    fn pixel_art_set_scale_clamps_and_sets() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        handle_command(make_command("pixel_art_screen")).unwrap();
        let mut cmd = make_command("pixel_art_set_scale");
        cmd.bindings = Some(HashMap::from([("scale".into(), "1".into())]));
        handle_command(cmd).unwrap();
        let state = STATE.ui_lock();
        assert_eq!(state.pixel_art.scale_factor, 2);
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
            let state = STATE.ui_lock();
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
        let state = STATE.ui_lock();
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

        let state = STATE.ui_lock();
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
        let state = STATE.ui_lock();
        assert!(matches!(state.current_screen(), Screen::Qr));
        assert!(state.nav_depth() > 1);
    }

    #[test]
    fn sensor_logger_actions_do_not_stack_nav() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        handle_command(make_command("sensor_logger_screen")).unwrap();
        {
            let state = STATE.ui_lock();
            assert_eq!(state.nav_depth(), 2);
            assert!(matches!(state.current_screen(), Screen::SensorLogger));
        }

        let mut start = make_command("sensor_logger_start");
        start.bindings = Some(HashMap::from([("sensor_accel".into(), "true".into())]));
        handle_command(start).unwrap();
        {
            let state = STATE.ui_lock();
            assert_eq!(state.nav_depth(), 2);
            assert!(matches!(state.current_screen(), Screen::SensorLogger));
        }

        handle_command(make_command("back")).unwrap();
        let state = STATE.ui_lock();
        assert_eq!(state.nav_depth(), 1);
        assert!(matches!(state.current_screen(), Screen::Home));
    }

    #[test]
    fn text_viewer_missing_source_sets_error() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        let ui = handle_command(make_command("text_viewer_open")).expect("should return UI");
        assert_contains_text(&ui, "missing_source");
        let state = STATE.ui_lock();
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
            let state = STATE.ui_lock();
            assert_eq!(state.last_hash.as_deref(), Some(MD5_ABC));
            assert_eq!(state.last_hash_algo.as_deref(), Some("MD5"));
        }

        let ui = handle_command(make_command("hash_file_md4"))
            .expect("hash command should still return UI even when failing");

        assert_contains_text(&ui, "missing_path");

        let state = STATE.ui_lock();
        assert_eq!(state.last_hash, None);
        assert_eq!(state.last_error.as_deref(), Some("missing_path"));
        assert_eq!(state.last_hash_algo.as_deref(), Some("MD4"));
    }

    #[test]
    fn text_viewer_open_runs_on_worker() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        TEST_FORCE_ASYNC_WORKER.store(true, Ordering::SeqCst);

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(SAMPLE_CONTENT.as_bytes()).unwrap();
        file.flush().unwrap();

        let mut cmd = make_command("text_viewer_open");
        cmd.path = Some(file.path().to_string_lossy().into_owned());
        let ui_loading = handle_command(cmd).expect("text_viewer_open should enqueue");
        assert_contains_text(&ui_loading, "Loading text");

        thread::sleep(Duration::from_millis(10));
        let _ = handle_command(make_command("snapshot")).unwrap();

        let state = STATE.ui_lock();
        assert!(matches!(state.current_screen(), Screen::TextViewer));
        let content = state.text_view_content.as_deref().unwrap_or("");
        assert!(content.contains(SAMPLE_CONTENT));

        TEST_FORCE_ASYNC_WORKER.store(false, Ordering::SeqCst);
    }

    #[test]
    fn file_info_runs_on_worker_and_updates_state() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();
        TEST_FORCE_ASYNC_WORKER.store(true, Ordering::SeqCst);

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(SAMPLE_CONTENT.as_bytes()).unwrap();
        file.flush().unwrap();

        let mut cmd = make_command("file_info");
        cmd.path = Some(file.path().to_string_lossy().into_owned());
        let ui_loading = handle_command(cmd).expect("file_info should enqueue");
        assert_contains_text(&ui_loading, "Reading file info");

        thread::sleep(Duration::from_millis(10));
        let _ = handle_command(make_command("snapshot")).unwrap();

        let state = STATE.ui_lock();
        assert_eq!(state.current_screen(), Screen::FileInfo);
        assert!(state.last_file_info.is_some());
        assert!(state.last_error.is_none());

        TEST_FORCE_ASYNC_WORKER.store(false, Ordering::SeqCst);
    }

    #[test]
    fn text_viewer_find_clear_removes_query() {
        let _guard = TEST_MUTEX.lock().unwrap();
        reset_state();

        handle_command(make_command("text_viewer_screen")).expect("enter text viewer");

        let mut find_cmd = make_command("text_viewer_find_submit");
        find_cmd.bindings = Some(HashMap::from_iter([("find_query".into(), "needle".into())]));
        handle_command(find_cmd).expect("set find query");

        let mut clear_cmd = make_command("text_viewer_find_clear");
        clear_cmd.bindings = Some(HashMap::from_iter([("find_query".into(), "".into())]));
        let ui = handle_command(clear_cmd).expect("clear find query");

        let state = STATE.ui_lock();
        assert!(state.text_view_find_query.is_none());
        assert_eq!(
            state.text_view_find_match.as_deref(),
            Some("Cleared search")
        );
        drop(state);

        assert!(ui.get("find_query").is_none());
    }
}
fn apply_worker_results(state: &mut AppState) {
    let results = STATE.drain_worker_results();
    if results.is_empty() {
        return;
    }

    for result in results {
        match result {
            WorkerResult::Hash { value } => match value {
                Ok(hash) => {
                    state.last_hash = Some(hash);
                    state.last_error = None;
                }
                Err(e) => {
                    state.last_error = Some(e);
                    state.last_hash = None;
                }
            },
            WorkerResult::MultiHash { value } => match value {
                Ok(results) => {
                    state.multi_hash_results = Some(results);
                    state.multi_hash_error = None;
                }
                Err(e) => {
                    state.multi_hash_error = Some(e);
                    state.multi_hash_results = None;
                }
            },
            WorkerResult::HashVerify { value } => match value {
                Ok(res) => {
                    let cleaned_ref = res.reference.trim().to_ascii_lowercase();
                    let cleaned_hash = res.computed.trim().to_ascii_lowercase();
                    state.hash_reference = Some(res.reference);
                    state.last_hash_algo = Some(hash_label(res.algo).into());
                    state.last_hash = Some(res.computed);
                    state.hash_match = Some(cleaned_ref == cleaned_hash);
                    state.last_error = None;
                    state.replace_current(Screen::HashVerify);
                }
                Err(e) => {
                    state.last_error = Some(e);
                    state.last_hash = None;
                    state.hash_match = None;
                    state.replace_current(Screen::HashVerify);
                }
            },
            WorkerResult::Compression { value } => match value {
                Ok(status) => {
                    state.compression_status = Some(status);
                    state.compression_error = None;
                    state.replace_current(Screen::Compression);
                }
                Err(e) => {
                    state.compression_error = Some(e);
                    state.compression_status = None;
                    state.replace_current(Screen::Compression);
                }
            },
            WorkerResult::Dithering { value } => match value {
                Ok(out) => {
                    state.dithering_result_path = Some(out);
                    state.dithering_error = None;
                    state.replace_current(Screen::Dithering);
                }
                Err(e) => {
                    state.dithering_result_path = None;
                    state.dithering_error = Some(e);
                    state.replace_current(Screen::Dithering);
                }
            },
            WorkerResult::PixelArt { value } => match value {
                Ok(out) => {
                    state.pixel_art.result_path = Some(out);
                    state.pixel_art.error = None;
                    state.replace_current(Screen::PixelArt);
                }
                Err(e) => {
                    state.pixel_art.result_path = None;
                    state.pixel_art.error = Some(e);
                    state.replace_current(Screen::PixelArt);
                }
            },
            WorkerResult::PdfOperation { value } => match value {
                Ok(res) => {
                    state.pdf.last_output = Some(res.out_path);
                    state.pdf.last_error = None;
                    state.pdf.selected_pages = res.selected_pages;
                    state.pdf.page_count = Some(res.page_count);
                    state.pdf.current_title = res.title;
                    if res.source_uri.is_some() {
                        state.pdf.source_uri = res.source_uri;
                    }
                    state.replace_current(Screen::PdfTools);
                }
                Err(e) => {
                    state.pdf.last_error = Some(e);
                    state.pdf.last_output = None;
                    state.replace_current(Screen::PdfTools);
                }
            },
            WorkerResult::ArchiveOpen { value } => match value {
                Ok(res) => {
                    state.archive.path = res.path;
                    state.archive.entries = res.entries;
                    state.archive.truncated = res.truncated;
                    state.archive.error = None;
                    state.archive.last_output = None;
                    state.replace_current(Screen::ArchiveTools);
                }
                Err(e) => {
                    state.archive.error = Some(e);
                    state.archive.last_output = None;
                    state.archive.entries.clear();
                    state.archive.truncated = false;
                    state.replace_current(Screen::ArchiveTools);
                }
            },
            WorkerResult::ArchiveCompress { value } => match value {
                Ok(res) => {
                    state.archive.path = res.open.path;
                    state.archive.entries = res.open.entries;
                    state.archive.truncated = res.open.truncated;
                    state.archive.error = None;
                    state.archive.last_output = Some(res.status);
                    state.replace_current(Screen::ArchiveTools);
                }
                Err(e) => {
                    state.archive.error = Some(e);
                    state.archive.last_output = None;
                    state.archive.entries.clear();
                    state.archive.truncated = false;
                    state.replace_current(Screen::ArchiveTools);
                }
            },
            WorkerResult::ArchiveExtract {
                archive_path,
                value,
            } => match value {
                Ok(status) => {
                    let path_matches = state
                        .archive
                        .path
                        .as_deref()
                        .map(|p| p == archive_path)
                        .unwrap_or(true);
                    if path_matches {
                        if state.archive.path.is_none() {
                            state.archive.path = Some(archive_path);
                        }
                        state.archive.last_output = Some(status);
                        state.archive.error = None;
                        state.replace_current(Screen::ArchiveTools);
                    }
                }
                Err(e) => {
                    let path_matches = state
                        .archive
                        .path
                        .as_deref()
                        .map(|p| p == archive_path)
                        .unwrap_or(true);
                    if path_matches {
                        state.archive.error = Some(e);
                        state.archive.last_output = None;
                        state.replace_current(Screen::ArchiveTools);
                    }
                }
            },
            WorkerResult::FileInfo { value } => match value {
                Ok(info) => {
                    state.last_file_info = Some(serde_json::to_string(&info).unwrap_or_default());
                    state.last_error = None;
                    state.replace_current(Screen::FileInfo);
                }
                Err(e) => {
                    state.last_error = Some(e);
                    state.last_file_info = None;
                    state.replace_current(Screen::FileInfo);
                }
            },
            WorkerResult::PdfSelect { value } => match value {
                Ok(res) => {
                    state.pdf.page_count = Some(res.page_count);
                    state.pdf.current_title = res.title;
                    state.pdf.source_uri = res.source_uri;
                    state.pdf.selected_pages.clear();
                    state.pdf.last_error = None;
                    state.pdf.last_output = None;
                    state.replace_current(Screen::PdfTools);
                }
                Err(e) => {
                    state.pdf.last_error = Some(e);
                    state.pdf.page_count = None;
                    state.pdf.selected_pages.clear();
                    state.pdf.last_output = None;
                    state.replace_current(Screen::PdfTools);
                }
            },
            WorkerResult::TextViewer { value } => match value {
                Ok(res) => {
                    apply_text_view_result(state, res);
                    state.replace_current(Screen::TextViewer);
                }
                Err(e) => {
                    state.text_view_error = Some(e);
                    state.text_view_content = None;
                    state.replace_current(Screen::TextViewer);
                }
            },
            WorkerResult::PdfSetTitle { value } => match value {
                Ok(res) => {
                    state.pdf.last_output = Some(res.out_path.clone());
                    state.pdf.source_uri = res.source_uri.clone().or_else(|| state.pdf.source_uri.clone());
                    state.pdf.current_title = res.title.clone();
                    state.pdf.page_count = Some(res.page_count);
                    state.pdf.last_error = None;
                    state.replace_current(Screen::PdfTools);
                }
                Err(e) => {
                    state.pdf.last_error = Some(e);
                    state.replace_current(Screen::PdfTools);
                }
            },
        }
    }
    state.loading_message = None;
    state.loading_with_spinner = false;
}
