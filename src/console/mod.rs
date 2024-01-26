use std::io::Write;

/// clears `stdout`.
pub fn clear() -> Result<(), std::io::Error> {
    clear_into(&mut std::io::stdout())
}

/// clears a console `stream` by writing clear + reset ansi escapes into it.
pub fn clear_into(stream: &mut impl Write) -> Result<(), std::io::Error> {
    const CLEAR_CONSOLE: &[u8] = b"\x1b[2J\x1b[1;1H";

    stream.write_all(CLEAR_CONSOLE)
}
