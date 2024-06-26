use alloc::{
    borrow::Cow,
    rc::Rc,
    string::{String, ToString},
};
use core::fmt;

#[derive(Debug, Default, Clone)]
pub enum Document {
    /// An empty document, rendered as an empty string
    #[default]
    Empty,
    /// A line break, rendered as a single '\n' char
    Newline,
    /// A single unicode character.
    ///
    /// NOTE: Certain `char` values are normalized to other [Document] variants, e.g. `\n` becomes
    /// a [Document::Newline], not a [Document::Char].
    Char(char, u32),
    /// A literal text string of width `n`
    Text(Cow<'static, str>, u32),
    /// A combinator which chooses the leftmost of each
    /// choice in the given document
    Flatten(Rc<Document>),
    /// Increase the indentation of the given document by `n`
    Indent(u32, Rc<Document>),
    /// Concatenate two documents
    Concat(Rc<Document>, Rc<Document>),
    /// Choose the more optimal of two documents depending on
    /// the amount of space remaining in the layout
    Choice(Rc<Document>, Rc<Document>),
}
impl Document {
    /// Returns true if this document has no content, i.e. [Document::Empty]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns true if the content of this document starts with a line break.
    ///
    /// This is primarily intended for use by the pretty printer itself, but may be useful to others.
    pub fn has_leading_newline(&self) -> bool {
        match self {
            Self::Empty => false,
            Self::Newline => true,
            Self::Char('\n' | '\r', _) => true,
            Self::Char(..) => false,
            Self::Text(ref text, _) => text.starts_with(['\n', '\r']),
            Self::Flatten(doc) => doc.has_leading_newline(),
            Self::Indent(_, doc) => doc.has_leading_newline(),
            Self::Concat(a, b) if a.is_empty() => b.has_leading_newline(),
            Self::Concat(a, _) => a.has_leading_newline(),
            // The choice should always have a single-line option, so we
            // have to return false here
            Self::Choice(..) => false,
        }
    }
}
impl From<char> for Document {
    #[inline(always)]
    fn from(c: char) -> Self {
        character(c)
    }
}
impl From<&'static str> for Document {
    #[inline(always)]
    fn from(s: &'static str) -> Self {
        const_text(s)
    }
}
impl From<String> for Document {
    #[inline(always)]
    fn from(s: String) -> Self {
        text(s)
    }
}

/// Render a line break (i.e. newline) in the output
pub fn nl() -> Document {
    Document::Newline
}

/// Display the given value using its [core::fmt::Display] implementation.
///
/// This function expects that the display format does not contain any newlines. Violating this
/// expectation may produce incorrect output.
pub fn display(s: impl ToString) -> Document {
    let string = Cow::<'static, str>::Owned(s.to_string());
    text(string)
}

/// Display the given character.
pub fn character(c: char) -> Document {
    match c {
        '\n' => Document::Newline,
        c => {
            let width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0) as u32;
            Document::Char(c, width)
        },
    }
}

/// Display the given string exactly.
///
/// Like [display], this function expects the string does not contain any newlines. Violating this
/// expectation may produce incorrect output.
pub fn text(s: impl ToString) -> Document {
    let string = Cow::<'static, str>::Owned(s.to_string());
    let mut chars = string.chars();
    match chars.next() {
        None => Document::Empty,
        Some(c) if chars.next().is_none() => character(c),
        Some(_) => {
            drop(chars);
            let width = unicode_width::UnicodeWidthStr::width(string.as_ref()) as u32;
            Document::Text(string, width)
        },
    }
}

/// Same as [text], but for static/constant strings
pub fn const_text(s: &'static str) -> Document {
    let mut chars = s.chars();
    match chars.next() {
        None => Document::Empty,
        Some(c) if chars.next().is_none() => character(c),
        Some(_) => {
            drop(chars);
            let string = Cow::Borrowed(s);
            let width = unicode_width::UnicodeWidthStr::width(string.as_ref()) as u32;
            Document::Text(string, width)
        },
    }
}

/// Create a document by splitting `input` on line breaks so ensure the invariants of [text] are upheld.
pub fn split<S: AsRef<str>>(input: S) -> Document {
    let input = input.as_ref();
    input
        .lines()
        .map(text)
        .reduce(|acc, doc| match acc {
            Document::Empty => doc + nl(),
            other => other + doc + nl(),
        })
        .unwrap_or(Document::Empty)
}

/// Concatenate two documents, producing a single document representing both.
#[inline(always)]
pub fn concat(left: Document, right: Document) -> Document {
    left + right
}

/// Use the leftmost option of every choice in the given document.
///
/// If the given document upholds the expectation that none of the
/// leftmost choices contain newlines, then this combinator has the
/// effect of displaying all choices on one line.
pub fn flatten(doc: Document) -> Document {
    if doc.is_empty() {
        return doc;
    }
    Document::Flatten(Rc::new(doc))
}

/// Increase the indentation level of the given document by `width`.
///
/// The indentation level determines the number of spaces put after newlines.
///
/// NOTE: Indentation is applied following newlines, therefore, the first
/// line of a document is _not_ indented.
pub fn indent(indent: u32, doc: Document) -> Document {
    if doc.is_empty() {
        return doc;
    }
    Document::Indent(indent, Rc::new(doc))
}

impl core::ops::Add for Document {
    type Output = Document;

    /// Concatenate the two documents
    fn add(self: Document, other: Document) -> Self::Output {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        Document::Concat(Rc::new(self), Rc::new(other))
    }
}

impl core::ops::Add<char> for Document {
    type Output = Document;

    /// Concatenate the two documents
    fn add(self: Document, other: char) -> Self::Output {
        let other = character(other);
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        Document::Concat(Rc::new(self), Rc::new(other))
    }
}

impl core::ops::Add<Document> for char {
    type Output = Document;

    /// Concatenate the two documents
    fn add(self: char, other: Document) -> Self::Output {
        let lhs = character(self);
        if lhs.is_empty() {
            return other;
        }
        if other.is_empty() {
            return lhs;
        }
        Document::Concat(Rc::new(lhs), Rc::new(other))
    }
}

impl core::ops::Add<&'static str> for Document {
    type Output = Document;

    /// Concatenate the two documents
    fn add(self: Document, other: &'static str) -> Self::Output {
        let other = const_text(other);
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        Document::Concat(Rc::new(self), Rc::new(other))
    }
}

impl core::ops::Add<Document> for &'static str {
    type Output = Document;

    /// Concatenate the two documents
    fn add(self: &'static str, other: Document) -> Self::Output {
        let lhs = const_text(self);
        if lhs.is_empty() {
            return other;
        }
        if other.is_empty() {
            return lhs;
        }
        Document::Concat(Rc::new(lhs), Rc::new(other))
    }
}

impl core::ops::AddAssign for Document {
    /// Append `rhs` to `self`
    fn add_assign(&mut self, rhs: Document) {
        if rhs.is_empty() {
            return;
        }
        if self.is_empty() {
            *self = rhs;
            return;
        }
        let lhs = core::mem::take(self);
        *self = Document::Concat(Rc::new(lhs), Rc::new(rhs));
    }
}

impl core::ops::AddAssign<char> for Document {
    /// Append `rhs` to `self`
    fn add_assign(&mut self, rhs: char) {
        let rhs = character(rhs);
        if rhs.is_empty() {
            return;
        }
        if self.is_empty() {
            *self = rhs;
            return;
        }
        let lhs = core::mem::take(self);
        *self = Document::Concat(Rc::new(lhs), Rc::new(rhs));
    }
}

impl core::ops::AddAssign<&'static str> for Document {
    /// Append `rhs` to `self`
    fn add_assign(&mut self, rhs: &'static str) {
        let rhs = const_text(rhs);
        if rhs.is_empty() {
            return;
        }
        if self.is_empty() {
            *self = rhs;
            return;
        }
        let lhs = core::mem::take(self);
        *self = Document::Concat(Rc::new(lhs), Rc::new(rhs));
    }
}

impl core::ops::BitOr for Document {
    type Output = Document;

    /// If inside a `flat`, _or_ the first line of the left document fits within
    /// the required width, then display the left document. Otherwise, display
    /// the right document.
    fn bitor(self: Document, other: Document) -> Self::Output {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        Document::Choice(Rc::new(self), Rc::new(other))
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use core::fmt::Write;
        match self {
            Self::Empty => Ok(()),
            Self::Newline => f.write_char('\n'),
            Self::Char(c, _) => f.write_char(*c),
            doc => {
                let width = f.width().unwrap_or(80);
                super::print::pretty_print(doc, width, f)
            },
        }
    }
}
