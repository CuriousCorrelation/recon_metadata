use std::{error, fmt};

/// A list of database or search providers.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    /// GoogleBooks API at <https://developers.google.com/books/docs/v1/using>
    GoogleBooks,
    /// OpenLibrary API at <https://openlibrary.org/developers/api>
    OpenLibrary,
    /// TBD
    Goodreads,
    /// TBD
    Amazon,
}

#[derive(Debug)]
/// A wrapper around errors raised by libraries used in `recon_metadata`
pub enum ReconError {
    /// Message in case of unexpected API response.
    Message(String),
    /// A wrapper around [`serde_json::Error`]
    /// typically raised by `serde_json::from_str/value`
    JSONParse(serde_json::Error),
    /// A wrapper around [`reqwest::Error`]
    /// typically raised by `reqwest::get(url)`
    Connection(reqwest::Error),
    /// A wrapper around [`isbn2::IsbnError`]
    /// typically raised by `isbn2::Isbn::from_str(possible_isbn_str)`
    ISBNParse(isbn2::IsbnError),
    /// A wrapper around [`chrono::ParseError`]
    /// typically raised by `NaiveDate::parse_from_str(&string, &format_Str)`
    DateParse(chrono::ParseError),
    /// Missing field error
    MissingField(String),
}

impl fmt::Display for ReconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:#?}", self)
    }
}

impl error::Error for ReconError {}
