use crate::recon::Source;
use crate::{
    recon::{Database, ReconError},
    source::google_books::GoogleBooks,
    source::open_library::OpenLibrary,
};
use chrono::NaiveDate;
use isbn2::{Isbn, Isbn10, Isbn13};
use std::collections::HashSet;
use std::ops::Add;

/// [`Metadata`] type contains information uniquely identify a book.
/// [`Metadata`] contains the following:
/// 1. ISBN-10 and/or ISBN-13
/// 2. Title(s)
/// 3. Author(s) [Can be "unknown"]
/// 4. Description
/// 5. Number of pages
///
/// [`Metadata`] can also fetch some additional information like:
/// 1. Publisher(s)
/// 2. Publication Date(s)
/// 3. Language
/// 4. Cover image
#[derive(Debug, Default)]
pub struct Metadata {
    pub(crate) isbn10s:           HashSet<Isbn10>,
    pub(crate) isbn13s:           HashSet<Isbn13>,
    pub(crate) titles:            HashSet<String>,
    pub(crate) authors:           HashSet<String>,
    pub(crate) descriptions:      HashSet<String>,
    pub(crate) page_count:        HashSet<u16>,
    pub(crate) publishers:        HashSet<String>,
    pub(crate) publication_dates: HashSet<NaiveDate>,
    pub(crate) languages:         HashSet<String>,
    pub(crate) tags:              HashSet<String>,
    pub(crate) cover_images:      HashSet<String>,
}

impl Add for Metadata {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        self.isbn10s.extend(other.isbn10s);
        self.isbn13s.extend(other.isbn13s);
        self.titles.extend(other.titles);
        self.authors.extend(other.authors);
        self.descriptions.extend(other.descriptions);
        self.page_count.extend(other.page_count);
        self.publishers.extend(other.publishers);
        self.publication_dates.extend(other.publication_dates);
        self.languages.extend(other.languages);
        self.tags.extend(other.tags);
        self.cover_images.extend(other.cover_images);

        self
    }
}

/// A type synonym for `Result<Vec<Metadata>, ReconError>`
pub type ReconResult = Result<Metadata, ReconError>;

impl Metadata {
    async fn ask_database(database: &Database, isbn: &Isbn) -> ReconResult {
        match database {
            Database::GoogleBooks => GoogleBooks::from_isbn(isbn).await,
            Database::OpenLibrary => OpenLibrary::from_isbn(isbn).await,
            Database::Amazon => unimplemented!(),
            Database::Goodreads => unimplemented!(),
        }
    }

    /// A simple `Isbn` search that'll use only one provider defined by
    /// [`ReconSetup`].
    /// An eclectic / diversified / exaustive search that'll use search provider
    /// for initial information and fill in the blacks making expensive calls
    /// but returning almost complete information about the book
    /// provides by the sources defined by [`Source`].
    pub async fn from_isbn(source: &Source, isbn: &Isbn) -> ReconResult {
        let database_list: &HashSet<Database> = &source.0;

        let mut metadata = Metadata::default();

        for database in database_list {
            metadata = metadata + Self::ask_database(database, isbn).await?;
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod test {
    use log::debug;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn parses_from_isbn() {
        use super::Metadata;
        use crate::metadata::ReconResult;
        use crate::recon::Database;
        use crate::recon::Source;
        use isbn2::Isbn;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();

        let source = Source::new()
            .source(Database::GoogleBooks)
            .source(Database::OpenLibrary);

        let res: ReconResult = Metadata::from_isbn(&source, &isbn).await;

        debug!("Response: {:#?}", res);
        assert!(res.is_ok());
    }
}
