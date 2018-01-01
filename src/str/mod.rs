use std::string::FromUtf16Error;
use unicode_width::UnicodeWidthStr;


#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum TextAlignment {
    Left,
    Right,
}

pub trait PadString {
    fn pad_left(&self, width: usize) -> String {
        self.pad(width, ' ', TextAlignment::Right)
    }

    fn pad_right(&self, width: usize) -> String {
        self.pad(width, ' ', TextAlignment::Left)
    }

    fn pad_left_with(&self, width: usize, character: char) -> String {
        self.pad(width, character, TextAlignment::Right)
    }

    fn pad_right_with(&self, width: usize, character: char) -> String {
        self.pad(width, character, TextAlignment::Left)
    }

    fn pad_to_width_with_alignment(&self, width: usize, alignment: TextAlignment) -> String {
        self.pad(width, ' ', alignment)
    }

    fn pad(&self, width: usize, character: char, alignment: TextAlignment) -> String;
}

impl PadString for str {
    fn pad(&self, width: usize, character: char, alignment: TextAlignment) -> String {
        let display_columns = UnicodeWidthStr::width(self);

        if display_columns >= width {
            self.to_string()
        } else {
            let required = width - display_columns;
            let mut string = String::with_capacity(self.len() + required);
            let (left, right) = match alignment {
                TextAlignment::Left => (0, required),
                TextAlignment::Right => (required, 0),
            };

            (0..left).for_each(|_| string.push(character));
            string.push_str(self);
            (0..right).for_each(|_| string.push(character));

            string
        }
    }
}


pub fn to_utf16(string: &str) -> Vec<u16> {
    string.encode_utf16().collect()
}

pub fn to_utf16_null(string: &str) -> Vec<u16> {
    string.encode_utf16().chain(Some(0)).collect()
}


pub fn from_utf16(data: &[u16]) -> Result<String, FromUtf16Error> {
    String::from_utf16(data)
}

pub fn from_utf16_null(data: &[u16]) -> Result<String, FromUtf16Error> {
    let data = utf16_extent(data);

    String::from_utf16(data)
}


pub fn from_utf16_lossy(data: &[u16]) -> String {
    String::from_utf16_lossy(data)
}

pub fn from_utf16_lossy_null(data: &[u16]) -> String {
    let data = utf16_extent(data);

    String::from_utf16_lossy(data)
}


fn utf16_extent(data: &[u16]) -> &[u16] {
    let length = data.iter().position(|x| *x == 0).unwrap_or(data.len());

    &data[..length]
}
