use std::{error, fmt};

/// A list of search providers.
/// Search providers are API to provide search functionality
/// This is the first API call in `recon_metadata`
#[derive(Debug)]
pub enum SearchProvider {
    GoogleBooks,
    OpenLibrary,
    Goodreads,
    Amazon,
}

impl Default for SearchProvider {
    fn default() -> Self {
        SearchProvider::GoogleBooks
    }
}

/// A list of sources.
/// Sources are the book information providers
/// where `recon_metadata` parses search results
/// from [`SearchProvider`] and gathers additional
/// information on it via these souces.
#[derive(Debug)]
pub enum Source {
    GoogleBooks,
    OpenLibrary,
    Goodreads,
    Amazon,
}

impl Default for Source {
    fn default() -> Self {
        Source::GoogleBooks
    }
}

impl Source {
    pub fn all() -> Vec<Self> {
        vec![
            Self::GoogleBooks,
            Self::OpenLibrary,
            Self::Goodreads,
            Self::Amazon,
        ]
    }

    pub fn add(sources: &mut Vec<Self>, source: Self) -> &Vec<Self> {
        sources.push(source);
        sources
    }
}

/// A config struct. Provides configuration information like sources and search providers.
#[derive(Debug)]
pub struct ReconSetup {
    pub(crate) search_provider: SearchProvider,
    pub(crate) sources:         Vec<Source>,
    pub(crate) extra:           bool,
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
