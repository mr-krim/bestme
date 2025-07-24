//! Tests for text injection functionality

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::text_injection::keycodes::{GenericKeycodeMapper, VirtualKey};
    
    #[test]
    fn test_keycode_mapper_basic_chars() {
        let mapper = GenericKeycodeMapper::new();
        
        // Test lowercase letters
        let a_presses = mapper.char_to_keypresses('a');
        assert_eq!(a_presses.len(), 1);
        assert_eq!(a_presses[0].key, VirtualKey::A);
        assert!(!a_presses[0].shift);
        
        // Test uppercase letters
        let a_upper_presses = mapper.char_to_keypresses('A');
        assert_eq!(a_upper_presses.len(), 1);
        assert_eq!(a_upper_presses[0].key, VirtualKey::A);
        assert!(a_upper_presses[0].shift);
        
        // Test numbers
        let num_presses = mapper.char_to_keypresses('5');
        assert_eq!(num_presses.len(), 1);
        assert_eq!(num_presses[0].key, VirtualKey::Key5);
        assert!(!num_presses[0].shift);
        
        // Test special characters requiring shift
        let exclaim_presses = mapper.char_to_keypresses('!');
        assert_eq!(exclaim_presses.len(), 1);
        assert_eq!(exclaim_presses[0].key, VirtualKey::Key1);
        assert!(exclaim_presses[0].shift);
    }
    
    #[test]
    fn test_keycode_mapper_special_chars() {
        let mapper = GenericKeycodeMapper::new();
        
        // Test punctuation
        let period_presses = mapper.char_to_keypresses('.');
        assert_eq!(period_presses.len(), 1);
        assert_eq!(period_presses[0].key, VirtualKey::Period);
        assert!(!period_presses[0].shift);
        
        // Test brackets
        let bracket_presses = mapper.char_to_keypresses('[');
        assert_eq!(bracket_presses.len(), 1);
        assert_eq!(bracket_presses[0].key, VirtualKey::LeftBracket);
        assert!(!bracket_presses[0].shift);
    }
    
    #[test]
    fn test_keycode_mapper_unicode_detection() {
        let mapper = GenericKeycodeMapper::new();
        
        // Test Russian character
        let russian_presses = mapper.char_to_keypresses('Я');
        assert_eq!(russian_presses.len(), 1);
        assert!(russian_presses[0].needs_unicode);
        
        // Test Spanish character
        let spanish_presses = mapper.char_to_keypresses('ñ');
        assert_eq!(spanish_presses.len(), 1);
        assert!(spanish_presses[0].needs_unicode);
        
        // Test German character
        let german_presses = mapper.char_to_keypresses('ü');
        assert_eq!(german_presses.len(), 1);
        assert!(german_presses[0].needs_unicode);
    }
    
    #[test]
    fn test_injection_config() {
        let config = InjectionConfig::default();
        assert_eq!(config.type_delay_ms, 10);
        assert!(config.restore_clipboard);
        assert!(!config.allow_in_password_fields);
        
        let custom_config = InjectionConfig {
            type_delay_ms: 50,
            restore_clipboard: false,
            allow_in_password_fields: true,
            ..Default::default()
        };
        assert_eq!(custom_config.type_delay_ms, 50);
        assert!(!custom_config.restore_clipboard);
        assert!(custom_config.allow_in_password_fields);
    }
    
    #[test]
    fn test_injection_mode() {
        use serde_json;
        
        // Test serialization
        let type_mode = InjectionMode::Type {
            delay_ms: 20,
            simulate_typing: true,
        };
        let json = serde_json::to_string(&type_mode).unwrap();
        assert!(json.contains("Type"));
        assert!(json.contains("delay_ms"));
        
        let paste_mode = InjectionMode::Paste {
            restore_clipboard: true,
            use_clipboard: true,
        };
        let json = serde_json::to_string(&paste_mode).unwrap();
        assert!(json.contains("Paste"));
        assert!(json.contains("restore_clipboard"));
    }
    
    #[test]
    fn test_window_info_default() {
        let window = WindowInfo::default();
        assert_eq!(window.app_name, "Unknown");
        assert!(!window.is_elevated);
        assert!(!window.is_focused);
        assert!(window.title.is_empty());
    }
    
    #[test]
    fn test_permission_status() {
        let status = PermissionStatus {
            available: true,
            accessibility_granted: Some(true),
            has_required_privileges: true,
            message: "All permissions granted".to_string(),
            required_actions: vec![],
        };
        
        assert!(status.available);
        assert_eq!(status.accessibility_granted, Some(true));
        assert!(status.has_required_privileges);
        assert!(status.required_actions.is_empty());
    }
    
    #[test]
    fn test_app_profile() {
        let profile = AppProfile {
            app_name: "TestApp".to_string(),
            app_path: Some("/path/to/app".to_string()),
            mode: InjectionMode::Direct,
            enabled: true,
            custom_settings: None,
        };
        
        assert_eq!(profile.app_name, "TestApp");
        assert!(profile.enabled);
        assert!(matches!(profile.mode, InjectionMode::Direct));
    }
    
    #[cfg(feature = "text-injection")]
    #[tokio::test]
    async fn test_injection_manager_creation() {
        use crate::text_injection::InjectionManager;
        
        let config = InjectionConfig::default();
        let manager = InjectionManager::new(config);
        
        // Manager creation should succeed on supported platforms
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        assert!(manager.is_ok());
        
        // On unsupported platforms, it should fail
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        assert!(manager.is_err());
    }
    
    #[cfg(all(feature = "text-injection", target_os = "windows"))]
    #[tokio::test]
    async fn test_windows_injector_permissions() {
        use crate::text_injection::windows::WindowsInjector;
        
        let injector = WindowsInjector::new().unwrap();
        let permissions = injector.check_permissions();
        
        // Windows doesn't require special permissions for SendInput
        assert!(permissions.available);
        assert!(permissions.has_required_privileges);
    }
    
    #[cfg(all(feature = "text-injection", target_os = "macos"))]
    #[tokio::test]
    async fn test_macos_injector_permissions() {
        use crate::text_injection::macos::MacOSInjector;
        
        let injector = MacOSInjector::new().unwrap();
        let permissions = injector.check_permissions();
        
        // macOS requires accessibility permissions
        assert!(permissions.available);
        // Permission status depends on system state
        assert!(permissions.accessibility_granted.is_some());
    }
    
    #[cfg(all(feature = "text-injection", target_os = "linux"))]
    #[tokio::test]
    async fn test_linux_injector_display_detection() {
        use crate::text_injection::linux::LinuxInjector;
        
        let injector = LinuxInjector::new().unwrap();
        let permissions = injector.check_permissions();
        
        // Availability depends on display server
        if std::env::var("DISPLAY").is_ok() {
            // X11 should be available
            assert!(permissions.message.contains("X11"));
        } else if std::env::var("WAYLAND_DISPLAY").is_ok() {
            // Wayland has limitations
            assert!(permissions.message.contains("Wayland"));
        }
    }
    
    #[tokio::test]
    async fn test_context_detector() {
        use crate::text_injection::context::ContextDetector;
        
        let detector = ContextDetector::new();
        
        // Try to get active window - this might fail in test environment
        let window_result = detector.get_active_window().await;
        
        // In a test environment, this might fail, which is okay
        if let Ok(window) = window_result {
            // If it succeeds, verify the window info has expected fields
            assert!(!window.app_name.is_empty());
            assert!(window.is_focused);
        }
    }
    
    #[test]
    fn test_modifiers() {
        let mods = Modifiers {
            shift: true,
            ctrl: false,
            alt: true,
            meta: false,
        };
        
        assert!(mods.shift);
        assert!(!mods.ctrl);
        assert!(mods.alt);
        assert!(!mods.meta);
        
        // Test any() method
        assert!(mods.any());
        
        let no_mods = Modifiers::default();
        assert!(!no_mods.any());
    }
}

#[cfg(all(test, feature = "text-injection"))]
mod integration_tests {
    use super::super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    #[tokio::test]
    async fn test_injection_manager_inject_text() {
        let config = InjectionConfig {
            type_delay_ms: 0, // No delay for tests
            ..Default::default()
        };
        
        let manager_result = InjectionManager::new(config);
        if let Ok(manager) = manager_result {
            // Test with empty text
            let result = manager.inject_text("").await;
            assert!(result.is_ok());
            
            // Test with simple text (won't actually inject in test environment)
            let result = manager.inject_text("test").await;
            // This might fail in test environment without proper permissions
            // but the API should work
            let _ = result;
        }
    }
    
    #[tokio::test]
    async fn test_injection_manager_mode_switching() {
        let config = InjectionConfig::default();
        
        if let Ok(mut manager) = InjectionManager::new(config) {
            // Test setting different modes
            manager.set_mode(InjectionMode::Type {
                delay_ms: 50,
                simulate_typing: true,
            });
            
            manager.set_mode(InjectionMode::Paste {
                restore_clipboard: false,
                use_clipboard: true,
            });
            
            manager.set_mode(InjectionMode::Direct);
            
            // All mode switches should succeed
        }
    }
    
    #[tokio::test]
    async fn test_app_profile_matching() {
        let profiles = vec![
            AppProfile {
                app_name: "chrome".to_string(),
                app_path: None,
                mode: InjectionMode::Paste {
                    restore_clipboard: true,
                    use_clipboard: true,
                },
                enabled: true,
                custom_settings: None,
            },
            AppProfile {
                app_name: "code".to_string(),
                app_path: Some("/usr/bin/code".to_string()),
                mode: InjectionMode::Type {
                    delay_ms: 5,
                    simulate_typing: true,
                },
                enabled: true,
                custom_settings: None,
            },
        ];
        
        // Test profile matching logic
        let window = WindowInfo {
            title: "Google Chrome".to_string(),
            app_name: "chrome".to_string(),
            app_path: Some("/usr/bin/google-chrome".to_string()),
            pid: Some(1234),
            is_elevated: false,
            is_focused: true,
        };
        
        // Find matching profile
        let matched = profiles.iter().find(|p| p.app_name == window.app_name);
        assert!(matched.is_some());
        assert!(matches!(matched.unwrap().mode, InjectionMode::Paste { .. }));
    }
}