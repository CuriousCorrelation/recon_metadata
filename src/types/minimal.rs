use super::base;
use log::info;

#[derive(Debug, Default)]

/// [`Minimal`] contains minimal essential information to uniquely identify a book.
pub struct Minimal {
    /// `isbns` is built from a [`Vec`] of
    /// [ISBN](https://en.wikipedia.org/wiki/International_Standard_Book_Number).
    isbns:        base::ISBNs,
    /// `titles` is built from a [`Vec`] of book titles.
    /// A book can have multiple titles depending on translations
    /// or way of writing e.g. "Book", "Book #1", "Series Name, Book #1", etc.
    titles:       base::Titles,
    /// `titles` is built from a [`Vec`] of book authors.
    authors:      base::Authors,
    /// `descriptions` is a short descriptions for the book.
    descriptions: base::Descriptions,
}

impl Minimal {
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

    pub fn descriptions(mut self, descriptions: base::Descriptions) -> Self {
        self.descriptions = descriptions;
        info!("Field `descriptions` is set to: {:#?}", self.descriptions);
        self
    }
}
