use crate::recon::Source;
use crate::{
    recon::ReconError,
    source::{google_books::GoogleBooks, open_library::OpenLibrary},
};
use chrono::NaiveDate;
use futures::future::join_all;
use isbn2::{Isbn, Isbn10, Isbn13};
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use std::collections::HashSet;
use std::ops::Add;

/// Information about type types of cover images according to their size
#[derive(Debug, Default, Serialize, PartialEq, Eq, Clone)]
pub(crate) struct CoverImage {
    pub(crate) small_thumbnail: HashSet<String>,
    pub(crate) thumbnail:       HashSet<String>,
    pub(crate) small:           HashSet<String>,
    pub(crate) medium:          HashSet<String>,
    pub(crate) large:           HashSet<String>,
    pub(crate) extra_large:     HashSet<String>,
}

impl CoverImage {
    pub(crate) fn extend(&mut self, other: Self) -> &mut Self {
        self.small_thumbnail.extend(other.small_thumbnail);
        self.thumbnail.extend(other.thumbnail);
        self.small.extend(other.small);
        self.medium.extend(other.medium);
        self.large.extend(other.large);
        self.extra_large.extend(other.extra_large);

        self
    }
}

/// [`Metadata`] type contains information to uniquely identify a book.
///
/// Contains one or multiple of the following:
///  1. ISBN10
///  2. ISBN13
///  3. Title
///  4. Author
///  5. Description
///  6. Page count
///  7. Publisher
///  8. Publication Date
///  9. Language
/// 10. Tag
/// 11. Cover image
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
    pub(crate) cover_image:      CoverImage,
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
            Source::Goodreads => {
                todo!("fix Goodreads::from_description(description).await, tendrill error")
            }
        }
    }

    async fn isbn_from_source(source: &Source, isbn: &Isbn) -> Result<Metadata, ReconError> {
        match source {
            Source::GoogleBooks => GoogleBooks::from_isbn(isbn).await,
            Source::OpenLibrary => OpenLibrary::from_isbn(isbn).await,
            Source::Amazon => unimplemented!(),
            Source::Goodreads => todo!("fix Goodreads::from_isbn(isbn).await, tendrill error"),
        }
    }

    /// Performs parallel ISBN search.
    /// First arg requires a list of [`Source`],
    /// second an `Isbn`.
    /// Combines information for a complete and exasutive result [`Metadata`].
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

    /// Performs parallel search on ISBNs provided by first argument.
    /// Second argument describes sources to cross-examine.
    /// Returns a list of [`Metadata`] that matches description
    /// provided by the third argument.
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
    use log::info;

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

        info!("Response: {:#?}", res);
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
            Metadata::from_description(&Source::GoogleBooks, &sources, description).await;

        info!("Response: {:#?}", res);
        assert!(res.is_ok());
    }
}
