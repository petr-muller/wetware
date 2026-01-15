//! Color mode configuration for styled output
//!
//! This module provides the [`ColorMode`] enum for controlling color output behavior
//! in CLI commands. It supports automatic terminal detection and explicit overrides.

use clap::ValueEnum;
use std::io::IsTerminal;

/// Controls color output behavior for styled entity rendering.
///
/// The color mode determines when ANSI color codes should be emitted in output.
/// It supports automatic detection based on terminal type, or explicit overrides.
///
/// # Examples
///
/// ```
/// use wetware::services::color_mode::ColorMode;
///
/// let mode = ColorMode::Auto;
/// // In a real terminal, this would return true
/// // When piped, this returns false
/// let use_colors = mode.should_use_colors();
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ColorMode {
    /// Always use colors regardless of terminal detection.
    ///
    /// Use this to force colored output even when piping to another command
    /// or redirecting to a file (e.g., for CI logs that support ANSI codes).
    Always,

    /// Automatically detect terminal and use colors when appropriate.
    ///
    /// This is the default behavior. Colors are enabled when stdout is a TTY
    /// (interactive terminal) and disabled when output is piped or redirected.
    #[default]
    Auto,

    /// Never use colors.
    ///
    /// Use this to ensure plain text output even in an interactive terminal.
    Never,
}

impl ColorMode {
    /// Determine if colors should be used based on mode and terminal state.
    ///
    /// # Returns
    ///
    /// - `true` if ANSI color codes should be emitted
    /// - `false` if output should be plain text
    ///
    /// # Behavior by mode
    ///
    /// | ColorMode | TTY Detected | Result |
    /// |-----------|--------------|--------|
    /// | Always    | Yes          | true   |
    /// | Always    | No           | true   |
    /// | Auto      | Yes          | true   |
    /// | Auto      | No           | false  |
    /// | Never     | Yes          | false  |
    /// | Never     | No           | false  |
    pub fn should_use_colors(&self) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => std::io::stdout().is_terminal(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode_always_returns_true() {
        assert!(ColorMode::Always.should_use_colors());
    }

    #[test]
    fn test_color_mode_never_returns_false() {
        assert!(!ColorMode::Never.should_use_colors());
    }

    #[test]
    fn test_color_mode_default_is_auto() {
        assert_eq!(ColorMode::default(), ColorMode::Auto);
    }

    #[test]
    fn test_color_mode_auto_depends_on_tty() {
        // In test environment, stdout is typically not a TTY
        // This test verifies the Auto mode actually checks terminal status
        let result = ColorMode::Auto.should_use_colors();
        let is_tty = std::io::stdout().is_terminal();
        assert_eq!(result, is_tty);
    }

    #[test]
    fn test_color_mode_clone_and_copy() {
        let mode = ColorMode::Always;
        #[allow(clippy::clone_on_copy)]
        let cloned = mode.clone();
        let copied = mode;
        assert_eq!(mode, cloned);
        assert_eq!(mode, copied);
    }

    #[test]
    fn test_color_mode_equality() {
        assert_eq!(ColorMode::Always, ColorMode::Always);
        assert_eq!(ColorMode::Auto, ColorMode::Auto);
        assert_eq!(ColorMode::Never, ColorMode::Never);
        assert_ne!(ColorMode::Always, ColorMode::Never);
        assert_ne!(ColorMode::Auto, ColorMode::Always);
    }
}
