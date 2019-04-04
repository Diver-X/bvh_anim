#![allow(unused)]

//! Contains options for `bvh` file formatting.

use bstr::{BStr, BString, B};
use crate::Bvh;
use std::{
    fmt,
    io::{self, Write},
    iter,
    num::NonZeroUsize,
};

/// Specify formatting options for writing a `Bvh`.
#[derive(Clone, Default, Debug, Eq, Hash, PartialEq)]
pub struct WriteOptions {
    /// Which indentation style to use for nested bones.
    pub indent: IndentStyle,
    /// Which style new line terminator to use when writing the `bvh`.
    pub line_terminator: LineTerminator,
    #[doc(hidden)]
    _nonexhaustive: (),
}

impl WriteOptions {
    /// Create a new `WriteOptions` with default values.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Output the `Bvh` file to the `writer` with the given options.
    pub fn write<W: Write>(&self, bvh: &Bvh, writer: &mut W) -> io::Result<()> {
        let mut curr_line = BString::new();
        let mut curr_bytes_written = 0usize;
        let mut curr_string_len = 0usize;
        let mut iter_state = WriteOptionsIterState::new(bvh);

        while self.next_line(bvh, &mut curr_line, &mut iter_state) != false {
            let bytes: &[u8] = curr_line.as_ref();
            curr_string_len += bytes.len();
            curr_bytes_written += writer.write(bytes)?;

            if curr_bytes_written != curr_string_len {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Data has been dropped while writing to file",
                ));
            }
        }
        writer.flush()
    }

    /// Output the `Bvh` file to the `string` with the given options.
    pub fn write_to_string(&self, bvh: &Bvh) -> BString {
        let mut curr_line = BString::new();
        let mut out_string = BString::new();
        let mut iter_state = WriteOptionsIterState::new(bvh);

        while self.next_line(bvh, &mut curr_line, &mut iter_state) != false {
            out_string.push(&curr_line);
        }

        out_string
    }

    /// Sets `indent` on `self` to the new `IndentStyle`.
    #[inline]
    pub fn with_indent(self, indent: IndentStyle) -> Self {
        WriteOptions { indent, ..self }
    }

    /// Sets `line_terminator` on `self` to the new `LineTerminator`.
    #[inline]
    pub fn with_line_terminator(self, line_terminator: LineTerminator) -> Self {
        WriteOptions {
            line_terminator,
            ..self
        }
    }

    /// Get the next line of the written bvh file. This function is
    /// structured so that the `line` string can be continually
    /// re-used without allocating and de-allocating memory.
    ///
    /// # Returns
    ///
    /// Returns `true` when there are still more lines available,
    /// `false` when all lines have been extracted.
    fn next_line(
        &self,
        bvh: &Bvh,
        line: &mut BString,
        iter_state: &mut WriteOptionsIterState,
    ) -> bool {
        line.clear();
        false
    }
}

enum WriteOptionsIterState<'a> {
    WriteBones { bvh: &'a Bvh, curr_bone: usize },
    WriteMotion { bvh: &'a Bvh, curr_frame: usize },
}

impl<'a> WriteOptionsIterState<'a> {
    #[inline]
    fn new(bvh: &'a Bvh) -> Self {
        WriteOptionsIterState::WriteBones { bvh, curr_bone: 0 }
    }
}

/// Specify indentation style to use when writing the `Bvh` joints.
///
/// By default, this value is set to 1 tab.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum IndentStyle {
    /// Do not indent nested joints.
    NoIndentation,
    /// Use a single tab (`'\t'`) for indentation.
    Tabs,
    /// Use `n` spaces for indentation.
    Spaces(NonZeroUsize),
}

impl IndentStyle {
    /// Create a new `IndentStyle` with `n` preceeding spaces.
    ///
    /// If `n` is `0`, then `IndentStyle::NoIndentation` is returned.
    #[inline]
    pub fn with_spaces(n: usize) -> Self {
        NonZeroUsize::new(n)
            .map(IndentStyle::Spaces)
            .unwrap_or(IndentStyle::NoIndentation)
    }

    /// Return an `Iterator` which yields bytes corresponding to the ascii
    /// chars which form the `String` this indentation style would take.
    #[inline]
    fn prefix_chars(&self) -> impl Iterator<Item = u8> {
        match *self {
            IndentStyle::NoIndentation => iter::repeat(b'\0').take(0),
            IndentStyle::Tabs => iter::repeat(b'\t').take(1),
            IndentStyle::Spaces(n) => iter::repeat(b' ').take(n.get()),
        }
    }
}

/// Create a new `IndentStyle` using a single tab.
impl Default for IndentStyle {
    #[inline]
    fn default() -> Self {
        IndentStyle::Tabs
    }
}

/// Represents which line terminator style to use when writing a `Bvh` file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LineTerminator {
    /// Use Unix-style line endings (`'\n'`).
    Unix,
    /// Use Windows-style line endings (`'\r\n'`).
    Windows,
}

impl LineTerminator {
    /// Get the line terminator style native to the current OS:
    ///
    /// * On Windows, this returns `LineTerminator::Windows`.
    /// * Otherwise, this returns `LineTerminator::Unix`.
    #[cfg(target_os = "windows")]
    #[inline]
    pub fn native() -> Self {
        LineTerminator::Windows
    }

    /// Get the line terminator style native to the current OS:
    ///
    /// * On Windows, this returns `LineTerminator::Windows`.
    /// * Otherwise, this returns `LineTerminator::Unix`.
    #[cfg(not(target_os = "windows"))]
    #[inline]
    pub fn native() -> Self {
        LineTerminator::Unix
    }

    /// Return the characters of the `LineTerminator` as a `&str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        match *self {
            LineTerminator::Unix => "\n",
            LineTerminator::Windows => "\r\n",
        }
    }

    /// Return the characters of the `LineTerminator` as a `&BStr`.
    #[inline]
    pub fn as_bstr(&self) -> &BStr {
        B(self.as_str())
    }
}

/// Returns the native line terminator for the current OS.
impl Default for LineTerminator {
    #[inline]
    fn default() -> Self {
        LineTerminator::native()
    }
}

impl fmt::Display for LineTerminator {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
