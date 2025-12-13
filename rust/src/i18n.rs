use crate::state::AppState;

pub fn update_locale(state: &mut AppState, locale_str: &str) {
    let normalized = normalize_locale(locale_str);
    state.locale = normalized.to_string();
    rust_i18n::set_locale(&normalized);
}

fn normalize_locale(locale_str: &str) -> &str {
    let trimmed = locale_str.trim();
    if trimmed.is_empty() {
        return "en";
    }

    // rust-i18n looks up compiled locales by name (e.g. "en", "is"), so normalize
    // incoming BCP-47 tags like "fr-FR" / "en_US" down to a supported language.
    let lower = trimmed.to_ascii_lowercase().replace('_', "-");
    let lang = lower.split('-').next().unwrap_or("en");

    match lang {
        "is" => "is",
        "en" => "en",
        _ => "en",
    }
}

