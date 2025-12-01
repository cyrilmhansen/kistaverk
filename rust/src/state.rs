use crate::features::archive::ArchiveState;
use crate::features::kotlin_image::KotlinImageState;
use crate::features::pdf::PdfState;
use crate::features::sensor_logger::SensorSelection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Screen {
    Home,
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
    Compass,
    Barometer,
    Magnetometer,
    PixelArt,
    RegexTester,
    UuidGenerator,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexTesterState {
    pub pattern: String,
    pub sample_text: String,
    pub match_result: Option<RegexMatchResult>,
    pub error: Option<String>,
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
pub struct AppState {
    pub counter: i32,
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
}

impl AppState {
    // const so it can be used in static initialization
    pub const fn new() -> Self {
        Self {
            counter: 0,
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
            regex_tester: RegexTesterState {
                pattern: String::new(),
                sample_text: String::new(),
                match_result: None,
                error: None,
            },
            uuid_generator: UuidGeneratorState {
                last_uuid: None,
                last_string: None,
                string_length: 16,
                string_charset: StringCharset::Alphanumeric,
            },
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

    pub fn nav_depth(&self) -> usize {
        let depth = self.nav_stack.len();
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
        self.regex_tester.match_result = None;
        self.regex_tester.error = None;
        self.uuid_generator.last_uuid = None;
        self.uuid_generator.last_string = None;
        self.uuid_generator.string_length = 16;
        self.uuid_generator.string_charset = StringCharset::Alphanumeric;
    }
}
