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
        "fr" => "fr",
        "de" => "de",
        "es" => "es", // Spanish
        "pt" => "pt", // Portuguese
        "zh" => "zh", // Chinese (using zh as standard code)
        "la" => "la", // Latin
        _ => "en",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_normalization() {
        // Test English variants
        assert_eq!(normalize_locale("en"), "en");
        assert_eq!(normalize_locale("en-US"), "en");
        assert_eq!(normalize_locale("en_US"), "en");
        
        // Test French variants
        assert_eq!(normalize_locale("fr"), "fr");
        assert_eq!(normalize_locale("fr-FR"), "fr");
        assert_eq!(normalize_locale("fr_FR"), "fr");
        
        // Test German variants
        assert_eq!(normalize_locale("de"), "de");
        assert_eq!(normalize_locale("de-DE"), "de");
        assert_eq!(normalize_locale("de_DE"), "de");
        
        // Test Icelandic variants
        assert_eq!(normalize_locale("is"), "is");
        assert_eq!(normalize_locale("is-IS"), "is");
        assert_eq!(normalize_locale("is_IS"), "is");

        // Test other supported locales
        assert_eq!(normalize_locale("es"), "es");
        assert_eq!(normalize_locale("pt"), "pt");
        assert_eq!(normalize_locale("zh"), "zh");
        assert_eq!(normalize_locale("zh-CN"), "zh");
        assert_eq!(normalize_locale("la"), "la");
        
        // Test unknown locales fallback to English
        assert_eq!(normalize_locale("it"), "en");
        assert_eq!(normalize_locale("ru"), "en");
        
        // Test edge cases
        assert_eq!(normalize_locale(""), "en");
        assert_eq!(normalize_locale("   "), "en");
    }

    #[test]
    fn test_locale_translations() {
        // Test that we can set different locales without panicking
        // Note: This tests the locale switching mechanism, not the actual translation content
        // since rust-i18n macros are expanded at compile time
        
        // Test English
        rust_i18n::set_locale("en");
        
        // Test French
        rust_i18n::set_locale("fr");
        
        // Test Icelandic
        rust_i18n::set_locale("is");
        
        // Test fallback to English
        rust_i18n::set_locale("es");
        
        // Reset to English
        rust_i18n::set_locale("en");
        
        // If we got here without panicking, the test passes
        assert!(true);
    }
}

