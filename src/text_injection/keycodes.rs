//! Cross-platform keycode mappings

use std::collections::HashMap;

/// Virtual key codes (platform-agnostic)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualKey {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Modifiers
    Shift, Control, Alt, Meta, // Meta is Cmd on macOS, Win on Windows
    
    // Special keys
    Space, Enter, Tab, Backspace, Delete,
    Escape, CapsLock, NumLock, ScrollLock,
    
    // Navigation
    Left, Right, Up, Down,
    Home, End, PageUp, PageDown,
    
    // Punctuation and symbols
    Minus, Equals, LeftBracket, RightBracket,
    Semicolon, Quote, Backslash, Comma, Period, Slash,
    Grave, // Backtick
    
    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadMultiply, NumpadAdd, NumpadSubtract,
    NumpadDecimal, NumpadDivide,
}

/// Modifier keys state
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Key press with modifiers
#[derive(Debug, Clone)]
pub struct KeyPress {
    pub key: VirtualKey,
    pub modifiers: Modifiers,
}

/// Trait for platform-specific keycode conversion
pub trait KeycodeMapper {
    /// Get platform-specific keycode for a virtual key
    fn get_keycode(&self, key: VirtualKey) -> Option<u16>;
    
    /// Convert a character to key presses
    fn char_to_keypresses(&self, ch: char) -> Vec<KeyPress>;
}

/// Generic keycode mapper with character mapping
pub struct GenericKeycodeMapper {
    char_map: HashMap<char, Vec<KeyPress>>,
}

impl GenericKeycodeMapper {
    pub fn new() -> Self {
        let mut mapper = Self {
            char_map: HashMap::new(),
        };
        mapper.init_char_map();
        mapper
    }
    
    fn init_char_map(&mut self) {
        // Lowercase letters
        for (i, ch) in ('a'..='z').enumerate() {
            self.char_map.insert(ch, vec![KeyPress {
                key: match i {
                    0 => VirtualKey::A,
                    1 => VirtualKey::B,
                    2 => VirtualKey::C,
                    3 => VirtualKey::D,
                    4 => VirtualKey::E,
                    5 => VirtualKey::F,
                    6 => VirtualKey::G,
                    7 => VirtualKey::H,
                    8 => VirtualKey::I,
                    9 => VirtualKey::J,
                    10 => VirtualKey::K,
                    11 => VirtualKey::L,
                    12 => VirtualKey::M,
                    13 => VirtualKey::N,
                    14 => VirtualKey::O,
                    15 => VirtualKey::P,
                    16 => VirtualKey::Q,
                    17 => VirtualKey::R,
                    18 => VirtualKey::S,
                    19 => VirtualKey::T,
                    20 => VirtualKey::U,
                    21 => VirtualKey::V,
                    22 => VirtualKey::W,
                    23 => VirtualKey::X,
                    24 => VirtualKey::Y,
                    25 => VirtualKey::Z,
                    _ => unreachable!(),
                },
                modifiers: Modifiers::default(),
            }]);
        }
        
        // Uppercase letters
        for (i, ch) in ('A'..='Z').enumerate() {
            self.char_map.insert(ch, vec![KeyPress {
                key: match i {
                    0 => VirtualKey::A,
                    1 => VirtualKey::B,
                    2 => VirtualKey::C,
                    3 => VirtualKey::D,
                    4 => VirtualKey::E,
                    5 => VirtualKey::F,
                    6 => VirtualKey::G,
                    7 => VirtualKey::H,
                    8 => VirtualKey::I,
                    9 => VirtualKey::J,
                    10 => VirtualKey::K,
                    11 => VirtualKey::L,
                    12 => VirtualKey::M,
                    13 => VirtualKey::N,
                    14 => VirtualKey::O,
                    15 => VirtualKey::P,
                    16 => VirtualKey::Q,
                    17 => VirtualKey::R,
                    18 => VirtualKey::S,
                    19 => VirtualKey::T,
                    20 => VirtualKey::U,
                    21 => VirtualKey::V,
                    22 => VirtualKey::W,
                    23 => VirtualKey::X,
                    24 => VirtualKey::Y,
                    25 => VirtualKey::Z,
                    _ => unreachable!(),
                },
                modifiers: Modifiers { shift: true, ..Default::default() },
            }]);
        }
        
        // Numbers
        for (i, ch) in ('0'..='9').enumerate() {
            self.char_map.insert(ch, vec![KeyPress {
                key: match i {
                    0 => VirtualKey::Num0,
                    1 => VirtualKey::Num1,
                    2 => VirtualKey::Num2,
                    3 => VirtualKey::Num3,
                    4 => VirtualKey::Num4,
                    5 => VirtualKey::Num5,
                    6 => VirtualKey::Num6,
                    7 => VirtualKey::Num7,
                    8 => VirtualKey::Num8,
                    9 => VirtualKey::Num9,
                    _ => unreachable!(),
                },
                modifiers: Modifiers::default(),
            }]);
        }
        
        // Common punctuation
        self.char_map.insert(' ', vec![KeyPress { key: VirtualKey::Space, modifiers: Modifiers::default() }]);
        self.char_map.insert('\n', vec![KeyPress { key: VirtualKey::Enter, modifiers: Modifiers::default() }]);
        self.char_map.insert('\t', vec![KeyPress { key: VirtualKey::Tab, modifiers: Modifiers::default() }]);
        self.char_map.insert('.', vec![KeyPress { key: VirtualKey::Period, modifiers: Modifiers::default() }]);
        self.char_map.insert(',', vec![KeyPress { key: VirtualKey::Comma, modifiers: Modifiers::default() }]);
        self.char_map.insert(';', vec![KeyPress { key: VirtualKey::Semicolon, modifiers: Modifiers::default() }]);
        self.char_map.insert('\'', vec![KeyPress { key: VirtualKey::Quote, modifiers: Modifiers::default() }]);
        self.char_map.insert('-', vec![KeyPress { key: VirtualKey::Minus, modifiers: Modifiers::default() }]);
        self.char_map.insert('=', vec![KeyPress { key: VirtualKey::Equals, modifiers: Modifiers::default() }]);
        self.char_map.insert('[', vec![KeyPress { key: VirtualKey::LeftBracket, modifiers: Modifiers::default() }]);
        self.char_map.insert(']', vec![KeyPress { key: VirtualKey::RightBracket, modifiers: Modifiers::default() }]);
        self.char_map.insert('\\', vec![KeyPress { key: VirtualKey::Backslash, modifiers: Modifiers::default() }]);
        self.char_map.insert('/', vec![KeyPress { key: VirtualKey::Slash, modifiers: Modifiers::default() }]);
        self.char_map.insert('`', vec![KeyPress { key: VirtualKey::Grave, modifiers: Modifiers::default() }]);
        
        // Shifted punctuation
        self.char_map.insert('!', vec![KeyPress { key: VirtualKey::Num1, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('@', vec![KeyPress { key: VirtualKey::Num2, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('#', vec![KeyPress { key: VirtualKey::Num3, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('$', vec![KeyPress { key: VirtualKey::Num4, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('%', vec![KeyPress { key: VirtualKey::Num5, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('^', vec![KeyPress { key: VirtualKey::Num6, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('&', vec![KeyPress { key: VirtualKey::Num7, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('*', vec![KeyPress { key: VirtualKey::Num8, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('(', vec![KeyPress { key: VirtualKey::Num9, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert(')', vec![KeyPress { key: VirtualKey::Num0, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('_', vec![KeyPress { key: VirtualKey::Minus, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('+', vec![KeyPress { key: VirtualKey::Equals, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('{', vec![KeyPress { key: VirtualKey::LeftBracket, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('}', vec![KeyPress { key: VirtualKey::RightBracket, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('|', vec![KeyPress { key: VirtualKey::Backslash, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert(':', vec![KeyPress { key: VirtualKey::Semicolon, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('"', vec![KeyPress { key: VirtualKey::Quote, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('<', vec![KeyPress { key: VirtualKey::Comma, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('>', vec![KeyPress { key: VirtualKey::Period, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('?', vec![KeyPress { key: VirtualKey::Slash, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        self.char_map.insert('~', vec![KeyPress { key: VirtualKey::Grave, modifiers: Modifiers { shift: true, ..Default::default() } }]);
        
        // Spanish special characters
        self.add_spanish_chars();
        
        // German special characters
        self.add_german_chars();
        
        // Russian characters (Cyrillic)
        self.add_russian_chars();
    }
    
    fn add_spanish_chars(&mut self) {
        // Spanish special characters - these would need special handling or Unicode input
        // For now, marking them as needing Unicode input
        let spanish_chars = vec![
            'á', 'é', 'í', 'ó', 'ú', 'ü', 'ñ',
            'Á', 'É', 'Í', 'Ó', 'Ú', 'Ü', 'Ñ',
            '¿', '¡'
        ];
        
        // These characters typically require:
        // - Alt codes on Windows
        // - Compose key sequences on Linux
        // - Option key combinations on macOS
        // For now, we'll mark them for Unicode input
        for ch in spanish_chars {
            self.char_map.insert(ch, vec![]); // Empty vec indicates Unicode input needed
        }
    }
    
    fn add_german_chars(&mut self) {
        // German special characters
        let german_chars = vec![
            'ä', 'ö', 'ü', 'ß',
            'Ä', 'Ö', 'Ü'
        ];
        
        // These also need special handling
        for ch in german_chars {
            self.char_map.insert(ch, vec![]); // Empty vec indicates Unicode input needed
        }
    }
    
    fn add_russian_chars(&mut self) {
        // Russian Cyrillic alphabet
        // These require keyboard layout switching or Unicode input
        let russian_lowercase = vec![
            'а', 'б', 'в', 'г', 'д', 'е', 'ё', 'ж', 'з', 'и', 'й',
            'к', 'л', 'м', 'н', 'о', 'п', 'р', 'с', 'т', 'у', 'ф',
            'х', 'ц', 'ч', 'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я'
        ];
        
        let russian_uppercase = vec![
            'А', 'Б', 'В', 'Г', 'Д', 'Е', 'Ё', 'Ж', 'З', 'И', 'Й',
            'К', 'Л', 'М', 'Н', 'О', 'П', 'Р', 'С', 'Т', 'У', 'Ф',
            'Х', 'Ц', 'Ч', 'Ш', 'Щ', 'Ъ', 'Ы', 'Ь', 'Э', 'Ю', 'Я'
        ];
        
        for ch in russian_lowercase.into_iter().chain(russian_uppercase) {
            self.char_map.insert(ch, vec![]); // Empty vec indicates Unicode input needed
        }
    }
    
    /// Get key presses for a character
    pub fn get_keypresses(&self, ch: char) -> Option<&Vec<KeyPress>> {
        self.char_map.get(&ch)
    }
}

impl KeycodeMapper for GenericKeycodeMapper {
    fn get_keycode(&self, _key: VirtualKey) -> Option<u16> {
        // This is platform-specific, so we return None here
        None
    }
    
    fn char_to_keypresses(&self, ch: char) -> Vec<KeyPress> {
        self.get_keypresses(ch).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_char_to_keypresses() {
        let mapper = GenericKeycodeMapper::new();
        
        // Test lowercase letter
        let presses = mapper.get_keypresses('a').unwrap();
        assert_eq!(presses.len(), 1);
        assert_eq!(presses[0].key, VirtualKey::A);
        assert!(!presses[0].modifiers.shift);
        
        // Test uppercase letter
        let presses = mapper.get_keypresses('A').unwrap();
        assert_eq!(presses.len(), 1);
        assert_eq!(presses[0].key, VirtualKey::A);
        assert!(presses[0].modifiers.shift);
        
        // Test number
        let presses = mapper.get_keypresses('5').unwrap();
        assert_eq!(presses.len(), 1);
        assert_eq!(presses[0].key, VirtualKey::Num5);
        assert!(!presses[0].modifiers.shift);
        
        // Test shifted symbol
        let presses = mapper.get_keypresses('!').unwrap();
        assert_eq!(presses.len(), 1);
        assert_eq!(presses[0].key, VirtualKey::Num1);
        assert!(presses[0].modifiers.shift);
    }
}