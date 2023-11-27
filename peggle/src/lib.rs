/// Keeps track of the current parse location of a string input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index<'a> {
    /// The remaining data
    pub remaining: &'a str,
    pub lineno: usize,
    pub colno: usize,
}

impl<'a> Index<'a> {
    #[inline]
    pub fn new(string: &'a str) -> Self {
        Self {
            remaining: string,
            lineno: 0,
            colno: 0,
        }
    }

    #[inline]
    pub fn peek(&self) -> Option<char> {
        self.remaining.chars().next()
    }

    #[inline]
    pub fn peek_multiple<const N: usize>(&self) -> Option<[char; N]> {
        let mut peeked = ['\0'; N];

        let mut chars = self.remaining.chars();
        for p in peeked.iter_mut() {
            let Some(c) = chars.next() else {
                return None
            };
            *p = c;
        }

        Some(peeked)
    }

    #[inline]
    pub fn next_multiple<const N: usize>(&mut self) -> Option<[char; N]> {
        let mut all_next = ['\0'; N];
        for n in all_next.iter_mut() {
            let Some(c) = self.next() else {
                return None
            };
            *n = c;
        }
        Some(all_next)
    }

    #[inline]
    pub fn advance_to_end(&mut self) {
        while let Some(_) = self.next() {}
    }
}

impl<'a> Iterator for Index<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(c) = self.remaining.chars().next() else {
            return None
        };

        if c == '\n' {
            self.lineno += 1;
            self.colno = 0;
        } else {
            self.colno += c.len_utf8();
        }

        self.remaining = &self.remaining[c.len_utf8()..];
        Some(c)
    }
}

/// Represents an error that occurred during the parsing of a string input.
#[derive(Debug)]
pub struct ParseError {
    pub lineno: usize,
    pub colno: usize,
}

impl ParseError {
    /// Crates an error containing the given index's line/column information.
    pub fn from_index(idx: Index<'_>) -> Self {
        Self {
            lineno: idx.lineno,
            colno: idx.colno,
        }
    }
}

pub trait Parse: Sized {
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError>;

    #[inline]
    fn parse_raw_at<'a>(index: Index<'a>) -> Result<(&'a str, Index<'a>), ParseError> {
        let (_, new_index) = Self::parse_at(index)?;
        let bytes_taken = index.remaining.len() - new_index.remaining.len();

        Ok((&index.remaining[..bytes_taken], new_index))
    }

    #[inline]
    fn parse(input: &str) -> Result<Self, ParseError> {
        let idx = Index {
            remaining: input,
            lineno: 0,
            colno: 0,
        };

        let (ret, remaining) = Self::parse_at(idx)?;
        if remaining.remaining.is_empty() {
            Ok(ret)
        } else {
            Err(ParseError::from_index(idx))
        }
    }
}

impl Parse for bool {
    #[inline]
    fn parse_at<'a>(mut index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        match index.next_multiple() {
            Some(['t', 'r', 'u', 'e']) => Ok((true, index)),
            Some(['f', 'a', 'l', 's']) if index.next() == Some('e') => Ok((true, index)),
            _ => Err(ParseError::from_index(index)),
        }
    }
}

impl Parse for u8 {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_unsigned(index)
    }
}

impl Parse for u16 {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_unsigned(index)
    }
}

impl Parse for u32 {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_unsigned(index)
    }
}

impl Parse for u64 {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_unsigned(index)
    }
}

impl Parse for u128 {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_unsigned(index)
    }
}

impl Parse for usize {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_unsigned(index)
    }
}

impl Parse for i8 {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_signed(index)
    }
}

impl Parse for i16 {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_signed(index)
    }
}

impl Parse for i32 {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_signed(index)
    }
}

impl Parse for i64 {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_signed(index)
    }
}

impl Parse for i128 {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_signed(index)
    }
}

impl Parse for isize {
    #[inline]
    fn parse_at<'a>(index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        parse_signed(index)
    }
}

impl Parse for String {
    #[inline]
    fn parse_at<'a>(mut index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        let s = index.remaining.to_string();
        index.advance_to_end();
        Ok((s, index))
    }
}

impl Parse for char {
    #[inline]
    fn parse_at<'a>(mut index: Index<'a>) -> Result<(Self, Index<'a>), ParseError> {
        index
            .next()
            .map(|c| (c, index))
            .ok_or(ParseError::from_index(index))
    }
}

fn parse_unsigned<'a, I: num::Unsigned + num::CheckedAdd + num::CheckedMul + From<u8>>(
    mut index: Index<'a>,
) -> Result<(I, Index<'a>), ParseError> {
    let mut value = None;

    // Edge case: 0 (no leading zeros are allowed)
    if let Some('0') = index.peek() {
        index.next();
        return match index.peek() {
            Some('0'..='9') => Err(ParseError::from_index(index)),
            _ => Ok((I::zero(), index)),
        };
    }

    // Match characters 0-9, checking for overflow
    while let Some(c @ '0'..='9') = index.peek() {
        let c_val = I::from((c as u8) - ('0' as u8));
        value = value
            .unwrap_or(I::zero())
            .checked_mul(&10u8.into())
            .and_then(|i| i.checked_add(&c_val));
        if value.is_none() {
            // Next digit would lead to overflow
            return Err(ParseError::from_index(index));
        }

        index.next();
    }

    // If value == None then no digits were detected--return an error
    value
        .ok_or(ParseError::from_index(index))
        .map(|i| (i, index))
}

fn parse_signed<
    'a,
    I: num::Signed + num::CheckedAdd + num::CheckedSub + num::CheckedMul + From<i8>,
>(
    mut index: Index<'a>,
) -> Result<(I, Index<'a>), ParseError> {
    let mut value = None;

    let mut is_negative = false;

    if let Some('-') = index.peek() {
        is_negative = true;
        index.next();
    }

    // Edge case: 0 (no leading zeros are allowed)
    if let Some('0') = index.peek() {
        index.next();
        return match index.peek() {
            Some('0'..='9') => Err(ParseError::from_index(index)),
            _ => Ok((I::zero(), index)),
        };
    }

    // Match characters 0-9, checking for overflow
    while let Some(c @ '0'..='9') = index.peek() {
        let c_val = I::from((c as i8) - ('0' as i8));
        value = value.unwrap_or(I::zero()).checked_mul(&10i8.into());

        value = if is_negative {
            value.and_then(|i| i.checked_sub(&c_val))
        } else {
            value.and_then(|i| i.checked_add(&c_val))
        };

        if value.is_none() {
            // Next digit would lead to overflow
            return Err(ParseError::from_index(index));
        }

        index.next();
    }

    // If value == None then no digits were detected--return an error
    value
        .ok_or(ParseError::from_index(index))
        .map(|i| (i, index))
}
