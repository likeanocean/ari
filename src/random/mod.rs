use rand::seq::SliceRandom;
use rand::Rng;

const LOWERCASE_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const LOWERCASE_ALPHNUMERIC: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

// todo: remove `TRandom: 'a` when lifetime reform is implemented.
pub struct RandomCharacter<'a, TRandom: 'a> {
    random: &'a mut TRandom,
    alphabet: &'a [char],
}

impl<TRandom> Iterator for RandomCharacter<'_, TRandom>
where
    TRandom: Rng,
{
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.alphabet.choose(&mut self.random).map(|x| *x as char)
    }
}

pub struct RandomAlphabet<'a, TRandom: 'a> {
    random: &'a mut TRandom,
}

impl<TRandom> Iterator for RandomAlphabet<'_, TRandom>
where
    TRandom: Rng,
{
    type Item = char;

    fn next(&mut self) -> Option<char> {
        LOWERCASE_ALPHABET
            .choose(&mut self.random)
            .map(|x| *x as char)
    }
}

pub struct RandomAlphanumeric<'a, TRandom: 'a> {
    random: &'a mut TRandom,
}

impl<TRandom> Iterator for RandomAlphanumeric<'_, TRandom>
where
    TRandom: Rng,
{
    type Item = char;

    fn next(&mut self) -> Option<char> {
        LOWERCASE_ALPHNUMERIC
            .choose(&mut self.random)
            .map(|x| *x as char)
    }
}

pub fn string(alphabet: &[char], length: usize) -> String {
    RandomCharacter {
        alphabet: alphabet,
        random: &mut rand::thread_rng(),
    }
    .take(length)
    .collect()
}

pub fn alpha_string(length: usize) -> String {
    RandomAlphabet {
        random: &mut rand::thread_rng(),
    }
    .take(length)
    .collect()
}

pub fn alphanumeric_string(length: usize) -> String {
    RandomAlphanumeric {
        random: &mut rand::thread_rng(),
    }
    .take(length)
    .collect()
}

pub fn vec(length: usize) -> Vec<u8> {
    let mut buffer = vec![0; length];
    let mut random = rand::thread_rng();

    random.fill(&mut buffer[..]);
    buffer
}

// todo: use const generics.

pub fn array_16() -> [u8; 16] {
    rand::random::<[u8; 16]>()
}

pub fn array_32() -> [u8; 32] {
    rand::random::<[u8; 32]>()
}
