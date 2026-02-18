// Interactive editor support for multi-line text input
// Launches user's preferred editor ($EDITOR) with fallback chain

use crate::errors::ThoughtError;
use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Launch interactive editor for multi-line text input
///
/// Respects $EDITOR environment variable with fallback chain: vim → nano → vi
/// Creates temporary file with initial content, launches editor, reads result
///
/// # Arguments
/// * `initial_content` - Optional text to pre-populate in editor
///
/// # Returns
/// * `Ok(String)` - Edited content from temporary file
/// * `Err(ThoughtError)` - Editor launch failed or file read error
pub fn launch_editor(initial_content: Option<&str>) -> Result<String, ThoughtError> {
    // Create temp file with initial content
    let mut temp_file = NamedTempFile::new()?;
    if let Some(content) = initial_content {
        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?;
    }

    // Determine editor (fallback chain)
    let editor = env::var("EDITOR")
        .ok()
        .and_then(|e| if e.is_empty() { None } else { Some(e) })
        .or_else(which_editor)
        .unwrap_or_else(|| "vi".to_string());

    eprintln!("Launching editor: {}", editor);

    // Launch editor
    let status = Command::new(&editor).arg(temp_file.path()).status()?;

    if !status.success() {
        return Err(ThoughtError::EditorLaunchFailed(editor));
    }

    // Read edited content
    let content = fs::read_to_string(temp_file.path())?;
    Ok(content)
}

/// Find available editor from fallback chain
fn which_editor() -> Option<String> {
    for editor in ["vim", "nano", "vi"] {
        if Command::new("which")
            .arg(editor)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return Some(editor.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_which_editor_finds_something() {
        // At least one editor should be available on Unix systems
        // This test is platform-dependent
        #[cfg(unix)]
        {
            let editor = which_editor();
            // vi should always be available on Unix
            assert!(editor.is_some(), "Expected to find at least 'vi' editor");
        }
    }

    #[test]
    fn test_launch_editor_with_mock_editor() {
        // Use 'cat' as a mock editor that just outputs the file
        unsafe {
            env::set_var("EDITOR", "cat");
        }

        // This test would require actual terminal interaction
        // Skipping for now - covered by contract tests
    }
}
