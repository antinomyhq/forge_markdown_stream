//! Demo of streaming LLM output with markdown rendering.

use std::io;
use streamdown::{terminal_width, StreamdownRenderer};

#[tokio::main]
async fn main() -> io::Result<()> {
    let content = include_str!(
        "/Users/ranjit/Desktop/workspace/forge/plans/2025-04-02-system-context-rendering-final.md"
    );
    let tokens: Vec<&str> = content.split_inclusive(" ").collect();

    let mut renderer = StreamdownRenderer::new(terminal_width());

    for token in &tokens {
        renderer.push(token)?;
    }

    renderer.finish().await?;

    Ok(())
}
