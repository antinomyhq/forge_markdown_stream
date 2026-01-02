//! Streamdown - Streaming markdown renderer for terminal output.
//!
//! This crate provides a streaming markdown renderer optimized for LLM output.
//! It renders markdown character-by-character with configurable delays,
//! giving a smooth typewriter-like effect even when tokens arrive in bursts.
//!
//! # Example
//!
//! ```no_run
//! use streamdown::StreamdownRenderer;
//!
//! #[tokio::main]
//! async fn main() -> std::io::Result<()> {
//!     let mut renderer = StreamdownRenderer::new(80);
//!     
//!     // Push tokens as they arrive from LLM
//!     renderer.push("Hello ")?;
//!     renderer.push("**world**!\n")?;
//!     
//!     // Wait for all output to be written
//!     renderer.finish().await?;
//!     Ok(())
//! }
//! ```

mod code;
mod heading;
mod inline;
mod list;
mod renderer;
mod table;
mod text;
mod theme;
mod writer;

use std::io;

use streamdown_parser::Parser;

use renderer::Renderer;
pub use theme::{Style, Theme};
pub use writer::StreamingWriter;

/// Streaming markdown renderer for terminal output.
///
/// Buffers incoming tokens and renders complete lines with syntax highlighting,
/// styled headings, tables, lists, and more. Output is written character-by-character
/// with small delays for a smooth streaming effect.
pub struct StreamdownRenderer {
    parser: Parser,
    renderer: Renderer<StreamingWriter>,
    line_buffer: String,
}

impl StreamdownRenderer {
    /// Create a new renderer with the given terminal width.
    ///
    /// Uses a default character delay of 3ms for smooth output.
    pub fn new(width: usize) -> Self {
        Self::with_delay(width, 3)
    }

    /// Create a new renderer with custom character delay in milliseconds.
    pub fn with_delay(width: usize, delay_ms: u64) -> Self {
        Self {
            parser: Parser::new(),
            renderer: Renderer::new(StreamingWriter::new(delay_ms), width),
            line_buffer: String::new(),
        }
    }

    /// Create a new renderer with a custom theme.
    pub fn with_theme(width: usize, theme: Theme) -> Self {
        Self {
            parser: Parser::new(),
            renderer: Renderer::with_theme(StreamingWriter::new(3), width, theme),
            line_buffer: String::new(),
        }
    }

    /// Create a new renderer with custom theme and delay.
    pub fn with_theme_and_delay(width: usize, theme: Theme, delay_ms: u64) -> Self {
        Self {
            parser: Parser::new(),
            renderer: Renderer::with_theme(StreamingWriter::new(delay_ms), width, theme),
            line_buffer: String::new(),
        }
    }

    /// Push a token to the renderer.
    ///
    /// Tokens are buffered until a complete line is received, then rendered.
    /// This method returns immediately - actual output happens in a background task.
    pub fn push(&mut self, token: &str) -> io::Result<()> {
        self.line_buffer.push_str(token);

        while let Some(pos) = self.line_buffer.find('\n') {
            let line = self.line_buffer[..pos].to_string();
            for event in self.parser.parse_line(&line) {
                self.renderer.render_event(&event)?;
            }
            self.line_buffer = self.line_buffer[pos + 1..].to_string();
        }
        Ok(())
    }

    /// Finish rendering and wait for all output to be written.
    ///
    /// This flushes any remaining buffered content and waits for the
    /// background writer task to complete all pending output.
    pub async fn finish(mut self) -> io::Result<()> {
        if !self.line_buffer.is_empty() {
            for event in self.parser.parse_line(&self.line_buffer) {
                self.renderer.render_event(&event)?;
            }
        }
        for event in self.parser.finalize() {
            self.renderer.render_event(&event)?;
        }
        self.renderer.into_writer().finish().await
    }
}

/// Get the current terminal width, or a default of 80 columns.
pub fn terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80)
}
