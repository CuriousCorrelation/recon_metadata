use crate::recon::Source;
use crate::types::base;
use crate::{
    recon::{Database, ReconError},
    sources::google_books::GoogleBooks,
    sources::open_library::OpenLibrary,
};
use isbn::Isbn;
use log::debug;
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
    pub(crate) isbns:             base::ISBNs,
    pub(crate) titles:            base::Titles,
    pub(crate) authors:           base::Authors,
    pub(crate) descriptions:      base::Descriptions,
    pub(crate) page_count:        base::PageCount,
    pub(crate) publishers:        base::Publishers,
    pub(crate) publication_dates: base::PublicationDates,
    pub(crate) languages:         base::Languages,
    pub(crate) tags:              base::Tags,
    pub(crate) cover_images:      base::CoverImages,
}

impl Add for Metadata {
    type Output = Self;

    fn add(mut self, mut other: Self) -> Self {
        self.isbns.append(&mut other.isbns);
        self.titles.append(&mut other.titles);
        self.authors.append(&mut other.authors);
        self.descriptions.append(&mut other.descriptions);
        self.page_count.append(&mut other.page_count);
        self.publishers.append(&mut other.publishers);
        self.publication_dates.append(&mut other.publication_dates);
        self.languages.append(&mut other.languages);
        self.tags.append(&mut other.tags);
        self.cover_images.append(&mut other.cover_images);

        self
    }
}

/// A type synonym for `Result<Vec<Metadata>, ReconError>`
pub type ReconResult = Result<Metadata, ReconError>;

impl Metadata {
    pub fn isbns(mut self, isbns: base::ISBNs) -> Self {
        self.isbns = isbns;
        debug!("Field `isbns` is set to: {:#?}", self.isbns);
        self
    }

    pub fn titles(mut self, titles: base::Titles) -> Self {
        self.titles = titles;
        debug!("Field `titles` is set to: {:#?}", self.titles);
        self
    }

    pub fn authors(mut self, authors: base::Authors) -> Self {
        self.authors = authors;
        debug!("Field `authors` is set to: {:#?}", self.authors);
        self
    }

    pub fn descriptions(mut self, descriptions: base::Descriptions) -> Self {
        self.descriptions = descriptions;
        debug!("Field `descriptions` is set to: {:#?}", self.descriptions);
        self
    }

    pub fn page_count(mut self, page_count: base::PageCount) -> Self {
        self.page_count = page_count;
        debug!("Field `page_count` is set to: {:#?}", self.page_count);
        self
    }

    pub fn publishers(mut self, publishers: base::Publishers) -> Self {
        self.publishers = publishers;
        debug!("Field `publishers` is set to: {:#?}", self.publishers);
        self
    }

    pub fn publication_dates(mut self, publication_dates: base::PublicationDates) -> Self {
        self.publication_dates = publication_dates;
        debug!(
            "Field `publication_dates` is set to: {:#?}",
            self.publication_dates
        );
        self
    }

    pub fn languages(mut self, languages: base::Languages) -> Self {
        self.languages = languages;
        debug!("Field `languages` is set to: {:#?}", self.languages);
        self
    }

    pub fn tags(mut self, tags: base::Tags) -> Self {
        self.tags = tags;
        debug!("Field `tags` is set to: {:#?}", self.tags);
        self
    }

    pub fn cover_images(mut self, cover_images: base::CoverImages) -> Self {
        self.cover_images = cover_images;
        debug!("Field `cover_images` is set to: {:#?}", self.cover_images);
        self
    }
}

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

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn parses_from_isbn() {
        use super::Metadata;
        use crate::metadata::ReconResult;
        use crate::recon::Database;
        use crate::recon::Source;
        use isbn::Isbn;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();

        let source = Source::new()
            .source(Database::GoogleBooks)
            .source(Database::OpenLibrary);

        let res: ReconResult = Metadata::from_isbn(&source, &isbn).await;

        println!("Response: {:#?}", res);
        assert!(res.is_ok());
    }
}
