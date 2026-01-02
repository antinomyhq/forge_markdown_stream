//! Streaming writer that outputs content character-by-character for smooth UX.
//!
//! When receiving tokens from an LLM with ~300ms latency, outputting full lines
//! at once creates a choppy experience. This writer outputs each character
//! individually with small delays, giving a smooth typewriter-like streaming effect.
//!
//! Uses a tokio task so writing never blocks the caller.

use std::io::{self, Write};
use std::time::Duration;

use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::task::JoinHandle;

/// Message sent to the background writer task.
enum WriterMsg {
    /// Write this content character-by-character
    Write(String),
    /// Flush and shutdown
    Shutdown,
}

/// A writer that outputs content character-by-character with small delays.
///
/// Uses a tokio background task to avoid blocking the caller.
/// Content is sent to the task which outputs it character-by-character
/// with configurable delays.
pub struct StreamingWriter {
    sender: UnboundedSender<WriterMsg>,
    handle: Option<JoinHandle<()>>,
    char_delay: Duration,
}

impl StreamingWriter {
    /// Create a new streaming writer that writes to stdout.
    pub fn new(ms: u64) -> Self {
        Self::with_delay(Duration::from_millis(ms))
    }

    /// Create a streaming writer with a custom character delay.
    pub fn with_delay(delay: Duration) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<WriterMsg>();

        let char_delay = delay;
        let handle = tokio::spawn(async move {
            let mut stdout = io::stdout();
            let mut first_char = true;

            while let Some(msg) = receiver.recv().await {
                match msg {
                    WriterMsg::Write(content) => {
                        for ch in content.chars() {
                            // Add delay before each character (except the very first)
                            if !first_char && !delay.is_zero() {
                                tokio::time::sleep(delay).await;
                            }
                            first_char = false;

                            // Write the character
                            let mut buf = [0u8; 4];
                            let encoded = ch.encode_utf8(&mut buf);
                            let _ = stdout.write_all(encoded.as_bytes());
                            let _ = stdout.flush();
                        }
                    }
                    WriterMsg::Shutdown => {
                        let _ = stdout.flush();
                        break;
                    }
                }
            }
        });

        Self {
            sender,
            handle: Some(handle),
            char_delay,
        }
    }

    /// Wait for all pending output to be written and shutdown.
    pub async fn finish(mut self) -> io::Result<()> {
        let _ = self.sender.send(WriterMsg::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
        Ok(())
    }

    /// Get the character delay.
    #[allow(dead_code)]
    pub fn char_delay(&self) -> Duration {
        self.char_delay
    }
}

impl Default for StreamingWriter {
    fn default() -> Self {
        Self::new(3)
    }
}

impl Write for StreamingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf).into_owned();
        self.sender
            .send(WriterMsg::Write(s))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "writer task gone"))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Flush happens automatically in the background task
        Ok(())
    }
}

impl Drop for StreamingWriter {
    fn drop(&mut self) {
        let _ = self.sender.send(WriterMsg::Shutdown);
        // Note: We can't await in drop, but the task will receive Shutdown
        // and exit gracefully. The handle is left to be cleaned up by tokio.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_writer_non_blocking() {
        // This test verifies that write() returns immediately
        let mut writer = StreamingWriter::with_delay(Duration::from_millis(100));

        let start = std::time::Instant::now();
        write!(writer, "hello").unwrap();
        let elapsed = start.elapsed();

        // Write should return almost immediately (not wait for output)
        assert!(
            elapsed < Duration::from_millis(50),
            "write() blocked for {:?}",
            elapsed
        );

        writer.finish().await.unwrap();
    }

    #[tokio::test]
    async fn test_finish_waits_for_output() {
        let writer = StreamingWriter::with_delay(Duration::from_millis(10));
        // finish() should wait for background task
        writer.finish().await.unwrap();
    }
}
