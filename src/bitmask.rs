/// Letter bitmask type (far outstrips HashSet<char> in terms of performance).
pub type LetterBitmask = u64;

/// Trait extension to allow easy computation of letter bitmasks.
pub trait CanRepresentLetterBitmask {
    fn char_bitmask(ch: &char) -> Self;

    fn compute_bitmask<'a, T>(word: T) -> Self
    where
        T: Iterator<Item = &'a char>;
}

impl CanRepresentLetterBitmask for LetterBitmask {
    #[inline(always)]
    fn char_bitmask(ch: &char) -> Self {
        1_u64 << (ch.to_ascii_lowercase() as u8 - 'a' as u8)
    }

    fn compute_bitmask<'a, T>(word: T) -> Self
    where
        T: Iterator<Item = &'a char>,
    {
        let mut output = 0_u64;
        for ch in word {
            output |= LetterBitmask::char_bitmask(ch);
        }

        output
    }
}

/// Trait extension to enable easy conversion of Vec<char> to letter bitmasks.
pub trait CanConvertToLetterBitmask {
    fn to_letter_bitmask(&self) -> LetterBitmask;
}

impl CanConvertToLetterBitmask for Vec<char> {
    fn to_letter_bitmask(&self) -> LetterBitmask {
        LetterBitmask::compute_bitmask(self.iter())
    }
}
