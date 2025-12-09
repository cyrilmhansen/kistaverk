use crate::state::{AppState, PlotType};
use crate::ui::{
    maybe_push_back, Button as UiButton, Column as UiColumn, Grid as UiGrid, HtmlView as UiHtmlView,
    Section as UiSection, Text as UiText, TextInput as UiTextInput,
};
use poloto::build;
use poloto::prelude::PlotIterator;
use poloto::plotnum::HasDefaultTicks;
use serde_json::{json, Value};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::FromRawFd;

const DEFAULT_HEIGHT_DP: u32 = 320;
const HIST_BINS: usize = 10;

pub fn render_plotting_screen(state: &AppState) -> Value {
    let plotting = &state.plotting;
    let mut children = vec![
        serde_json::to_value(UiText::new("The Lab: Data Plotting").size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(
                plotting
                    .display_path
                    .as_deref()
                    .unwrap_or("Pick a CSV file to begin."),
            )
            .size(12.0),
        )
        .unwrap(),
        serde_json::to_value(UiButton::new("Pick CSV", "plotting_pick").requires_file_picker(true))
            .unwrap(),
    ];

    if !plotting.headers.is_empty() {
        children.push(
            serde_json::to_value(UiText::new(&format!(
                "Columns: {}",
                plotting.headers.join(", ")
            )))
            .unwrap(),
        );
    }

    if let Some(err) = &plotting.error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {err}")).size(12.0)).unwrap(),
        );
    }

    let inputs = UiSection::new(vec![
        json!(
            UiTextInput::new("plot_x_col")
                .hint("X column name")
                .text(plotting.x_col.as_deref().unwrap_or(""))
                .single_line(true)
                .debounce_ms(200)
                .action_on_submit("plotting_set_x")
        ),
        json!(
            UiTextInput::new("plot_y_col")
                .hint("Y column name (ignored for histogram)")
                .text(plotting.y_col.as_deref().unwrap_or(""))
                .single_line(true)
                .debounce_ms(200)
                .action_on_submit("plotting_set_y")
        ),
    ])
    .title("Columns")
    .padding(12);
    children.push(serde_json::to_value(inputs).unwrap());

    let plot_buttons = UiGrid::new(vec![
        json!(UiButton::new("Line", "plotting_type_line")),
        json!(UiButton::new("Scatter", "plotting_type_scatter")),
        json!(UiButton::new("Histogram", "plotting_type_hist")),
    ])
    .columns(3)
    .padding(8);
    children.push(serde_json::to_value(plot_buttons).unwrap());

    children.push(
        serde_json::to_value(UiButton::new("Plot", "plotting_generate")).unwrap(),
    );

    if let Some(svg_html) = &plotting.generated_svg {
        children.push(
            serde_json::to_value(UiHtmlView::new(svg_html).height_dp(DEFAULT_HEIGHT_DP)).unwrap(),
        );
    }

    maybe_push_back(&mut children, state);
    serde_json::to_value(UiColumn::new(children).padding(20)).unwrap()
}

pub fn load_headers(path: &str) -> Result<Vec<String>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(|e| format!("csv_open_failed:{e}"))?;
    let headers = rdr
        .headers()
        .map_err(|e| format!("csv_headers_failed:{e}"))?
        .iter()
        .map(|s| s.to_string())
        .collect();
    Ok(headers)
}

pub fn generate_plot(state: &mut crate::state::PlottingState) -> Result<(), String> {
    let path = state
        .file_path
        .clone()
        .ok_or_else(|| "no_file_selected".to_string())?;

    match state.plot_type {
        PlotType::Histogram => {
            let col = state
                .x_col
                .clone()
                .or_else(|| state.y_col.clone())
                .ok_or_else(|| "select_column".to_string())?;
            let values = load_column_values(&path, &col)?;
            let series = histogram_points(&values, HIST_BINS);
            let plots = poloto::plots!(build::plot("histogram").histogram(series));
            let svg = render_svg(plots, "Histogram", "Bins", "Count")?;
            state.generated_svg = Some(wrap_html("Histogram", &svg));
            state.error = None;
        }
        PlotType::Line | PlotType::Scatter => {
            let x = state
                .x_col
                .clone()
                .ok_or_else(|| "missing_x_column".to_string())?;
            let y = state
                .y_col
                .clone()
                .ok_or_else(|| "missing_y_column".to_string())?;
            let pairs = load_xy_pairs(&path, &x, &y)?;
            if pairs.is_empty() {
                return Err("no_rows".into());
            }
            let plots = match state.plot_type {
                PlotType::Line => poloto::plots!(build::plot("series").line(pairs)),
                PlotType::Scatter => poloto::plots!(build::plot("series").scatter(pairs)),
                _ => unreachable!(),
            };
            let svg = render_svg(plots, "Plot", &x, &y)?;
            state.generated_svg = Some(wrap_html("Plot", &svg));
            state.error = None;
        }
    }

    Ok(())
}

fn render_svg<T>(plots: T, title: &str, x_label: &str, y_label: &str) -> Result<String, String>
where
    T: PlotIterator,
    <T::L as build::Point>::X: HasDefaultTicks,
    <T::L as build::Point>::Y: HasDefaultTicks,
{
    poloto::frame_build()
        .data(plots)
        .build_and_label((title, x_label, y_label))
        .append_to(poloto::header().light_theme())
        .render_string()
        .map_err(|e| format!("render_failed:{e}"))
}

fn wrap_html(title: &str, svg: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{title}</title>
  <style>
    body {{ margin: 0; padding: 12px; background: #0f111a; color: #f5f5f5; }}
    svg {{ width: 100%; height: auto; background: #0f111a; }}
  </style>
</head>
<body>
  {svg}
</body>
</html>"#
    )
}

fn load_column_values(path: &str, col: &str) -> Result<Vec<f64>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(|e| format!("csv_open_failed:{e}"))?;
    let headers = rdr
        .headers()
        .map_err(|e| format!("csv_headers_failed:{e}"))?;
    let idx = headers
        .iter()
        .position(|h| h == col)
        .ok_or_else(|| "column_not_found".to_string())?;
    let mut vals = Vec::new();
    for rec in rdr.records() {
        let record = rec.map_err(|e| format!("csv_record_failed:{e}"))?;
        if let Some(raw) = record.get(idx) {
            if let Ok(v) = raw.trim().parse::<f64>() {
                vals.push(v);
            }
        }
    }
    Ok(vals)
}

fn load_xy_pairs(path: &str, x_col: &str, y_col: &str) -> Result<Vec<(f64, f64)>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(|e| format!("csv_open_failed:{e}"))?;
    let headers = rdr
        .headers()
        .map_err(|e| format!("csv_headers_failed:{e}"))?;
    let x_idx = headers
        .iter()
        .position(|h| h == x_col)
        .ok_or_else(|| "x_not_found".to_string())?;
    let y_idx = headers
        .iter()
        .position(|h| h == y_col)
        .ok_or_else(|| "y_not_found".to_string())?;
    let mut pairs = Vec::new();
    for rec in rdr.records() {
        let record = rec.map_err(|e| format!("csv_record_failed:{e}"))?;
        let x = record.get(x_idx).unwrap_or("").trim();
        let y = record.get(y_idx).unwrap_or("").trim();
        if let (Ok(xv), Ok(yv)) = (x.parse::<f64>(), y.parse::<f64>()) {
            pairs.push((xv, yv));
        }
    }
    Ok(pairs)
}

fn histogram_points(values: &[f64], bins: usize) -> Vec<(f64, f64)> {
    if values.is_empty() || bins == 0 {
        return Vec::new();
    }
    let min = values
        .iter()
        .cloned()
        .fold(f64::INFINITY, |a, b| a.min(b));
    let max = values
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));
    if !min.is_finite() || !max.is_finite() || max == min {
        return Vec::new();
    }
    let bin_size = (max - min) / bins as f64;
    let mut counts = vec![0usize; bins];
    for v in values {
        let mut idx = ((v - min) / bin_size) as usize;
        if idx >= bins {
            idx = bins - 1;
        }
        counts[idx] += 1;
    }
    counts
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let start = min + i as f64 * bin_size;
            let center = start + bin_size / 2.0;
            (center, *c as f64)
        })
        .collect()
}

pub fn copy_csv_from_fd(fd: i32, fallback_name: &str) -> Result<String, String> {
    if fd < 0 {
        return Err("invalid_fd".into());
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    let dir = crate::features::storage::preferred_temp_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("temp_dir_failed:{e}"))?;
    let target = dir.join(fallback_name);
    let mut out = File::create(&target).map_err(|e| format!("temp_open_failed:{e}"))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| format!("read_failed:{e}"))?;
    out.write_all(&buf)
        .map_err(|e| format!("write_failed:{e}"))?;
    target
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "temp_path_invalid_utf8".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PlottingState;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn loads_headers() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "x,y").unwrap();
        writeln!(tmp, "1,2").unwrap();
        let headers = load_headers(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(headers, vec!["x", "y"]);
    }

    #[test]
    fn generates_line_plot_svg() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "x,y").unwrap();
        writeln!(tmp, "1,2").unwrap();
        writeln!(tmp, "2,3").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let mut state = PlottingState::new();
        state.file_path = Some(path);
        state.x_col = Some("x".into());
        state.y_col = Some("y".into());
        state.plot_type = PlotType::Line;

        generate_plot(&mut state).expect("plot");
        let html = state.generated_svg.clone().unwrap();
        assert!(html.contains("<svg"), "html: {html}");
    }
}
