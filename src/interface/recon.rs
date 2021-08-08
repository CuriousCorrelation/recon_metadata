use std::{error, fmt};

#[derive(Debug)]
/// A wrapper around errors raised by parsing libraries used in `book_recon_metadata`
pub enum ReconError {
    /// Message in case of unexpected API response.
    Message(String),
    /// A wrapper around [`serde_json::Error`]
    /// typically raised by `serde_json::from_str/value`
    JSONParse(serde_json::Error),
    /// A wrapper around [`reqwest::Error`]
    /// typically raised by `reqwest::get(url)`
    Connection(reqwest::Error),
    /// A wrapper around [`isbn::IsbnError`]
    /// typically raised by `isbn::Isbn::from_str(possible_isbn_str)`
    ISBNParse(isbn::IsbnError),
    /// A wrapper around [`chrono::ParseError`]
    /// typically raised by `NaiveDate::parse_from_str(&string, &format_Str)`
    DateParse(chrono::ParseError),
}

impl fmt::Display for ReconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:#?}", self)
    }
}

impl error::Error for ReconError {}
