//! Demo of streaming LLM output with markdown rendering.

use std::io;
use streamdown::{terminal_width, StreamdownRenderer};

fn main() -> io::Result<()> {
    let content = include_str!("data.md");
    let tokens: Vec<&str> = content.split("<separator>").collect();
    let mut renderer = StreamdownRenderer::new(io::stdout(), terminal_width());

    for token in &tokens {
        renderer.push(token)?;
    }

    renderer.finish()?;

    Ok(())
}
