#![allow(dead_code)]

use std::io::{self, Read};

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Char(char),
    Ctrl(char),
    Alt(char),
    Enter,
    Tab,
    Backspace,
    Delete,
    Escape,
    Space,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
    Unknown(Vec<u8>),
}

pub struct InputReader {
    buffer: Vec<u8>,
}

impl InputReader {
    pub fn new() -> Self {
        InputReader {
            buffer: Vec::with_capacity(16),
        }
    }

    pub fn read_key(&mut self) -> Result<Key, Box<dyn std::error::Error>> {
        self.buffer.clear();
        let mut stdin = io::stdin();
        
        // Read first byte
        let mut byte = [0];
        stdin.read_exact(&mut byte)?;
        self.buffer.push(byte[0]);

        // Check for escape sequences
        if byte[0] == 27 {  // ESC
            // Try to read more bytes for escape sequences
            // Set a short timeout to avoid blocking
            let mut additional_bytes = [0; 8];
            match stdin.read(&mut additional_bytes) {
                Ok(n) if n > 0 => {
                    for i in 0..n {
                        self.buffer.push(additional_bytes[i]);
                    }
                }
                _ => {} // No additional bytes, just ESC
            }
        }

        self.parse_key()
    }

    fn parse_key(&self) -> Result<Key, Box<dyn std::error::Error>> {
        if self.buffer.is_empty() {
            return Err("Empty buffer".into());
        }

        match self.buffer[0] {
            // Control characters
            1 => Ok(Key::Ctrl('a')),
            2 => Ok(Key::Ctrl('b')),
            3 => Ok(Key::Ctrl('c')),
            4 => Ok(Key::Ctrl('d')),
            5 => Ok(Key::Ctrl('e')),
            6 => Ok(Key::Ctrl('f')),
            7 => Ok(Key::Ctrl('g')),
            8 => Ok(Key::Backspace),
            9 => Ok(Key::Tab),
            10 | 13 => Ok(Key::Enter),
            11 => Ok(Key::Ctrl('k')),
            12 => Ok(Key::Ctrl('l')),
            14 => Ok(Key::Ctrl('n')),
            15 => Ok(Key::Ctrl('o')),
            16 => Ok(Key::Ctrl('p')),
            17 => Ok(Key::Ctrl('q')),
            18 => Ok(Key::Ctrl('r')),
            19 => Ok(Key::Ctrl('s')),
            20 => Ok(Key::Ctrl('t')),
            21 => Ok(Key::Ctrl('u')),
            22 => Ok(Key::Ctrl('v')),
            23 => Ok(Key::Ctrl('w')),
            24 => Ok(Key::Ctrl('x')),
            25 => Ok(Key::Ctrl('y')),
            26 => Ok(Key::Ctrl('z')),

            // Escape sequences
            27 => self.parse_escape_sequence(),

            // Space
            32 => Ok(Key::Space),

            // Delete
            127 => Ok(Key::Backspace),

            // Regular characters
            c if c >= 32 && c < 127 => Ok(Key::Char(c as char)),

            // Unknown
            _ => Ok(Key::Unknown(self.buffer.clone())),
        }
    }

    fn parse_escape_sequence(&self) -> Result<Key, Box<dyn std::error::Error>> {
        if self.buffer.len() == 1 {
            return Ok(Key::Escape);
        }

        match &self.buffer[1..] {
            // Alt + character
            [c] if *c >= 32 && *c < 127 => Ok(Key::Alt(*c as char)),

            // Arrow keys and other escape sequences
            [91, 65] => Ok(Key::ArrowUp),      // ESC [ A
            [91, 66] => Ok(Key::ArrowDown),    // ESC [ B
            [91, 67] => Ok(Key::ArrowRight),   // ESC [ C
            [91, 68] => Ok(Key::ArrowLeft),    // ESC [ D

            // Function keys
            [79, 80] => Ok(Key::F(1)),         // ESC O P
            [79, 81] => Ok(Key::F(2)),         // ESC O Q
            [79, 82] => Ok(Key::F(3)),         // ESC O R
            [79, 83] => Ok(Key::F(4)),         // ESC O S

            // Extended function keys
            [91, 49, 53, 126] => Ok(Key::F(5)),   // ESC [ 1 5 ~
            [91, 49, 55, 126] => Ok(Key::F(6)),   // ESC [ 1 7 ~
            [91, 49, 56, 126] => Ok(Key::F(7)),   // ESC [ 1 8 ~
            [91, 49, 57, 126] => Ok(Key::F(8)),   // ESC [ 1 9 ~
            [91, 50, 48, 126] => Ok(Key::F(9)),   // ESC [ 2 0 ~
            [91, 50, 49, 126] => Ok(Key::F(10)),  // ESC [ 2 1 ~
            [91, 50, 51, 126] => Ok(Key::F(11)),  // ESC [ 2 3 ~
            [91, 50, 52, 126] => Ok(Key::F(12)),  // ESC [ 2 4 ~

            // Home, End, Page Up, Page Down, Delete
            [91, 72] => Ok(Key::Home),         // ESC [ H
            [91, 70] => Ok(Key::End),          // ESC [ F
            [91, 49, 126] => Ok(Key::Home),    // ESC [ 1 ~
            [91, 52, 126] => Ok(Key::End),     // ESC [ 4 ~
            [91, 53, 126] => Ok(Key::PageUp),  // ESC [ 5 ~
            [91, 54, 126] => Ok(Key::PageDown), // ESC [ 6 ~
            [91, 51, 126] => Ok(Key::Delete),  // ESC [ 3 ~

            // Unknown escape sequence
            _ => Ok(Key::Unknown(self.buffer.clone())),
        }
    }
}

// Helper function to check if a key is printable
impl Key {
    pub fn is_printable(&self) -> bool {
        match self {
            Key::Char(_) | Key::Space => true,
            _ => false,
        }
    }

    pub fn to_char(&self) -> Option<char> {
        match self {
            Key::Char(c) => Some(*c),
            Key::Space => Some(' '),
            Key::Tab => Some('\t'),
            Key::Enter => Some('\n'),
            _ => None,
        }
    }

    pub fn is_ctrl(&self) -> bool {
        matches!(self, Key::Ctrl(_))
    }

    pub fn is_alt(&self) -> bool {
        matches!(self, Key::Alt(_))
    }

    pub fn is_arrow(&self) -> bool {
        matches!(self, Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight)
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Key::F(_))
    }
}

// Event-based input handling
#[derive(Debug, Clone)]
pub enum InputEvent {
    Key(Key),
    Resize(u16, u16), // width, height
    Unknown(Vec<u8>),
}

pub struct EventReader {
    input_reader: InputReader,
}

impl EventReader {
    pub fn new() -> Self {
        EventReader {
            input_reader: InputReader::new(),
        }
    }

    pub fn read_event(&mut self) -> Result<InputEvent, Box<dyn std::error::Error>> {
        // For now, we only handle key events
        // In a more advanced implementation, we could handle window resize events
        let key = self.input_reader.read_key()?;
        Ok(InputEvent::Key(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_properties() {
        assert!(Key::Char('a').is_printable());
        assert!(Key::Space.is_printable());
        assert!(!Key::Ctrl('c').is_printable());
        assert!(!Key::ArrowUp.is_printable());

        assert_eq!(Key::Char('x').to_char(), Some('x'));
        assert_eq!(Key::Space.to_char(), Some(' '));
        assert_eq!(Key::ArrowUp.to_char(), None);

        assert!(Key::Ctrl('c').is_ctrl());
        assert!(!Key::Char('c').is_ctrl());

        assert!(Key::Alt('a').is_alt());
        assert!(!Key::Char('a').is_alt());

        assert!(Key::ArrowUp.is_arrow());
        assert!(Key::ArrowDown.is_arrow());
        assert!(!Key::Char('a').is_arrow());

        assert!(Key::F(1).is_function());
        assert!(!Key::Char('1').is_function());
    }

    #[test]
    fn test_input_reader_creation() {
        let reader = InputReader::new();
        assert_eq!(reader.buffer.len(), 0);
        assert_eq!(reader.buffer.capacity(), 16);
    }

    #[test]
    fn test_event_reader_creation() {
        let _reader = EventReader::new();
        // Just test that it can be created without panicking
    }
}