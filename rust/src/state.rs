use crate::features::archive::ArchiveState;
use crate::features::hex_editor::HexEditorState;
use crate::features::kotlin_image::KotlinImageState;
use crate::features::logic::LogicState;
use crate::features::pdf::PdfState;
use crate::features::jwt::JwtState;
use crate::features::presets::PresetState;
use crate::features::qr_transfer::{QrReceiveState, QrSlideshowState};
use crate::features::scripting::ScriptingState;
use crate::features::sensor_logger::SensorSelection;
use crate::features::sql_engine::{QueryResult, SqlEngine, TableInfo};
use crate::features::system_info::SystemInfoState;
use crate::features::vault::VaultState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Screen {
    Home,
    Ruler,
    ShaderDemo,
    KotlinImage,
    Dithering,
    HashVerify,
    MultiHash,
    FileInfo,
    TextTools,
    Loading,
    ProgressDemo,
    Qr,
    ColorTools,
    PdfTools,
    PdfPreview,
    About,
    SensorLogger,
    TextViewer,
    ArchiveTools,
    Compression,
    SystemInfo,
    Compass,
    Barometer,
    Magnetometer,
    PixelArt,
    RegexTester,
    UuidGenerator,
    PresetManager,
    PresetSave,
    QrSlideshow,
    QrReceive,
    MathTool,
    Vault,
    Logic,
    Jwt,
    HexEditor,
    Plotting,
    SqlQuery,
    Scripting,
    Scheduler,
    UnitConverter,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum UnitCategory {
    Length,
    Mass,
    Temperature,
    DigitalStorage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitConverterState {
    pub category: UnitCategory,
    pub from_unit: String,
    pub to_unit: String,
    pub input_value: String,
    pub output_value: String,
}

impl UnitConverterState {
    pub const fn new() -> Self {
        Self {
            category: UnitCategory::Length,
            from_unit: String::new(),
            to_unit: String::new(),
            input_value: String::new(),
            output_value: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DitheringMode {
    FloydSteinberg,
    Bayer4x4,
    Bayer8x8,
    Sierra,
    Atkinson,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DitheringPalette {
    Monochrome,
    Cga,
    GameBoy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiHashResults {
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub blake3: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelArtState {
    pub source_path: Option<String>,
    pub result_path: Option<String>,
    pub scale_factor: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexMatchResult {
    pub matched: bool,
    pub groups: Vec<Option<String>>,
    pub match_text: String,
    pub start_index: usize,
    pub end_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexTesterState {
    pub pattern: String,
    pub sample_text: String,
    pub match_results: Vec<RegexMatchResult>,
    pub error: Option<String>,
    pub global_mode: bool,
    pub common_patterns: Vec<String>,
}

impl RegexTesterState {
    pub const fn new() -> Self {
        Self {
            pattern: String::new(),
            sample_text: String::new(),
            match_results: Vec::new(),
            error: None,
            global_mode: false,
            common_patterns: Vec::new(),
        }
    }

    pub fn init_common_patterns(&mut self) {
        self.common_patterns = vec![
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(), // Email
            r"\b(?:\d{1,3}\.){3}\d{1,3}\b".to_string(), // IPv4
            r"\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b".to_string(), // IPv6
            r"\b\d{4}-\d{2}-\d{2}\b".to_string(), // Date YYYY-MM-DD
            r"\b\d{2}:\d{2}:\d{2}\b".to_string(), // Time HH:MM:SS
            r"\b(?:https?|ftp):\/\/[^\s/$.?#].[^\s]*\b".to_string(), // URL
        ];
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyState {
    pub query: String,
}

impl DependencyState {
    pub const fn new() -> Self {
        Self {
            query: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.query.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: u32,
    pub name: String,
    pub action: String,
    pub cron: String,
    pub enabled: bool,
    pub last_run_epoch: Option<i64>,
    pub last_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerLog {
    pub task_id: u32,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerState {
    pub tasks: Vec<ScheduledTask>,
    pub form_name: String,
    pub form_action: String,
    pub form_cron: String,
    pub last_error: Option<String>,
    pub logs: Vec<SchedulerLog>,
    pub next_id: u32,
}

impl SchedulerState {
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            form_name: String::new(),
            form_action: String::new(),
            form_cron: String::new(),
            last_error: None,
            logs: Vec::new(),
            next_id: 1,
        }
    }

    pub fn reset(&mut self) {
        self.tasks.clear();
        self.form_name.clear();
        self.form_action.clear();
        self.form_cron.clear();
        self.last_error = None;
        self.logs.clear();
        self.next_id = 1;
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StringCharset {
    Alphanumeric,
    Numeric,
    Alpha,
    Hex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UuidGeneratorState {
    pub last_uuid: Option<String>,
    pub last_string: Option<String>,
    pub string_length: u32,
    pub string_charset: StringCharset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathHistoryEntry {
    pub expression: String,
    pub result: String,
    pub error_estimate: Option<f64>,
    pub precision_bits: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlotType {
    Line,
    Scatter,
    Histogram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlottingState {
    pub file_path: Option<String>,
    pub display_path: Option<String>,
    pub headers: Vec<String>,
    pub x_col: Option<String>,
    pub y_col: Option<String>,
    pub plot_type: PlotType,
    pub generated_svg: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlQueryState {
    pub query: String,
    pub result: Option<QueryResult>,
    pub tables: Vec<TableInfo>,
    pub query_history: Vec<String>,
    pub error: Option<String>,
}

impl SqlQueryState {
    pub const fn new() -> Self {
        Self {
            query: String::new(),
            result: None,
            tables: Vec::new(),
            query_history: Vec::new(),
            error: None,
        }
    }
}

impl PlottingState {
    pub const fn new() -> Self {
        Self {
            file_path: None,
            display_path: None,
            headers: Vec::new(),
            x_col: None,
            y_col: None,
            plot_type: PlotType::Line,
            generated_svg: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathToolState {
    pub expression: String,
    pub history: Vec<MathHistoryEntry>,
    pub error: Option<String>,
    /// Precision setting in bits (0 = f64, 64+ = arbitrary precision via rug::Float)
    pub precision_bits: u32,
    /// Cumulative floating-point error for the current session
    pub cumulative_error: f64,
}

impl MathToolState {
    pub const fn new() -> Self {
        Self {
            expression: String::new(),
            history: Vec::new(),
            error: None,
            precision_bits: 0, // Default to f64 precision
            cumulative_error: 0.0, // Start with zero error
        }
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub counter: i32,
    pub locale: String,
    pub preferred_locale: String,
    pub home_filter: String,
    pub theme_mode: Option<String>,
    pub nav_stack: Vec<Screen>,
    pub last_hash: Option<String>,
    pub last_error: Option<String>,
    pub last_shader: Option<String>,
    pub last_hash_algo: Option<String>,
    pub hash_reference: Option<String>,
    pub hash_match: Option<bool>,
    pub image: KotlinImageState,
    pub last_file_info: Option<String>,
    pub text_input: Option<String>,
    pub text_output: Option<String>,
    pub text_operation: Option<String>,
    pub text_aggressive_trim: bool,
    pub loading_message: Option<String>,
    pub progress_status: Option<String>,
    pub loading_with_spinner: bool,
    pub dependencies: DependencyState,
    pub last_qr_base64: Option<String>,
    pub pdf: PdfState,
    pub last_sensor_log: Option<String>,
    pub sensor_status: Option<String>,
    pub sensor_interval_ms: Option<u64>,
    pub sensor_selection: Option<SensorSelection>,
    pub text_view_content: Option<String>,
    pub text_view_path: Option<String>,
    pub text_view_cached_path: Option<String>,
    pub text_view_error: Option<String>,
    pub text_view_hex_preview: Option<String>,
    pub text_view_language: Option<String>,
    pub text_view_dark: bool,
    pub text_view_line_numbers: bool,
    pub text_view_find_query: Option<String>,
    pub text_view_find_match: Option<String>,
    pub text_view_total_bytes: Option<u64>,
    pub text_view_loaded_bytes: u64,
    pub text_view_has_more: bool,
    pub text_view_window_offset: u64,
    pub text_view_has_previous: bool,
    pub archive: ArchiveState,
    pub compression_status: Option<String>,
    pub compression_error: Option<String>,
    pub compass_angle_radians: f64,
    pub compass_error: Option<String>,
    pub barometer_hpa: Option<f64>,
    pub barometer_error: Option<String>,
    pub magnetometer_ut: Option<f64>,
    pub magnetometer_error: Option<String>,
    pub compass_filter_angle: Option<f64>,
    pub barometer_filter_value: Option<f64>,
    pub magnetometer_filter_value: Option<f64>,
    pub multi_hash_results: Option<MultiHashResults>,
    pub multi_hash_error: Option<String>,
    pub dithering_source_path: Option<String>,
    pub dithering_result_path: Option<String>,
    pub dithering_mode: DitheringMode,
    pub dithering_palette: DitheringPalette,
    pub dithering_error: Option<String>,
    pub dithering_output_dir: Option<String>,
    pub pixel_art: PixelArtState,
    pub regex_tester: RegexTesterState,
    pub uuid_generator: UuidGeneratorState,
    pub system_info: SystemInfoState,
    pub preset_state: PresetState,
    pub qr_slideshow: QrSlideshowState,
    pub qr_receive: QrReceiveState,
    pub math_tool: MathToolState,
    pub vault: VaultState,
    pub logic: LogicState,
    pub jwt: JwtState,
    pub hex_editor: HexEditorState,
    pub plotting: PlottingState,
    pub sql_query: SqlQueryState,
    pub scripting: ScriptingState,
    pub scheduler: SchedulerState,
    pub unit_converter: UnitConverterState,
    #[serde(skip)]
    pub sql_engine: Option<SqlEngine>,
    #[serde(skip)]
    pub toast: Option<String>,
    #[serde(skip)]
    pub haptic: bool,
}

impl AppState {
    // const so it can be used in static initialization
    pub const fn new() -> Self {
        Self {
            counter: 0,
            locale: String::new(),
            preferred_locale: String::new(),
            home_filter: String::new(),
            theme_mode: None,
            nav_stack: Vec::new(),
            last_hash: None,
            last_error: None,
            last_shader: None,
            last_hash_algo: None,
            hash_reference: None,
            hash_match: None,
            image: KotlinImageState::new(),
            last_file_info: None,
            text_input: None,
            text_output: None,
            text_operation: None,
            text_aggressive_trim: false,
            loading_message: None,
            progress_status: None,
            loading_with_spinner: true,
            dependencies: DependencyState::new(),
            last_qr_base64: None,
            pdf: PdfState::new(),
            last_sensor_log: None,
            sensor_status: None,
            sensor_interval_ms: None,
            sensor_selection: None,
            text_view_content: None,
            text_view_path: None,
            text_view_cached_path: None,
            text_view_error: None,
            text_view_hex_preview: None,
            text_view_language: None,
            text_view_dark: false,
            text_view_line_numbers: false,
            text_view_find_query: None,
            text_view_find_match: None,
            text_view_total_bytes: None,
            text_view_loaded_bytes: 0,
            text_view_has_more: false,
            text_view_window_offset: 0,
            text_view_has_previous: false,
            archive: ArchiveState::new(),
            compression_status: None,
            compression_error: None,
            compass_angle_radians: 0.0,
            compass_error: None,
            barometer_hpa: None,
            barometer_error: None,
            magnetometer_ut: None,
            magnetometer_error: None,
            compass_filter_angle: None,
            barometer_filter_value: None,
            magnetometer_filter_value: None,
            multi_hash_results: None,
            multi_hash_error: None,
            dithering_source_path: None,
            dithering_result_path: None,
            dithering_mode: DitheringMode::Atkinson,
            dithering_palette: DitheringPalette::Monochrome,
            dithering_error: None,
            dithering_output_dir: None,
            pixel_art: PixelArtState {
                source_path: None,
                result_path: None,
                scale_factor: 4,
                error: None,
            },
            regex_tester: RegexTesterState::new(),
            uuid_generator: UuidGeneratorState {
                last_uuid: None,
                last_string: None,
                string_length: 16,
                string_charset: StringCharset::Alphanumeric,
            },
            system_info: SystemInfoState::new(),
            preset_state: PresetState::new(),
            qr_slideshow: QrSlideshowState::new(),
            qr_receive: QrReceiveState::new(),
            math_tool: MathToolState::new(),
            vault: VaultState::new(),
            logic: LogicState::new(),
            jwt: JwtState::new(),
            hex_editor: HexEditorState::new(),
            plotting: PlottingState::new(),
            sql_query: SqlQueryState::new(),
            scripting: ScriptingState::new(),
            scheduler: SchedulerState::new(),
            unit_converter: UnitConverterState::new(),
            sql_engine: None,
            toast: None,
            haptic: false,
        }
    }

    pub fn ensure_regex_patterns_initialized(&mut self) {
        if self.regex_tester.common_patterns.is_empty() {
            self.regex_tester.init_common_patterns();
        }
    }

    pub fn ensure_navigation(&mut self) {
        if self.nav_stack.is_empty() {
            self.nav_stack.push(Screen::Home);
        }
    }

    pub fn current_screen(&self) -> Screen {
        self.nav_stack.last().cloned().unwrap_or(Screen::Home)
    }

    pub fn nav_depth(&self) -> u32 {
        let depth = self.nav_stack.len() as u32;
        if depth == 0 {
            1
        } else {
            depth
        }
    }

    pub fn push_screen(&mut self, screen: Screen) {
        self.ensure_navigation();
        self.nav_stack.push(screen);
    }

    pub fn replace_current(&mut self, screen: Screen) {
        self.ensure_navigation();
        if let Some(last) = self.nav_stack.last_mut() {
            *last = screen;
        } else {
            self.nav_stack.push(screen);
        }
    }

    pub fn pop_screen(&mut self) {
        self.ensure_navigation();
        if self.nav_stack.len() > 1 {
            self.nav_stack.pop();
        }
    }

    pub fn reset_navigation(&mut self) {
        self.nav_stack.clear();
        self.nav_stack.push(Screen::Home);
    }

    pub fn reset_runtime(&mut self) {
        self.counter = 0;
        self.last_hash = None;
        self.last_error = None;
        self.last_shader = None;
        self.last_hash_algo = None;
        self.home_filter.clear();
        self.theme_mode = None;
        self.toast = None;
        self.haptic = false;
        self.hash_reference = None;
        self.hash_match = None;
        self.image.reset();
        self.last_file_info = None;
        self.text_input = None;
        self.text_output = None;
        self.text_operation = None;
        self.text_aggressive_trim = false;
        self.loading_message = None;
        self.progress_status = None;
        self.loading_with_spinner = true;
        self.dependencies.reset();
        self.last_qr_base64 = None;
        self.pdf.reset();
        self.last_sensor_log = None;
        self.sensor_status = None;
        self.sensor_interval_ms = None;
        self.sensor_selection = None;
        self.text_view_content = None;
        self.text_view_path = None;
        self.text_view_cached_path = None;
        self.text_view_error = None;
        self.text_view_hex_preview = None;
        self.text_view_language = None;
        self.text_view_dark = false;
        self.text_view_line_numbers = false;
        self.text_view_find_query = None;
        self.text_view_find_match = None;
        self.text_view_total_bytes = None;
        self.text_view_loaded_bytes = 0;
        self.text_view_has_more = false;
        self.text_view_window_offset = 0;
        self.text_view_has_previous = false;
        self.archive.reset();
        self.compression_status = None;
        self.compression_error = None;
        self.compass_angle_radians = 0.0;
        self.compass_error = None;
        self.barometer_hpa = None;
        self.barometer_error = None;
        self.magnetometer_ut = None;
        self.magnetometer_error = None;
        self.compass_filter_angle = None;
        self.barometer_filter_value = None;
        self.magnetometer_filter_value = None;
        self.multi_hash_results = None;
        self.multi_hash_error = None;
        self.dithering_source_path = None;
        self.dithering_result_path = None;
        self.dithering_mode = DitheringMode::Atkinson;
        self.dithering_palette = DitheringPalette::Monochrome;
        self.dithering_error = None;
        self.dithering_output_dir = None;
        self.pixel_art.source_path = None;
        self.pixel_art.result_path = None;
        self.pixel_art.scale_factor = 4;
        self.pixel_art.error = None;
        self.regex_tester.pattern.clear();
        self.regex_tester.sample_text.clear();
        self.regex_tester.match_results.clear();
        self.regex_tester.error = None;
        self.regex_tester.global_mode = false;
        self.uuid_generator.last_uuid = None;
        self.uuid_generator.last_string = None;
        self.uuid_generator.string_length = 16;
        self.uuid_generator.string_charset = StringCharset::Alphanumeric;
        self.system_info = SystemInfoState::new();
        self.preset_state.reset();
        self.qr_slideshow.reset();
        self.qr_receive.reset();
        self.math_tool = MathToolState::new();
        self.vault = VaultState::new();
        self.logic = LogicState::new();
        self.jwt = JwtState::new();
        self.hex_editor = HexEditorState::new();
        self.plotting = PlottingState::new();
        self.scheduler.reset();
        self.unit_converter = UnitConverterState::new();
        self.image.batch_queue.clear();
        self.pdf.merge_queue.clear();
    }
}
