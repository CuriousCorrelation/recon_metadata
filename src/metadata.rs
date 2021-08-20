use crate::recon::Source;
use crate::{
    recon::ReconError, source::google_books::GoogleBooks, source::open_library::OpenLibrary,
};
use chrono::NaiveDate;
use futures::future::join_all;
use isbn2::{Isbn, Isbn10, Isbn13};
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
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
#[derive(Debug, Default, Serialize)]
pub struct Metadata {
    #[serde(serialize_with = "serialize_hashset_isbn10")]
    pub(crate) isbn10:           HashSet<Isbn10>,
    #[serde(serialize_with = "serialize_hashset_isbn13")]
    pub(crate) isbn13:           HashSet<Isbn13>,
    pub(crate) title:            HashSet<String>,
    pub(crate) author:           HashSet<String>,
    pub(crate) description:      HashSet<String>,
    pub(crate) page_count:       HashSet<u16>,
    pub(crate) publisher:        HashSet<String>,
    #[serde(serialize_with = "serialize_hashset_naivedate")]
    pub(crate) publication_date: HashSet<NaiveDate>,
    pub(crate) language:         HashSet<String>,
    pub(crate) tag:              HashSet<String>,
    pub(crate) cover_image:      HashSet<String>,
}

fn serialize_hashset_naivedate<S>(
    dates: &HashSet<NaiveDate>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(dates.len()))?;

    for date in dates {
        let s = date.format("%Y-%m-%d").to_string();
        seq.serialize_element(&s)?;
    }
    seq.end()
}

fn serialize_hashset_isbn10<S>(isbn10s: &HashSet<Isbn10>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(isbn10s.len()))?;

    for isbn10 in isbn10s {
        let s = isbn10.to_string();
        seq.serialize_element(&s)?;
    }
    seq.end()
}

fn serialize_hashset_isbn13<S>(isbn13s: &HashSet<Isbn13>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(isbn13s.len()))?;

    for isbn13 in isbn13s {
        let s = isbn13.to_string();
        seq.serialize_element(&s)?;
    }
    seq.end()
}

impl Add for Metadata {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        self.isbn10.extend(other.isbn10);
        self.isbn13.extend(other.isbn13);
        self.title.extend(other.title);
        self.author.extend(other.author);
        self.description.extend(other.description);
        self.page_count.extend(other.page_count);
        self.publisher.extend(other.publisher);
        self.publication_date.extend(other.publication_date);
        self.language.extend(other.language);
        self.tag.extend(other.tag);
        self.cover_image.extend(other.cover_image);

        self
    }
}

impl Metadata {
    async fn description_from_source(
        source: &Source,
        description: &str,
    ) -> Result<Vec<Isbn>, ReconError> {
        match source {
            Source::GoogleBooks => GoogleBooks::from_description(description).await,
            Source::OpenLibrary => OpenLibrary::from_description(description).await,
            Source::Amazon => unimplemented!(),
            Source::Goodreads => unimplemented!(),
        }
    }

    async fn isbn_from_source(source: &Source, isbn: &Isbn) -> Result<Metadata, ReconError> {
        match source {
            Source::GoogleBooks => GoogleBooks::from_isbn(isbn).await,
            Source::OpenLibrary => OpenLibrary::from_isbn(isbn).await,
            Source::Amazon => unimplemented!(),
            Source::Goodreads => unimplemented!(),
        }
    }

    /// A simple `Isbn` search that'll use only one provider defined by
    /// [`ReconSetup`].
    /// An eclectic / diversified / exaustive search that'll use search provider
    /// for initial information and fill in the blacks making expensive calls
    /// but returning almost complete information about the book
    /// provides by the sources defined by [`Source`].
    pub async fn from_isbn(sources: &[Source], isbn: &Isbn) -> Result<Metadata, ReconError> {
        let mut metadata = Metadata::default();

        let futures_list = sources
            .iter()
            .map(|s| Self::isbn_from_source(s, isbn))
            .collect::<Vec<_>>();

        let metadata_list = join_all(futures_list).await;

        for m in metadata_list {
            metadata = metadata + m?;
        }

        Ok(metadata)
    }

    pub async fn from_description(
        search: &Source,
        sources: &[Source],
        description: &str,
    ) -> Result<Vec<Metadata>, ReconError> {
        let isbns: Vec<Isbn> = Self::description_from_source(search, description).await?;

        let futures_list = isbns
            .iter()
            .map(|isbn| Self::from_isbn(sources, isbn))
            .collect::<Vec<_>>();

        let metadata_list = join_all(futures_list).await;

        Ok(metadata_list.into_iter().flatten().collect())
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
        use crate::recon::{ReconError, Source};
        use isbn2::Isbn;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();

        let sources = [Source::GoogleBooks, Source::OpenLibrary];

        let res: Result<Metadata, ReconError> = Metadata::from_isbn(&sources, &isbn).await;

        debug!("Response: {:#?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn parses_from_description() {
        use super::Metadata;
        use crate::recon::{ReconError, Source};

        init_logger();

        let description = "This is how you lose the time war";

        let sources = [Source::GoogleBooks, Source::OpenLibrary];

        let res: Result<Vec<Metadata>, ReconError> =
            Metadata::from_description(&Source::GoogleBooks, &sources, &description).await;

        debug!("Response: {:#?}", res);
        assert!(res.is_ok());

        let res: Result<Vec<Metadata>, ReconError> =
            Metadata::from_description(&Source::OpenLibrary, &sources, &description).await;

        debug!("Response: {:#?}", res);
        assert!(res.is_ok());
    }
}
