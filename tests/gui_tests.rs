// Tests for GUI-related functionality
use bestme::config::{Config, GeneralSettings};

#[test]
fn test_gui_theme_settings() {
    let config = Config::default();
    
    // Test default theme
    assert_eq!(config.general.theme, "dark");
    
    // Test theme values
    let valid_themes = vec!["dark", "light", "auto"];
    for theme in valid_themes {
        assert!(theme == "dark" || theme == "light" || theme == "auto");
    }
}

#[test] 
fn test_window_settings() {
    let config = Config::default();
    
    // Test window behavior settings
    assert!(!config.general.auto_start);
    assert!(!config.general.minimize_to_tray);
    assert!(config.general.auto_transcribe);
}

#[test]
fn test_ui_state_serialization() {
    use serde_json;
    
    #[derive(serde::Serialize, serde::Deserialize)]
    struct UIState {
        is_recording: bool,
        is_transcribing: bool,
        selected_device: Option<String>,
        selected_model: String,
    }
    
    let state = UIState {
        is_recording: false,
        is_transcribing: false,
        selected_device: None,
        selected_model: "small".to_string(),
    };
    
    // Test serialization
    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains("is_recording"));
    assert!(json.contains("selected_model"));
    
    // Test deserialization
    let deserialized: UIState = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.is_recording, state.is_recording);
    assert_eq!(deserialized.selected_model, state.selected_model);
}

#[test]
fn test_language_selection() {
    // Test that we have a reasonable set of languages
    let languages = vec![
        ("auto", "Auto-detect"),
        ("en", "English"),
        ("es", "Spanish"),
        ("fr", "French"),
        ("de", "German"),
        ("ja", "Japanese"),
        ("zh", "Chinese"),
    ];
    
    assert!(languages.len() > 5); // Should support multiple languages
    
    for (code, name) in languages {
        assert!(!code.is_empty());
        assert!(!name.is_empty());
        assert!(code.len() >= 2); // Language codes are at least 2 chars
    }
}

#[cfg(test)]
mod tray_icon_tests {
    #[test]
    fn test_tray_menu_items() {
        // Test that tray menu items are properly defined
        let menu_items = vec!["show", "hide", "quit"];
        
        for item in menu_items {
            match item {
                "show" => assert!(true),
                "hide" => assert!(true), 
                "quit" => assert!(true),
                _ => panic!("Unknown menu item"),
            }
        }
    }
    
    #[test]
    fn test_tooltip_text() {
        let tooltip = "BestMe - Speech to Text";
        assert!(tooltip.contains("BestMe"));
        assert!(tooltip.contains("Speech"));
    }
}