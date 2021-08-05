use super::base;
use log::info;

#[derive(Debug, Default)]

/// [`Metadata`] type provides additional information to [`Minimal`]'s essential base.
/// [`Metadata`] contains the following:
/// 1. (Certain) ISBN-10 and/or ISBN-13
/// 2. (Certain) Title(s)
/// 3. (Certain) Author(s) [Can be "unknown"]
/// 4. (Probable) Description
/// 5. (Probable) Publisher(s)
/// 6. (Probable) Publication Date(s)
/// 7. (Probable) Language
/// 8. (Probable) Number of pages
/// 9. (Probable) Cover image
pub struct Metadata {
    // `isbns` is built from a [`Vec`] of
    // [ISBN](https://en.wikipedia.org/wiki/International_Standard_Book_Number).
    isbns:             base::ISBNs,
    // `titles` is built from a [`Vec`] of book titles.
    // A book can have multiple titles depending on translations
    // or way of writing e.g. "Book", "Book #1", "Series Name, Book #1", etc.
    titles:            base::Titles,
    // `titles` is built from a [`Vec`] of book authors.
    authors:           base::Authors,
    // `description` is a short description for the book.
    description:       base::Description,
    // `publishers` is built from a [`Vec`] of book publishers.
    publishers:        base::Publishers,
    // `publisher_dates` is built from a [`Vec`] of book's publication dates.
    // A book can have multiple publication dates depening on publication rights
    // and individual publisher or publishing houses.
    publication_dates: base::PublicationDates,
    // `languages` refers to the number of languages the book is published in.
    languages:         base::Languages,
    // `page_count` is the number of pages the book has.
    // A book can have different number of pages depending on the type of book.
    // e.g. paperback, hardcover, e-book (different formats), etc.
    page_count:        base::PageCount,
    // `tags` are the book subjects and/or book genre.
    tags:              base::Tags,
    // `cover_images/` is build from a [`Vec`] of URLs of book cover images
    // of different sizes and formats.
    cover_images:      base::CoverImages,
}

impl Metadata {
    pub fn isbns(mut self, isbns: base::ISBNs) -> Self {
        self.isbns = isbns;
        info!("Field `isbns` is set to: {:#?}", self.isbns);
        self
    }

    pub fn titles(mut self, titles: base::Titles) -> Self {
        self.titles = titles;
        info!("Field `titles` is set to: {:#?}", self.titles);
        self
    }

    pub fn authors(mut self, authors: base::Authors) -> Self {
        self.authors = authors;
        info!("Field `authors` is set to: {:#?}", self.authors);
        self
    }

    pub fn description(mut self, description: base::Description) -> Self {
        self.description = description;
        info!("Field `description` is set to: {:#?}", self.description);
        self
    }

    pub fn publishers(mut self, publishers: base::Publishers) -> Self {
        self.publishers = publishers;
        info!("Field `publishers` is set to: {:#?}", self.publishers);
        self
    }

    pub fn publication_dates(mut self, publication_dates: base::PublicationDates) -> Self {
        self.publication_dates = publication_dates;
        info!(
            "Field `publication_dates` is set to: {:#?}",
            self.publication_dates
        );
        self
    }

    pub fn languages(mut self, languages: base::Languages) -> Self {
        self.languages = languages;
        info!("Field `languages` is set to: {:#?}", self.languages);
        self
    }

    pub fn page_count(mut self, page_count: base::PageCount) -> Self {
        self.page_count = page_count;
        info!("Field `page_count` is set to: {:#?}", self.page_count);
        self
    }

    pub fn tags(mut self, tags: base::Tags) -> Self {
        self.tags = tags;
        info!("Field `tags` is set to: {:#?}", self.tags);
        self
    }

    pub fn cover_images(mut self, cover_images: base::CoverImages) -> Self {
        self.cover_images = cover_images;
        info!("Field `cover_images` is set to: {:#?}", self.cover_images);
        self
    }
}
