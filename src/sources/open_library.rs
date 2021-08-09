use crate::sources::adaptor;
use crate::{recon::ReconError, types::metadata::Metadata};
use chrono::NaiveDate;
use core::fmt;
use isbn::Isbn;
use log::debug;
use serde::{
    de::{self, Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, Default)]
pub struct OpenLibrary(Metadata);

impl OpenLibrary {
    pub async fn from_isbn(isbn: &isbn::Isbn) -> Result<Vec<Metadata>, ReconError> {
        let isbn_key = format!("ISBN:{}", urlencoding::encode(&isbn.to_string()));
        let req = format!(
            "https://openlibrary.org/api/books?bibkeys={}&jscmd=data&format=json",
            isbn_key
        );

        debug!("ISBN: {:#?}", &isbn);
        debug!("Request: {:#?}", &req);

        let mut response = reqwest::get(req)
            .await
            .map_err(ReconError::Connection)?
            .json::<serde_json::Value>()
            .await
            .map_err(ReconError::Connection)?;

        debug!("Response: {:#?}", &response);

        #[derive(Debug, Deserialize)]
        struct ISBNResponse(#[serde(deserialize_with = "deserialize")] Metadata);

        let ISBNResponse(details) =
            serde_json::from_value(response[isbn_key].take()).map_err(ReconError::JSONParse)?;

        Ok(vec![details])
    }
}

fn deserialize<'de, D>(deserializer: D) -> Result<Metadata, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug)]
    enum Field {
        Title,
        Authors,
        Description,
        ISBN10,
        ISBN13,
        NumberOfPages,
        Ignore,
    }

    const FIELDS: &[&str] = &[
        "title",
        "authors",
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
            let mut number_of_pages = None;

            while let Some(key) = map.next_key()? {
                match key {
                    Field::Title => {
                        if title.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field("title")))
                                .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;

                        debug!("`Field::Title` value is: {:#?}", value);
                        title = adaptor::parse_string(value);
                    }

                    Field::ISBN10 => {
                        if isbn_10.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "isbn_10",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::ISBN10` value is: {:#?}", value);
                        isbn_10 = adaptor::parse_open_library_isbn(value);
                    }

                    Field::ISBN13 => {
                        if isbn_13.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "isbn_13",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::ISBN13` value is: {:#?}", value);
                        isbn_13 = adaptor::parse_open_library_isbn(value);
                    }

                    Field::Authors => {
                        if authors.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "authors",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::Authors` value is: {:#?}", value);
                        authors = adaptor::parse_authors(value);
                    }

                    Field::NumberOfPages => {
                        if number_of_pages.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "number_of_pages",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::NumberOfPages` value is: {:#?}", value);
                        number_of_pages = adaptor::parse_number_of_pages(value);
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

            let number_of_pages: Result<u16, ReconError> =
                number_of_pages.ok_or_else(|| de::Error::missing_field("number_of_pages"))?;
            let page_count: Vec<Result<u16, ReconError>> = vec![number_of_pages];

            Ok(Metadata::default()
                .titles(titles)
                .isbns(isbns)
                .authors(authors)
                .page_count(page_count))
        }
    }

    deserializer.deserialize_struct("Metadata", FIELDS, MetadataVisitor)
}

fn deserialize_extra<'de, D>(deserializer: D) -> Result<Metadata, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug)]
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
        Cover,
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
        "cover",
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
                        "cover" => Ok(Field::Cover),
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
            let mut cover = None;

            while let Some(key) = map.next_key()? {
                match key {
                    Field::Title => {
                        if title.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field("title")))
                                .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::Title` value is: {:#?}", value);
                        title = adaptor::parse_string(value);
                    }

                    Field::ISBN10 => {
                        if isbn_10.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "isbn_10",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::ISBN10` value is: {:#?}", value);
                        isbn_10 = adaptor::parse_open_library_isbn(value);
                    }

                    Field::ISBN13 => {
                        if isbn_13.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "isbn_13",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::ISBN13` value is: {:#?}", value);
                        isbn_13 = adaptor::parse_open_library_isbn(value);
                    }

                    Field::Authors => {
                        if authors.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "authors",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::Authors` value is: {:#?}", value);
                        authors = adaptor::parse_authors(value);
                    }

                    Field::NumberOfPages => {
                        if number_of_pages.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "number_of_pages",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::NumberOfPages` value is: {:#?}", value);
                        number_of_pages = adaptor::parse_number_of_pages(value);
                    }

                    Field::Description => {
                        if description.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "description",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::Description` value is: {:#?}", value);
                        description = adaptor::parse_string(value);
                    }

                    Field::Publishers => {
                        if publishers.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "publishers",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::Publishers` value is: {:#?}", value);
                        publishers = adaptor::parse_vec_hashmap_field(value, "name");
                    }

                    Field::PublishDate => {
                        if publish_date.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "publish_date",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::PublishDate` value is: {:#?}", value);
                        publish_date = adaptor::parse_publish_date(value);
                    }

                    Field::Languages => {
                        if languages.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "languages",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::Languages` value is: {:#?}", value);
                        languages = adaptor::parse_vec_hashmap_field(value, "name");
                    }

                    Field::Subjects => {
                        if subjects.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "subjects",
                            )))
                            .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::Subjects` value is: {:#?}", value);
                        subjects = adaptor::parse_vec_hashmap_field(value, "name");
                    }

                    Field::Cover => {
                        if cover.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field("cover")))
                                .map_err(V::Error::custom);
                        }

                        let value = map.next_value()?;
                        debug!("`Field::Cover` value is: {:#?}", value);
                        cover = adaptor::parse_hashmap(value);
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

            let languages: Vec<Result<String, ReconError>> = match languages {
                Some(languages) => languages,
                None => vec![],
            };

            let number_of_pages: Result<u16, ReconError> =
                number_of_pages.ok_or_else(|| de::Error::missing_field("number_of_pages"))?;
            let page_count: Vec<Result<u16, ReconError>> = vec![number_of_pages];

            let cover: Vec<Result<String, ReconError>> = match cover {
                Some(cover) => cover,
                None => vec![],
            };
            let cover_images: Vec<Result<String, ReconError>> = cover;

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
    async fn parses_extra_from_isbn() {
        use super::OpenLibrary;
        use isbn::Isbn;
        use log::info;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();
        let resp = OpenLibrary::from_isbn_extra(&isbn).await;
        info!("Response: {:#?}", resp);
        assert!(resp.is_ok());
    }
}
