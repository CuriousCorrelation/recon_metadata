use crate::sources::adaptor;
use crate::{recon::ReconError, types::metadata::Metadata};
use chrono::NaiveDate;
use core::fmt;
use isbn::Isbn;
use log::info;
use serde::{
    de::{self, Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, Default)]
pub struct OpenLibraryMetadata(Metadata);

impl OpenLibraryMetadata {
    pub async fn from_isbn(isbn: &isbn::Isbn) -> Result<Self, ReconError> {
        let isbn_key = format!("ISBN:{}", urlencoding::encode(&isbn.to_string()));
        let req = format!(
            "https://openlibrary.org/api/books?bibkeys={}&jscmd=details&format=json",
            isbn_key
        );

        info!("ISBN: {:#?}", isbn);
        info!("Request: {:#?}", req);

        unimplemented!()
    }
}

fn deserialize<'de, D>(deserializer: D) -> Result<Metadata, D::Error>
where
    D: Deserializer<'de>,
{
    enum Field {
        Title,
        Authors,
        Works,
        Description,
        ISBN10,
        ISBN13,
        NumberOfPages,
        Ignore,
    }

    const FIELDS: &[&str] = &[
        "title",
        "authors",
        "works",
        "description",
        "isbn_10",
        "isbn_13",
        "number_of_pages",
    ];

    impl<'de> Deserialize<'de> for Field {
        fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct FieldVisitor;

            impl<'de> Visitor<'de> for FieldVisitor {
                type Value = Field;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("Any of `Metadata` fields.")
                }

                fn visit_str<E>(self, value: &str) -> Result<Field, E>
                where
                    E: de::Error,
                {
                    match value {
                        "title" => Ok(Field::Title),
                        "authors" => Ok(Field::Authors),
                        "works" => Ok(Field::Works),
                        "description" => Ok(Field::Description),
                        "isbn_10" => Ok(Field::ISBN10),
                        "isbn_13" => Ok(Field::ISBN13),
                        "number_of_pages" => Ok(Field::NumberOfPages),
                        _ => Ok(Field::Ignore),
                    }
                }
            }

            deserializer.deserialize_identifier(FieldVisitor)
        }
    }

    struct MetadataVisitor;

    impl<'de> Visitor<'de> for MetadataVisitor {
        type Value = Metadata;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("struct Metadata")
        }

        fn visit_map<V>(self, mut map: V) -> Result<Metadata, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut isbn_10 = None;
            let mut isbn_13 = None;
            let mut title = None;
            let mut authors = None;
            let mut works = None;
            let mut number_of_pages = None;

            while let Some(key) = map.next_key()? {
                match key {
                    Field::Title => {
                        if title.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field("title")))
                                .map_err(V::Error::custom);
                        }

                        title = adaptor::parse_string(map.next_value()?);
                    }

                    Field::ISBN10 => {
                        if isbn_10.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "isbn_10",
                            )))
                            .map_err(V::Error::custom);
                        }

                        isbn_10 = adaptor::parse_open_library_isbn(map.next_value()?);
                    }

                    Field::ISBN13 => {
                        if isbn_13.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "isbn_13",
                            )))
                            .map_err(V::Error::custom);
                        }

                        isbn_13 = adaptor::parse_open_library_isbn(map.next_value()?);
                    }

                    Field::Authors => {
                        if authors.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "authors",
                            )))
                            .map_err(V::Error::custom);
                        }

                        authors = adaptor::parse_authors(map.next_value()?);
                    }

                    Field::Works => {
                        if works.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field("works")))
                                .map_err(V::Error::custom);
                        }

                        works = adaptor::parse_works(map.next_value()?);
                    }

                    Field::NumberOfPages => {
                        if number_of_pages.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "number_of_pages",
                            )))
                            .map_err(V::Error::custom);
                        }

                        number_of_pages = adaptor::parse_number_of_pages(map.next_value()?);
                    }

                    _ => {
                        let _ = match V::next_value::<serde::de::IgnoredAny>(&mut map) {
                            Ok(val) => val,
                            Err(err) => {
                                return Err(err);
                            }
                        };
                    }
                }
            }

            // These variable besides converting `Option` to `Result` with serde error
            // convert singular into plural if required otherwise simply
            // rename to preserve consistency in variable and method names.
            // ```
            //        ...
            //       .titles(titles)
            //        ...
            //       .descriptions(descriptions)
            //        ...
            //       .publication_dates(publication_dates)
            // ```
            // Contrast between `publish_date` and `publication_dates`
            // is to highlight `API` field name vs `Metadata` field name.
            //
            // Here `titles` is converting singular `title` into plural `titles`
            // by wrapping `title` into a `Vec`.
            //
            // `isbns` is simply renaming the variable.
            let title: Result<String, ReconError> =
                title.ok_or_else(|| de::Error::missing_field("title"))?;
            let titles: Vec<Result<String, ReconError>> = vec![title];

            let authors = match authors {
                Some(authors) => authors,
                None => vec![],
            };

            let isbn_10: Vec<Result<Isbn, ReconError>> = match isbn_10 {
                Some(isbn_10) => isbn_10,
                None => vec![],
            };

            let isbn_13: Vec<Result<Isbn, ReconError>> = match isbn_13 {
                Some(isbn_13) => isbn_13,
                None => vec![],
            };

            let mut isbns = Vec::new();
            isbns.extend(isbn_10);
            isbns.extend(isbn_13);

            let works: Vec<Result<String, ReconError>> =
                works.ok_or_else(|| de::Error::missing_field("works"))?;

            let number_of_pages: Result<u16, ReconError> =
                number_of_pages.ok_or_else(|| de::Error::missing_field("number_of_pages"))?;
            let page_count: Vec<Result<u16, ReconError>> = vec![number_of_pages];

            Ok(Metadata::default()
                .titles(titles)
                .isbns(isbns)
                .authors(authors)
                .descriptions(works)
                .page_count(page_count))
        }
    }

    deserializer.deserialize_struct("Metadata", FIELDS, MetadataVisitor)
}

#[derive(Debug, Default)]
pub struct OpenLibrary(Metadata);

impl OpenLibrary {
    pub async fn from_isbn(isbn: &isbn::Isbn) -> Result<Self, ReconError> {
        let isbn_key = format!("ISBN:{}", urlencoding::encode(&isbn.to_string()));
        let req = format!(
            "https://openlibrary.org/api/books?bibkeys={}&jscmd=details&format=json",
            isbn_key
        );

        info!("ISBN: {:#?}", isbn);
        info!("Request: {:#?}", req);

        unimplemented!()
    }
}

fn deserialize_extra<'de, D>(deserializer: D) -> Result<Metadata, D::Error>
where
    D: Deserializer<'de>,
{
    enum Field {
        Title,
        Authors,
        Publishers,
        PublishDate,
        Description,
        ISBN10,
        ISBN13,
        NumberOfPages,
        Subjects,
        Covers,
        Languages,
        Ignore,
    }

    const FIELDS: &[&str] = &[
        "title",
        "authors",
        "publishers",
        "publishedDate",
        "description",
        "isbn_10",
        "isbn_13",
        "number_of_pages",
        "subjects",
        "covers",
        "languages",
    ];

    impl<'de> Deserialize<'de> for Field {
        fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct FieldVisitor;

            impl<'de> Visitor<'de> for FieldVisitor {
                type Value = Field;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("Any of `Metadata` fields.")
                }

                fn visit_str<E>(self, value: &str) -> Result<Field, E>
                where
                    E: de::Error,
                {
                    match value {
                        "title" => Ok(Field::Title),
                        "authors" => Ok(Field::Authors),
                        "publishers" => Ok(Field::Publishers),
                        "publishedDate" => Ok(Field::PublishDate),
                        "description" => Ok(Field::Description),
                        "isbn_10" => Ok(Field::ISBN10),
                        "isbn_13" => Ok(Field::ISBN13),
                        "number_of_pages" => Ok(Field::NumberOfPages),
                        "subjects" => Ok(Field::Subjects),
                        "covers" => Ok(Field::Covers),
                        "languages" => Ok(Field::Languages),
                        _ => Ok(Field::Ignore),
                    }
                }
            }

            deserializer.deserialize_identifier(FieldVisitor)
        }
    }

    struct MetadataVisitor;

    impl<'de> Visitor<'de> for MetadataVisitor {
        type Value = Metadata;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("struct Metadata")
        }

        fn visit_map<V>(self, mut map: V) -> Result<Metadata, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut isbn_10 = None;
            let mut isbn_13 = None;
            let mut title = None;
            let mut authors = None;
            let mut description = None;
            let mut publishers = None;
            let mut publish_date = None;
            let mut languages = None;
            let mut number_of_pages = None;
            let mut subjects = None;
            let mut covers = None;

            while let Some(key) = map.next_key()? {
                match key {
                    Field::Title => {
                        if title.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field("title")))
                                .map_err(V::Error::custom);
                        }

                        title = adaptor::parse_string(map.next_value()?);
                    }

                    Field::ISBN10 => {
                        if isbn_10.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "isbn_10",
                            )))
                            .map_err(V::Error::custom);
                        }

                        isbn_10 = adaptor::parse_open_library_isbn(map.next_value()?);
                    }

                    Field::ISBN13 => {
                        if isbn_13.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "isbn_13",
                            )))
                            .map_err(V::Error::custom);
                        }

                        isbn_13 = adaptor::parse_open_library_isbn(map.next_value()?);
                    }

                    Field::Authors => {
                        if authors.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "authors",
                            )))
                            .map_err(V::Error::custom);
                        }

                        authors = adaptor::parse_authors(map.next_value()?);
                    }

                    Field::Description => {
                        if description.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "description",
                            )))
                            .map_err(V::Error::custom);
                        }

                        description = adaptor::parse_string(map.next_value()?);
                    }

                    Field::Publishers => {
                        if publishers.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "publishers",
                            )))
                            .map_err(V::Error::custom);
                        }

                        publishers = adaptor::parse_vec(map.next_value()?);
                    }

                    Field::PublishDate => {
                        if publish_date.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "publishDate",
                            )))
                            .map_err(V::Error::custom);
                        }

                        publish_date = adaptor::parse_publish_date(map.next_value()?);
                    }

                    Field::Languages => {
                        if languages.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "languages",
                            )))
                            .map_err(V::Error::custom);
                        }

                        languages = adaptor::parse_languages(map.next_value()?);
                    }

                    Field::NumberOfPages => {
                        if number_of_pages.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "number_of_pages",
                            )))
                            .map_err(V::Error::custom);
                        }

                        number_of_pages = adaptor::parse_number_of_pages(map.next_value()?);
                    }

                    Field::Subjects => {
                        if subjects.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "subjects",
                            )))
                            .map_err(V::Error::custom);
                        }

                        subjects = adaptor::parse_vec(map.next_value()?);
                    }

                    Field::Covers => {
                        if covers.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "covers",
                            )))
                            .map_err(V::Error::custom);
                        }

                        covers = adaptor::parse_covers(map.next_value()?);
                    }
                    _ => {
                        let _ = match V::next_value::<serde::de::IgnoredAny>(&mut map) {
                            Ok(val) => val,
                            Err(err) => {
                                return Err(err);
                            }
                        };
                    }
                }
            }

            // These variable besides converting `Option` to `Result` with serde error
            // convert singular into plural if required otherwise simply
            // rename to preserve consistency in variable and method names.
            // ```
            //        ...
            //       .titles(titles)
            //        ...
            //       .descriptions(descriptions)
            //        ...
            //       .publication_dates(publication_dates)
            // ```
            // Contrast between `publish_date` and `publication_dates`
            // is to highlight `API` field name vs `Metadata` field name.
            //
            // Here `titles` is converting singular `title` into plural `titles`
            // by wrapping `title` into a `Vec`.
            //
            // `isbns` is simply renaming the variable.
            let title: Result<String, ReconError> =
                title.ok_or_else(|| de::Error::missing_field("title"))?;
            let titles: Vec<Result<String, ReconError>> = vec![title];

            let authors = match authors {
                Some(authors) => authors,
                None => vec![],
            };

            let isbn_10: Vec<Result<Isbn, ReconError>> = match isbn_10 {
                Some(isbn_10) => isbn_10,
                None => vec![],
            };

            let isbn_13: Vec<Result<Isbn, ReconError>> = match isbn_13 {
                Some(isbn_13) => isbn_13,
                None => vec![],
            };

            let mut isbns = Vec::new();
            isbns.extend(isbn_10);
            isbns.extend(isbn_13);

            let descriptions: Vec<Result<String, ReconError>> = match description {
                Some(description) => vec![description],
                None => vec![],
            };

            let publishers: Vec<Result<String, ReconError>> =
                publishers.ok_or_else(|| de::Error::missing_field("publishers"))?;

            let publication_dates: Vec<Result<NaiveDate, ReconError>> = match publish_date {
                Some(publish_date) => vec![publish_date],
                None => vec![],
            };

            let languages: Vec<Result<String, ReconError>> =
                languages.ok_or_else(|| de::Error::missing_field("languages"))?;

            let number_of_pages: Result<u16, ReconError> =
                number_of_pages.ok_or_else(|| de::Error::missing_field("number_of_pages"))?;
            let page_count: Vec<Result<u16, ReconError>> = vec![number_of_pages];

            let covers: Vec<Result<String, ReconError>> =
                covers.ok_or_else(|| de::Error::missing_field("covers"))?;
            let cover_images: Vec<Result<String, ReconError>> = covers;

            let subjects = match subjects {
                Some(subjects) => subjects,
                None => vec![],
            };
            let tags: Vec<Result<String, ReconError>> = subjects;

            Ok(Metadata::default()
                .titles(titles)
                .isbns(isbns)
                .authors(authors)
                .descriptions(descriptions)
                .publishers(publishers)
                .publication_dates(publication_dates)
                .languages(languages)
                .page_count(page_count)
                .tags(tags)
                .cover_images(cover_images))
        }
    }

    deserializer.deserialize_struct("Metadata", FIELDS, MetadataVisitor)
}

#[cfg(test)]
mod test {
    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn parses_from_isbn() {
        use super::OpenLibrary;
        use isbn::Isbn;
        use log::info;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();
        let resp = OpenLibrary::from_isbn(&isbn).await;
        info!("Response: {:#?}", resp);
        assert!(resp.is_ok());
    }

    #[tokio::test]
    async fn parses_minimal_from_isbn() {
        use super::OpenLibraryMetadata;
        use isbn::Isbn;
        use log::info;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();
        let resp = OpenLibraryMetadata::from_isbn(&isbn).await;
        info!("Response: {:#?}", resp);
        assert!(resp.is_ok());
    }
}
