use std::io::{self, Write};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[cfg(unix)]
use libc::{tcgetattr, tcsetattr, termios, ECHO, ICANON, TCSANOW, VMIN, VTIME};

#[cfg(windows)]
use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
#[cfg(windows)]
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
#[cfg(windows)]
use winapi::um::processenv::GetStdHandle;
#[cfg(windows)]
use winapi::um::winbase::{STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
#[cfg(windows)]
use winapi::um::wincon::{ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT};

pub struct RawTerminal {
    #[cfg(unix)]
    original_termios: Option<termios>,
    #[cfg(windows)]
    original_input_mode: Option<u32>,
    #[cfg(windows)]
    original_output_mode: Option<u32>,
    is_raw: bool,
}

impl RawTerminal {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(RawTerminal {
            #[cfg(unix)]
            original_termios: None,
            #[cfg(windows)]
            original_input_mode: None,
            #[cfg(windows)]
            original_output_mode: None,
            is_raw: false,
        })
    }

    #[allow(dead_code)]
    pub fn enable_raw_mode(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_raw {
            return Ok(());
        }

        #[cfg(unix)]
        {
            let stdin_fd = io::stdin().as_raw_fd();
            let mut termios = unsafe { std::mem::zeroed::<termios>() };
            
            if unsafe { tcgetattr(stdin_fd, &mut termios) } != 0 {
                return Err("Failed to get terminal attributes".into());
            }
            
            self.original_termios = Some(termios);
            
            // Disable canonical mode and echo
            termios.c_lflag &= !(ICANON | ECHO);
            // Set minimum read to 1 byte, no timeout
            termios.c_cc[VMIN] = 1;
            termios.c_cc[VTIME] = 0;
            
            if unsafe { tcsetattr(stdin_fd, TCSANOW, &termios) } != 0 {
                return Err("Failed to set terminal attributes".into());
            }
        }

        #[cfg(windows)]
        {
            unsafe {
                let stdin_handle = GetStdHandle(STD_INPUT_HANDLE);
                let stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);
                
                if stdin_handle == INVALID_HANDLE_VALUE || stdout_handle == INVALID_HANDLE_VALUE {
                    return Err("Failed to get console handles".into());
                }
                
                let mut input_mode = 0;
                let mut output_mode = 0;
                
                if GetConsoleMode(stdin_handle, &mut input_mode) == 0 {
                    return Err("Failed to get input console mode".into());
                }
                
                if GetConsoleMode(stdout_handle, &mut output_mode) == 0 {
                    return Err("Failed to get output console mode".into());
                }
                
                self.original_input_mode = Some(input_mode);
                self.original_output_mode = Some(output_mode);
                
                // Disable line input, echo input, and processed input
                let new_input_mode = input_mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
                
                if SetConsoleMode(stdin_handle, new_input_mode) == 0 {
                    return Err("Failed to set input console mode".into());
                }
            }
        }

        self.is_raw = true;
        Ok(())
    }

    pub fn disable_raw_mode(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_raw {
            return Ok(());
        }

        #[cfg(unix)]
        {
            if let Some(original_termios) = self.original_termios {
                let stdin_fd = io::stdin().as_raw_fd();
                if unsafe { tcsetattr(stdin_fd, TCSANOW, &original_termios) } != 0 {
                    return Err("Failed to restore terminal attributes".into());
                }
            }
        }

        #[cfg(windows)]
        {
            unsafe {
                let stdin_handle = GetStdHandle(STD_INPUT_HANDLE);
                let stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);
                
                if let Some(input_mode) = self.original_input_mode {
                    SetConsoleMode(stdin_handle, input_mode);
                }
                
                if let Some(output_mode) = self.original_output_mode {
                    SetConsoleMode(stdout_handle, output_mode);
                }
            }
        }

        self.is_raw = false;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_raw(&self) -> bool {
        self.is_raw
    }

    #[allow(dead_code)]
    pub fn flush_stdout() -> Result<(), Box<dyn std::error::Error>> {
        io::stdout().flush()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_terminal_size() -> Result<(u16, u16), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            use libc::{ioctl, winsize, TIOCGWINSZ};
            
            let mut ws: winsize = unsafe { std::mem::zeroed() };
            let result = unsafe { ioctl(io::stdout().as_raw_fd(), TIOCGWINSZ, &mut ws) };
            
            if result == 0 {
                Ok((ws.ws_col, ws.ws_row))
            } else {
                // Default fallback
                Ok((80, 24))
            }
        }

        #[cfg(windows)]
        {
            use winapi::um::wincon::{GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO};
            
            unsafe {
                let stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);
                let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
                
                if GetConsoleScreenBufferInfo(stdout_handle, &mut csbi) != 0 {
                    let width = csbi.srWindow.Right - csbi.srWindow.Left + 1;
                    let height = csbi.srWindow.Bottom - csbi.srWindow.Top + 1;
                    Ok((width as u16, height as u16))
                } else {
                    // Default fallback
                    Ok((80, 24))
                }
            }
        }
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        let _ = self.disable_raw_mode();
    }
}

#[allow(dead_code)]
pub fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    let _ = io::stdout().flush();
}

#[allow(dead_code)]
pub fn move_cursor(row: u16, col: u16) {
    print!("\x1B[{};{}H", row, col);
    let _ = io::stdout().flush();
}

#[allow(dead_code)]
pub fn hide_cursor() {
    print!("\x1B[?25l");
    let _ = io::stdout().flush();
}

#[allow(dead_code)]
pub fn show_cursor() {
    print!("\x1B[?25h");
    let _ = io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        let terminal = RawTerminal::new();
        assert!(terminal.is_ok());
        assert!(!terminal.unwrap().is_raw());
    }

    #[test]
    fn test_terminal_size() {
        let size = RawTerminal::get_terminal_size();
        assert!(size.is_ok());
        let (width, height) = size.unwrap();
        assert!(width > 0);
        assert!(height > 0);
    }
}