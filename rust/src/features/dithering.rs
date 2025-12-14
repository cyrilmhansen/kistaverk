use crate::features::storage::{output_dir_for, preferred_temp_dir};
use crate::state::{AppState, DitheringMode, DitheringPalette};
use crate::ui::{maybe_push_back, Button as UiButton, Column as UiColumn, Text as UiText};
use image::{Rgba, RgbaImage};
use serde_json::{json, Value};
use std::fs;
use std::fs::File;
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::{Path, PathBuf};
use tempfile::Builder;
use rust_i18n::t;

const MONOCHROME: &[[u8; 3]] = &[[0, 0, 0], [255, 255, 255]];
const CGA: &[[u8; 3]] = &[[0, 0, 0], [85, 255, 255], [255, 85, 255], [255, 255, 85]];
const GAME_BOY: &[[u8; 3]] = &[[15, 56, 15], [48, 98, 48], [139, 172, 15], [155, 188, 15]];

const BAYER_4X4: [[i32; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

const BAYER_8X8: [[i32; 8]; 8] = [
    [0, 48, 12, 60, 3, 51, 15, 63],
    [32, 16, 44, 28, 35, 19, 47, 31],
    [8, 56, 4, 52, 11, 59, 7, 55],
    [40, 24, 36, 20, 43, 27, 39, 23],
    [2, 50, 14, 62, 1, 49, 13, 61],
    [34, 18, 46, 30, 33, 17, 45, 29],
    [10, 58, 6, 54, 9, 57, 5, 53],
    [42, 26, 38, 22, 41, 25, 37, 21],
];

const FLOYD_KERNEL: &[(i32, i32, f32)] = &[
    (1, 0, 7.0 / 16.0),
    (-1, 1, 3.0 / 16.0),
    (0, 1, 5.0 / 16.0),
    (1, 1, 1.0 / 16.0),
];

const SIERRA_KERNEL: &[(i32, i32, f32)] = &[
    (1, 0, 2.0 / 4.0),
    (-1, 1, 1.0 / 4.0),
    (0, 1, 1.0 / 4.0),
    (1, 1, 1.0 / 4.0),
];

const ATKINSON_KERNEL: &[(i32, i32, f32)] = &[
    (1, 0, 1.0 / 8.0),
    (2, 0, 1.0 / 8.0),
    (-1, 1, 1.0 / 8.0),
    (0, 1, 1.0 / 8.0),
    (1, 1, 1.0 / 8.0),
    (0, 2, 1.0 / 8.0),
];

fn palette_colors(palette: DitheringPalette) -> &'static [[u8; 3]] {
    match palette {
        DitheringPalette::Monochrome => MONOCHROME,
        DitheringPalette::Cga => CGA,
        DitheringPalette::GameBoy => GAME_BOY,
    }
}

fn nearest_color(palette: &[[u8; 3]], r: f32, g: f32, b: f32) -> [u8; 3] {
    let mut best = palette[0];
    let mut best_dist = f32::MAX;
    for color in palette {
        let dr = r - color[0] as f32;
        let dg = g - color[1] as f32;
        let db = b - color[2] as f32;
        let dist = dr * dr + dg * dg + db * db;
        if dist < best_dist {
            best_dist = dist;
            best = *color;
        }
    }
    best
}

fn apply_error_diffusion(
    input: &RgbaImage,
    palette: &[[u8; 3]],
    kernel: &[(i32, i32, f32)],
) -> RgbaImage {
    let width = input.width() as i32;
    let height = input.height() as i32;
    let mut buffer: Vec<[f32; 3]> = input
        .pixels()
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();
    let mut output = RgbaImage::new(input.width(), input.height());

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let [r, g, b] = buffer[idx];
            let src_alpha = input.get_pixel(x as u32, y as u32)[3];
            let target = nearest_color(palette, r, g, b);
            output.put_pixel(
                x as u32,
                y as u32,
                Rgba([target[0], target[1], target[2], src_alpha]),
            );
            let err = [
                r - target[0] as f32,
                g - target[1] as f32,
                b - target[2] as f32,
            ];
            for (dx, dy, factor) in kernel {
                add_error(&mut buffer, width, height, x + *dx, y + *dy, err, *factor);
            }
        }
    }

    output
}

fn add_error(
    buffer: &mut Vec<[f32; 3]>,
    width: i32,
    height: i32,
    x: i32,
    y: i32,
    err: [f32; 3],
    factor: f32,
) {
    if x >= 0 && x < width && y >= 0 && y < height {
        let idx = (y * width + x) as usize;
        buffer[idx][0] += err[0] * factor;
        buffer[idx][1] += err[1] * factor;
        buffer[idx][2] += err[2] * factor;
    }
}

fn apply_bayer<const N: usize>(
    input: &RgbaImage,
    palette: &[[u8; 3]],
    matrix: &[[i32; N]; N],
) -> RgbaImage {
    let mut output = RgbaImage::new(input.width(), input.height());
    let scale = (N * N) as f32;

    for (idx, pixel) in input.pixels().enumerate() {
        let x = (idx as u32) % input.width();
        let y = (idx as u32) / input.width();
        let threshold = (matrix[(y as usize) % N][(x as usize) % N] as f32 + 0.5) / scale - 0.5;
        let adjust = threshold * 255.0;
        let r = (pixel[0] as f32 + adjust).clamp(0.0, 255.0);
        let g = (pixel[1] as f32 + adjust).clamp(0.0, 255.0);
        let b = (pixel[2] as f32 + adjust).clamp(0.0, 255.0);
        let target = nearest_color(palette, r, g, b);
        output.put_pixel(x, y, Rgba([target[0], target[1], target[2], pixel[3]]));
    }

    output
}

pub fn process_dithering(
    path: &str,
    mode: DitheringMode,
    palette: DitheringPalette,
    output_dir: Option<&str>,
) -> Result<String, String> {
    let img = image::open(path).map_err(|e| format!("open_failed:{e}"))?;
    let rgba = img.to_rgba8();
    let palette = palette_colors(palette);
    let processed = match mode {
        DitheringMode::FloydSteinberg => apply_error_diffusion(&rgba, palette, FLOYD_KERNEL),
        DitheringMode::Sierra => apply_error_diffusion(&rgba, palette, SIERRA_KERNEL),
        DitheringMode::Atkinson => apply_error_diffusion(&rgba, palette, ATKINSON_KERNEL),
        DitheringMode::Bayer4x4 => apply_bayer(&rgba, palette, &BAYER_4X4),
        DitheringMode::Bayer8x8 => apply_bayer(&rgba, palette, &BAYER_8X8),
    };

    let target_dir = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| output_dir_for(Some(path)));
    fs::create_dir_all(&target_dir).map_err(|e| format!("output_dir_create_failed:{e}"))?;
    let tmp = new_temp_file_in("dithered_", ".png", &target_dir)?;
    let path = tmp.into_temp_path();
    let path_buf = path.to_path_buf();
    processed
        .save(&path_buf)
        .map_err(|e| format!("save_failed:{e}"))?;
    let final_path = path_buf
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "path_utf8".to_string())?;
    path.keep().map_err(|e| format!("persist_failed:{e}"))?;
    Ok(final_path)
}

pub fn save_fd_to_temp(fd: RawFd, hint_path: Option<&str>) -> Result<String, String> {
    let suffix = hint_path
        .and_then(|p| Path::new(p).extension().and_then(|e| e.to_str()))
        .map(|ext| format!(".{}", ext))
        .unwrap_or_else(|| ".bin".to_string());
    let mut reader = unsafe { File::from_raw_fd(fd) };
    let mut tmp = new_temp_file("dither_src_", &suffix)?;
    std::io::copy(&mut reader, &mut tmp).map_err(|e| format!("copy_failed:{e}"))?;
    let path = tmp.into_temp_path();
    let path_buf = path.to_path_buf();
    let final_path = path_buf
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "path_utf8".to_string())?;
    path.keep().map_err(|e| format!("persist_failed:{e}"))?;
    Ok(final_path)
}

fn new_temp_file(prefix: &str, suffix: &str) -> Result<tempfile::NamedTempFile, String> {
    let dirs = temp_dirs();
    let mut last_err = None;
    for dir in dirs {
        if let Err(e) = fs::create_dir_all(&dir) {
            last_err = Some(format!("tempdir_mkdir_failed:{e}"));
            continue;
        }
        match Builder::new()
            .prefix(prefix)
            .suffix(suffix)
            .tempfile_in(&dir)
        {
            Ok(f) => return Ok(f),
            Err(e) => last_err = Some(format!("tempfile_failed:{e}")),
        }
    }
    Err(last_err.unwrap_or_else(|| "tempfile_failed".into()))
}

fn new_temp_file_in(
    prefix: &str,
    suffix: &str,
    dir: &Path,
) -> Result<tempfile::NamedTempFile, String> {
    Builder::new()
        .prefix(prefix)
        .suffix(suffix)
        .tempfile_in(dir)
        .map_err(|e| format!("tempfile_failed:{e}"))
}

fn temp_dirs() -> Vec<PathBuf> {
    vec![preferred_temp_dir()]
}

pub fn render_dithering_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new(&t!("dithering_title")).size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(&t!("dithering_description"))
                .size(14.0),
        )
        .unwrap(),
        json!({
            "type": "Button",
            "text": t!("dithering_pick_image_button"),
            "action": "dithering_pick_image",
            "requires_file_picker": true,
            "content_description": t!("dithering_pick_image_content_description")
        }),
        json!({
            "type": "Button",
            "text": t!("presets_title"),
            "action": "presets_list",
            "id": "dithering_presets",
            "payload": { "tool_id": "dithering" }
        }),
        json!({
            "type": "Button",
            "text": t!("presets_save_title"),
            "action": "preset_save_dialog",
            "id": "dithering_preset_save",
            "payload": { "tool_id": "dithering" }
        }),
    ];

    if let Some(path) = &state.dithering_source_path {
        children.push(
            serde_json::to_value(UiText::new(&format!("{}{}", t!("dithering_source_prefix"), path)).size(12.0)).unwrap(),
        );
    }

    let modes = [
        (
            DitheringMode::Atkinson,
            &t!("dithering_mode_atkinson"),
            "dithering_mode_atkinson",
        ),
        (
            DitheringMode::FloydSteinberg,
            &t!("dithering_mode_fs"),
            "dithering_mode_fs",
        ),
        (DitheringMode::Sierra, &t!("dithering_mode_sierra"), "dithering_mode_sierra"),
        (
            DitheringMode::Bayer4x4,
            &t!("dithering_mode_bayer4"),
            "dithering_mode_bayer4",
        ),
        (
            DitheringMode::Bayer8x8,
            &t!("dithering_mode_bayer8"),
            "dithering_mode_bayer8",
        ),
    ];
    children.push(serde_json::to_value(UiText::new(&t!("dithering_algorithm_section")).size(14.0)).unwrap());
    for (mode, label, action) in modes {
        let mut button = UiButton::new(label, action).id(action);
        if mode == state.dithering_mode {
            button = button.content_description("selected");
        }
        children.push(serde_json::to_value(button).unwrap());
    }

    let palettes = [
        (
            DitheringPalette::Monochrome,
            &t!("dithering_palette_monochrome"),
            "dithering_palette_mono",
        ),
        (DitheringPalette::Cga, &t!("dithering_palette_cga"), "dithering_palette_cga"),
        (
            DitheringPalette::GameBoy,
            &t!("dithering_palette_gameboy"),
            "dithering_palette_gb",
        ),
    ];
    children.push(serde_json::to_value(UiText::new(&t!("dithering_palette_section")).size(14.0)).unwrap());
    for (palette, label, action) in palettes {
        let mut button = UiButton::new(label, action).id(action);
        if palette == state.dithering_palette {
            button = button.content_description("selected");
        }
        children.push(serde_json::to_value(button).unwrap());
    }

    if let Some(err) = &state.dithering_error {
        children
            .push(serde_json::to_value(UiText::new(&format!("{}{}", t!("multi_hash_error_prefix"), err)).size(12.0)).unwrap());
    }

    if let Some(result) = &state.dithering_result_path {
        children.push(
            serde_json::to_value(
                UiButton::new(&t!("dithering_copy_result_path_button"), "copy_clipboard")
                    .copy_text(result)
                    .id("copy_dithering_result"),
            )
            .unwrap(),
        );
    }

    if state.dithering_source_path.is_some() {
        children.push(
            serde_json::to_value(
                UiButton::new(&t!("dithering_apply_button"), "dithering_apply")
                    .id("dithering_apply")
                    .content_description("Apply dithering"),
            )
            .unwrap(),
        );
    }

    maybe_push_back(&mut children, state);

    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tempfile::NamedTempFile;

    fn load_app_icon() -> RgbaImage {
        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../app/app/src/main/res/mipmap-xxhdpi/ic_launcher.webp"
        ));
        image::load_from_memory(bytes)
            .expect("should decode app icon")
            .to_rgba8()
    }

    #[test]
    fn atkinson_monochrome_produces_dithered_output() {
        let src = load_app_icon();
        let palette = palette_colors(DitheringPalette::Monochrome);
        let out = apply_error_diffusion(&src, palette, ATKINSON_KERNEL);

        assert_eq!(out.dimensions(), src.dimensions());

        let mut colors = HashSet::new();
        for p in out.pixels() {
            colors.insert([p[0], p[1], p[2]]);
            assert!(
                palette.iter().any(|c| *c == [p[0], p[1], p[2]]),
                "pixel {:?} not in monochrome palette",
                [p[0], p[1], p[2]]
            );
        }

        // Expect both black and white to appear in the dithered icon.
        assert!(
            colors.len() >= 2,
            "dithered output should contain multiple tones, got {colors:?}"
        );
    }

    #[test]
    fn palette_quantization_picks_expected_color() {
        let palette = palette_colors(DitheringPalette::Monochrome);
        let white = nearest_color(palette, 255.0, 255.0, 255.0);
        assert_eq!(white, [255, 255, 255]);

        let black = nearest_color(palette, 0.0, 0.0, 0.0);
        assert_eq!(black, [0, 0, 0]);
    }

    #[test]
    fn zero_sized_image_is_handled() {
        let input = RgbaImage::new(0, 0);
        let palette = palette_colors(DitheringPalette::Monochrome);
        let out = apply_error_diffusion(&input, palette, ATKINSON_KERNEL);
        assert_eq!(out.dimensions(), (0, 0));
    }

    #[test]
    fn invalid_image_path_returns_error() {
        let tmp = NamedTempFile::new().expect("temp file");
        // Leave it empty so decoding fails.
        let result = process_dithering(
            tmp.path().to_str().unwrap(),
            DitheringMode::Atkinson,
            DitheringPalette::Monochrome,
            None,
        );
        assert!(result.is_err(), "expected error, got {:?}", result);
    }
}
