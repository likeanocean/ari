/// extension methods for ascii-subset only operations.
///
/// be aware that operations on seemingly non-ascii characters can sometimes
/// have unexpected results. consider this example:
///
/// ```
/// # use ari::str::AsciiExt;
///
/// assert_eq!(AsciiExt::to_ascii_uppercase("café"), "CAFÉ");
/// assert_eq!(AsciiExt::to_ascii_uppercase("café"), "CAFé");
/// ```
///
/// in the first example, the lowercased string is represented `"cafe\u{301}"`
/// (the last character is an acute accent [combining character]). unlike the
/// other characters in the string, the combining character will not get mapped
/// to an uppercase variant, resulting in `"CAFE\u{301}"`. in the second
/// example, the lowercased string is represented `"caf\u{e9}"` (the last
/// character is a single unicode character representing an 'e' with an acute
/// accent). since the last character is defined outside the scope of ascii,
/// it will not get mapped to an uppercase variant, resulting in `"CAF\u{e9}"`.
///
/// [combining character]: https://en.wikipedia.org/wiki/combining_character
#[rustfmt::skip]
pub trait AsciiExt {
    type Owned;

    fn is_ascii(&self) -> bool;
    fn to_ascii_uppercase(&self) -> Self::Owned;
    fn to_ascii_lowercase(&self) -> Self::Owned;
    fn eq_ignore_ascii_case(&self, other: &Self) -> bool;
    fn make_ascii_uppercase(&mut self);
    fn make_ascii_lowercase(&mut self);
    fn is_ascii_alphabetic(&self) -> bool { unimplemented!(); }
    fn is_ascii_uppercase(&self) -> bool { unimplemented!(); }
    fn is_ascii_lowercase(&self) -> bool { unimplemented!(); }
    fn is_ascii_alphanumeric(&self) -> bool { unimplemented!(); }
    fn is_ascii_digit(&self) -> bool { unimplemented!(); }
    fn is_ascii_hexdigit(&self) -> bool { unimplemented!(); }
    fn is_ascii_punctuation(&self) -> bool { unimplemented!(); }
    fn is_ascii_graphic(&self) -> bool { unimplemented!(); }
    fn is_ascii_whitespace(&self) -> bool { unimplemented!(); }
    fn is_ascii_control(&self) -> bool { unimplemented!(); }
}

macro_rules! delegate_ascii_methods {
    () => {
        #[inline] fn is_ascii             (&self) -> bool { self.is_ascii() }
        #[inline] fn to_ascii_uppercase   (&self) -> Self::Owned { self.to_ascii_uppercase() }
        #[inline] fn to_ascii_lowercase   (&self) -> Self::Owned { self.to_ascii_lowercase() }
        #[inline] fn eq_ignore_ascii_case (&self, other: &Self) -> bool { self.eq_ignore_ascii_case(other) }
        #[inline] fn make_ascii_uppercase (&mut self) { self.make_ascii_uppercase(); }
        #[inline] fn make_ascii_lowercase (&mut self) { self.make_ascii_lowercase(); }
    }
}

macro_rules! delegate_ascii_ctype_methods {
    () => {
        #[inline] fn is_ascii_alphabetic  (&self) -> bool { self.is_ascii_alphabetic() }
        #[inline] fn is_ascii_uppercase   (&self) -> bool { self.is_ascii_uppercase() }
        #[inline] fn is_ascii_lowercase   (&self) -> bool { self.is_ascii_lowercase() }
        #[inline] fn is_ascii_alphanumeric(&self) -> bool { self.is_ascii_alphanumeric() }
        #[inline] fn is_ascii_digit       (&self) -> bool { self.is_ascii_digit() }
        #[inline] fn is_ascii_hexdigit    (&self) -> bool { self.is_ascii_hexdigit() }
        #[inline] fn is_ascii_punctuation (&self) -> bool { self.is_ascii_punctuation() }
        #[inline] fn is_ascii_graphic     (&self) -> bool { self.is_ascii_graphic() }
        #[inline] fn is_ascii_whitespace  (&self) -> bool { self.is_ascii_whitespace() }
        #[inline] fn is_ascii_control     (&self) -> bool { self.is_ascii_control() }
    }
}

impl AsciiExt for u8 {
    type Owned = u8;

    delegate_ascii_methods!();
    delegate_ascii_ctype_methods!();
}

impl AsciiExt for char {
    type Owned = char;

    delegate_ascii_methods!();
    delegate_ascii_ctype_methods!();
}

impl AsciiExt for [u8] {
    type Owned = Vec<u8>;

    delegate_ascii_methods!();

    #[inline]
    fn is_ascii_alphabetic(&self) -> bool {
        self.iter().all(|b| b.is_ascii_alphabetic())
    }

    #[inline]
    fn is_ascii_uppercase(&self) -> bool {
        self.iter().all(|b| b.is_ascii_uppercase())
    }

    #[inline]
    fn is_ascii_lowercase(&self) -> bool {
        self.iter().all(|b| b.is_ascii_lowercase())
    }

    #[inline]
    fn is_ascii_alphanumeric(&self) -> bool {
        self.iter().all(|b| b.is_ascii_alphanumeric())
    }

    #[inline]
    fn is_ascii_digit(&self) -> bool {
        self.iter().all(|b| b.is_ascii_digit())
    }

    #[inline]
    fn is_ascii_hexdigit(&self) -> bool {
        self.iter().all(|b| b.is_ascii_hexdigit())
    }

    #[inline]
    fn is_ascii_punctuation(&self) -> bool {
        self.iter().all(|b| b.is_ascii_punctuation())
    }

    #[inline]
    fn is_ascii_graphic(&self) -> bool {
        self.iter().all(|b| b.is_ascii_graphic())
    }

    #[inline]
    fn is_ascii_whitespace(&self) -> bool {
        self.iter().all(|b| b.is_ascii_whitespace())
    }

    #[inline]
    fn is_ascii_control(&self) -> bool {
        self.iter().all(|b| b.is_ascii_control())
    }
}

impl AsciiExt for str {
    type Owned = String;

    delegate_ascii_methods!();

    #[inline]
    fn is_ascii_alphabetic(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_alphabetic())
    }

    #[inline]
    fn is_ascii_uppercase(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_uppercase())
    }

    #[inline]
    fn is_ascii_lowercase(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_lowercase())
    }

    #[inline]
    fn is_ascii_alphanumeric(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_alphanumeric())
    }

    #[inline]
    fn is_ascii_digit(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_digit())
    }

    #[inline]
    fn is_ascii_hexdigit(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_hexdigit())
    }

    #[inline]
    fn is_ascii_punctuation(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_punctuation())
    }

    #[inline]
    fn is_ascii_graphic(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_graphic())
    }

    #[inline]
    fn is_ascii_whitespace(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_whitespace())
    }

    #[inline]
    fn is_ascii_control(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_control())
    }
}
