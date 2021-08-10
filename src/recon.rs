use std::{collections::HashSet, error, fmt};

/// A list of database or search providers.
/// Search providers are API to provide search functionality.
/// This is the first API call in `recon_metadata`
/// that will populate [`Metadata`].
/// Additional data will be provided by [`Source`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Database {
    GoogleBooks,
    OpenLibrary,
    Goodreads,
    Amazon,
}

impl Default for Database {
    fn default() -> Self {
        Database::GoogleBooks
    }
}

/// A list of sources.
/// Sources are the book information providers
/// where `recon_metadata` parses search results
/// from [`Database`] and gathers additional
/// information on it via these souces.
#[derive(Debug)]
pub struct Source(pub(crate) HashSet<Database>);

impl Default for Source {
    fn default() -> Self {
        Source(HashSet::new())
    }
}

impl Source {
    pub fn new() -> Self {
        Source(HashSet::new())
    }

    pub fn all() -> Self {
        let mut source = HashSet::new();
        source.insert(Database::GoogleBooks);
        source.insert(Database::OpenLibrary);
        source.insert(Database::Goodreads);
        source.insert(Database::Amazon);

        Source(source)
    }

    pub fn source(mut self, database: Database) -> Self {
        self.0.insert(database);
        self
    }
}

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
