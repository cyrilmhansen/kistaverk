use crate::features::storage::preferred_temp_dir;
use crate::state::{AppState, PixelArtState};
use crate::ui::{maybe_push_back, Button as UiButton, Column as UiColumn, Text as UiText};
use image::imageops::FilterType;
use image::DynamicImage;
use image::GenericImageView;
use serde_json::{json, Value};
use std::fs::File;
use std::os::unix::io::{FromRawFd, RawFd};
use tempfile::Builder;
use rust_i18n::t;

pub fn render_pixel_art_screen(state: &AppState) -> Value {
    let mut children = vec![
        serde_json::to_value(UiText::new(&t!("pixel_art_title")).size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(&t!("pixel_art_description")).size(14.0),
        )
        .unwrap(),
        json!({
            "type": "Button",
            "text": t!("presets_title"),
            "action": "presets_list",
            "id": "pixel_art_presets",
            "payload": { "tool_id": "pixel_art" }
        }),
        json!({
            "type": "Button",
            "text": t!("presets_save_title"),
            "action": "preset_save_dialog",
            "id": "pixel_art_preset_save",
            "payload": { "tool_id": "pixel_art" }
        }),
        serde_json::to_value(
            UiButton::new(&t!("pixel_art_pick_image_button"), "pixel_art_pick")
                .requires_file_picker(true)
                .content_description(&t!("pixel_art_pick_image_content_description")),
        )
        .unwrap(),
    ];

    if let Some(path) = &state.pixel_art.source_path {
        children.push(
            serde_json::to_value(UiText::new(&format!("{}{}", t!("dithering_source_prefix"), path)).size(12.0)).unwrap(),
        );
    }

    let scales = [2u32, 4, 8, 16];
    children.push(serde_json::to_value(UiText::new(&t!("pixel_art_scale_factor")).size(14.0)).unwrap());
    for s in scales {
        children.push(json!({
            "type": "Button",
            "text": format!("{}x", s),
            "action": "pixel_art_set_scale",
            "content_description": if s == state.pixel_art.scale_factor { Some("selected") } else { None::<&str> },
            "payload": { "scale": s }
        }));
    }

    if let Some(err) = &state.pixel_art.error {
        children
            .push(serde_json::to_value(UiText::new(&format!("{}{}", t!("multi_hash_error_prefix"), err)).size(12.0)).unwrap());
    }

    if state.pixel_art.source_path.is_some() {
        children.push(
            serde_json::to_value(UiButton::new(&t!("dithering_apply_button"), "pixel_art_apply").id("pixel_art_apply"))
                .unwrap(),
        );
    }

    if let Some(out) = &state.pixel_art.result_path {
        children.push(
            serde_json::to_value(
                UiText::new(&format!("{}{}", t!("pixel_art_result_prefix"), out))
                    .size(12.0)
                    .content_description("pixel_art_result"),
            )
            .unwrap(),
        );
        children.push(
            serde_json::to_value(
                UiButton::new(&t!("dithering_copy_result_path_button"), "copy_clipboard").copy_text(out),
            )
            .unwrap(),
        );
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

pub fn process_pixel_art(path: &str, factor: u32) -> Result<String, String> {
    let factor = factor.max(2);
    let img = image::open(path).map_err(|e| format!("open_failed:{e}"))?;
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return Err("empty_image".into());
    }
    let down_w = (w / factor.max(1)).max(1);
    let down_h = (h / factor.max(1)).max(1);
    let small = resize_nearest(&img, down_w, down_h);
    let up = small.resize_exact(w, h, FilterType::Nearest);

    let tmp = new_temp_file("pixel_art_", ".png")?;
    up.save(&tmp).map_err(|e| format!("save_failed:{e}"))?;
    let path = tmp
        .into_temp_path()
        .keep()
        .map_err(|e| format!("persist_failed:{e}"))?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "path_utf8".to_string())
}

fn resize_nearest(img: &DynamicImage, w: u32, h: u32) -> DynamicImage {
    img.resize_exact(w, h, FilterType::Nearest)
}

fn new_temp_file(prefix: &str, suffix: &str) -> Result<tempfile::NamedTempFile, String> {
    let dir = preferred_temp_dir();
    Builder::new()
        .prefix(prefix)
        .suffix(suffix)
        .tempfile_in(dir)
        .map_err(|e| format!("tempfile_failed:{e}"))
}

pub fn save_fd_to_temp(fd: RawFd, hint_path: Option<&str>) -> Result<String, String> {
    if fd < 0 {
        return Err("invalid_fd".into());
    }
    let mut reader = unsafe { File::from_raw_fd(fd) };
    let suffix = hint_path
        .and_then(|p| std::path::Path::new(p).extension().and_then(|e| e.to_str()))
        .map(|e| format!(".{}", e))
        .unwrap_or_else(|| ".bin".into());
    let mut tmp = Builder::new()
        .prefix("pixel_src_")
        .suffix(&suffix)
        .tempfile_in(preferred_temp_dir())
        .map_err(|e| format!("tempfile_failed:{e}"))?;
    std::io::copy(&mut reader, &mut tmp).map_err(|e| format!("copy_failed:{e}"))?;
    let path = tmp
        .into_temp_path()
        .keep()
        .map_err(|e| format!("persist_failed:{e}"))?
        .to_string_lossy()
        .into_owned();
    Ok(path)
}

pub fn reset_pixel_art(state: &mut PixelArtState) {
    state.source_path = None;
    state.result_path = None;
    state.error = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn process_keeps_dimensions() {
        let _guard = crate::features::storage::test_env_lock()
            .lock()
            .expect("lock env");
        let prev_temp = std::env::var("KISTAVERK_TEMP_DIR").ok();
        let mut img = RgbaImage::new(8, 8);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = Rgba([(x * 10) as u8, (y * 10) as u8, 0, 255]);
        }
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("KISTAVERK_TEMP_DIR", dir.path());
        let path = dir.path().join("input.png");
        img.save(&path).unwrap();

        let out = process_pixel_art(path.to_str().unwrap(), 4).expect("process ok");
        let out_img = image::open(out).unwrap();
        assert_eq!(out_img.dimensions(), (8, 8));

        match prev_temp {
            Some(v) => std::env::set_var("KISTAVERK_TEMP_DIR", v),
            None => std::env::remove_var("KISTAVERK_TEMP_DIR"),
        }
    }
}
