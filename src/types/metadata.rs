use crate::{
    recon::{ReconError, ReconSetup},
    sources::{google_books::GoogleBooks, open_library::OpenLibrary},
};

use super::base;
use isbn::Isbn;
use log::debug;

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
    isbns:        base::ISBNs,
    titles:       base::Titles,
    authors:      base::Authors,
    descriptions: base::Descriptions,
    page_count:   base::PageCount,
    extra:        self::Extra,
}

#[derive(Debug, Default)]
pub struct Extra {
    publishers:        base::Publishers,
    publication_dates: base::PublicationDates,
    languages:         base::Languages,
    tags:              base::Tags,
    cover_images:      base::CoverImages,
}

/// A type synonym for `Result<Vec<Metadata>, ReconError>`
pub type ReconResult = Result<Vec<Metadata>, ReconError>;

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
        self.extra.publishers = publishers;
        debug!("Field `publishers` is set to: {:#?}", self.extra.publishers);
        self
    }

    pub fn publication_dates(mut self, publication_dates: base::PublicationDates) -> Self {
        self.extra.publication_dates = publication_dates;
        debug!(
            "Field `extra.publication_dates` is set to: {:#?}",
            self.extra.publication_dates
        );
        self
    }

    pub fn languages(mut self, languages: base::Languages) -> Self {
        self.extra.languages = languages;
        debug!(
            "Field `extra.languages` is set to: {:#?}",
            self.extra.languages
        );
        self
    }

    pub fn tags(mut self, tags: base::Tags) -> Self {
        self.extra.tags = tags;
        debug!("Field `extra.tags` is set to: {:#?}", self.extra.tags);
        self
    }

    pub fn cover_images(mut self, cover_images: base::CoverImages) -> Self {
        self.extra.cover_images = cover_images;
        debug!(
            "Field `extra.cover_images` is set to: {:#?}",
            self.extra.cover_images
        );
        self
    }
}

impl Metadata {
    pub async fn from_isbn(recon_setup: &ReconSetup, isbn: &Isbn) -> ReconResult {
        use crate::recon::SearchProvider;

        match recon_setup.search_provider {
            SearchProvider::GoogleBooks => GoogleBooks::from_isbn(isbn).await,
            SearchProvider::OpenLibrary => OpenLibrary::from_isbn(isbn).await,
            SearchProvider::Amazon => unimplemented!(),
            SearchProvider::Goodreads => unimplemented!(),
        }
    }

    pub async fn from_isbn_extra(recon_setup: &ReconSetup, isbn: &Isbn) -> ReconResult {
        use crate::recon::SearchProvider;

        match recon_setup.search_provider {
            SearchProvider::GoogleBooks => GoogleBooks::from_isbn_extra(isbn).await,
            SearchProvider::OpenLibrary => OpenLibrary::from_isbn_extra(isbn).await,
            SearchProvider::Amazon => unimplemented!(),
            SearchProvider::Goodreads => unimplemented!(),
        }
    }
}
