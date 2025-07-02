//! Input utilities for handling keyboard input in terminal applications.

use std::io::{self, Read};

/// A function similar to the DOS batch CHOICE command that waits for a keypress
/// from a specified set of valid choices.
///
/// # Arguments
///
/// * `choices` - A string containing all valid characters to listen for.
/// * `case_sensitive` - Whether the choices are case sensitive.
/// * `prompt` - An optional prompt to display before waiting for input.
///
/// # Returns
///
/// Returns the character that was pressed as a `char`.
///
/// # Examples
///
/// ```
/// use utils::input::choice;
///
/// // Wait for user to press Y, N, or Escape
/// let result = choice("YNy\x1B", false, Some("Continue? [Y/N] "));
/// match result {
///     'Y' | 'y' => println!("User chose Yes"),
///     'N' | 'n' => println!("User chose No"),
///     '\x1B' => println!("User pressed Escape"),
///     _ => unreachable!(),
/// }
/// ```
pub fn choice(choices: &str, case_sensitive: bool, prompt: Option<&str>) -> char {
    // Print prompt if provided
    if let Some(text) = prompt {
        print!("{}", text);
        let _ = io::Write::flush(&mut io::stdout());
    }

    // Prepare choices for comparison
    let choices_vec: Vec<char> = if case_sensitive {
        choices.chars().collect()
    } else {
        let mut chars = Vec::new();
        for c in choices.chars() {
            chars.push(c.to_lowercase().next().unwrap());
            chars.push(c.to_uppercase().next().unwrap());
        }
        chars
    };

    // Read single key presses until a valid choice is made
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = [0; 1];

    loop {
        if let Ok(1) = stdin.read(&mut buffer) {
            let pressed = buffer[0] as char;
            if choices_vec.contains(&pressed)
                || (!case_sensitive
                    && choices_vec.contains(&pressed.to_lowercase().next().unwrap()))
            {
                return pressed;
            }
            // Invalid key, ignore and continue listening
        }
    }
}

/// A version of the choice function that uses crossterm for better
/// terminal handling. Must be used in a context where terminal raw mode
/// is appropriate.
///
/// Requires the `crossterm` feature to be enabled.
#[cfg(feature = "crossterm")]
pub fn crossterm_choice(
    choices: &str,
    case_sensitive: bool,
    prompt: Option<&str>,
) -> io::Result<char> {
    use crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        terminal, ExecutableCommand,
    };
    use std::io::Write;

    // Print prompt if provided
    if let Some(text) = prompt {
        print!("{}", text);
        io::stdout().flush()?;
    }

    // Prepare choices for comparison
    let choices_vec: Vec<char> = if case_sensitive {
        choices.chars().collect()
    } else {
        let mut chars = Vec::new();
        for c in choices.chars() {
            chars.push(c.to_lowercase().next().unwrap());
            chars.push(c.to_uppercase().next().unwrap());
        }
        chars
    };

    // Enable raw mode
    terminal::enable_raw_mode()?;

    let result = loop {
        if let Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
            // Only process key press events (not key releases)
            if kind == KeyEventKind::Press {
                match code {
                    KeyCode::Char(c) => {
                        if choices_vec.contains(&c)
                            || (!case_sensitive
                                && choices_vec.contains(&c.to_lowercase().next().unwrap()))
                        {
                            break Ok(c);
                        }
                    }
                    KeyCode::Esc => {
                        if choices.contains('\x1B') {
                            break Ok('\x1B');
                        }
                    }
                    KeyCode::Enter => {
                        if choices.contains('\r') || choices.contains('\n') {
                            break Ok('\n');
                        }
                    }
                    // Handle other special keys if needed
                    _ => {}
                }
            }
        }
    };

    // Disable raw mode
    terminal::disable_raw_mode()?;

    result
}
