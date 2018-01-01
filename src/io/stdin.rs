use std::io::Read;


/// reads stdin until it encounters a new line, consuming it.
pub fn read_enter_key() -> Result<(), std::io::Error> {
    let mut stream = std::io::stdin();
    let character = &mut [0u8];

    loop {
        match stream.read(character)? {
            1 if character[0] == b'\n' => return Ok(()),
            0 => continue,
            1 => continue,
            _ => panic!["invariant: read of one byte cannot exceed one byte."],
        }
    }
}
