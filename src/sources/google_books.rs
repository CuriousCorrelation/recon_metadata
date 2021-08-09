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

#[derive(Debug)]
pub struct GoogleBooks(Metadata);

impl GoogleBooks {
    pub async fn from_isbn(isbn: &isbn::Isbn, extra: bool) -> Result<Vec<Metadata>, ReconError> {
        let req = format!(
            "https://www.googleapis.com/books/v1/volumes?q=isbn:{}",
            urlencoding::encode(&isbn.to_string())
        );

        debug!("ISBN: {:#?}", &isbn);
        debug!("Request: {:#?}", &req);

        #[derive(Debug, Deserialize)]
        struct Items {
            items: Vec<VolumeInfo>,
        }

        #[derive(Debug, Deserialize)]
        struct VolumeInfo {
            #[serde(rename = "volumeInfo", deserialize_with = "deserialize")]
            volume_info: Metadata,
        }

        let response = reqwest::get(req)
            .await
            .map_err(ReconError::Connection)?
            .json::<Items>()
            .await
            .map_err(ReconError::Connection)?;

        debug!("Response: {:#?}", &response);

        Ok(response
            .items
            .into_iter()
            .map(|v| v.volume_info)
            .collect::<Vec<Metadata>>())
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
        IndustryIdentifiers,
        Isbn,
        PageCount,
        Ignore,
    }

    const FIELDS: &[&str] = &[
        "title",
        "authors",
        "description",
        "industryIdentifiers",
        "isbn",
        "pageCount",
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
                        "industryIdentifiers" => Ok(Field::IndustryIdentifiers),
                        "identifier" => Ok(Field::Isbn),
                        "pageCount" => Ok(Field::PageCount),
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
            let mut isbn = None;
            let mut title = None;
            let mut authors = None;
            let mut description = None;
            let mut page_count = None;

            while let Some(key) = map.next_key()? {
                debug!("deserialize::visit_map key is: {:#?}", &key);
                match key {
                    Field::Title => {
                        if title.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field("title")))
                                .map_err(V::Error::custom);
                        }
                        title = adaptor::parse_string(map.next_value()?);
                    }

                    Field::IndustryIdentifiers => {
                        if isbn.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "industryIdentifiers",
                            )))
                            .map_err(V::Error::custom);
                        }
                        isbn = adaptor::parse_google_books_isbn(map.next_value()?);
                    }

                    Field::Authors => {
                        if authors.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "authors",
                            )))
                            .map_err(V::Error::custom);
                        }
                        authors = adaptor::parse_vec(map.next_value()?);
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

                    Field::PageCount => {
                        if page_count.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "page_count",
                            )))
                            .map_err(V::Error::custom);
                        }
                        page_count = adaptor::parse_page_count(map.next_value()?);
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
            // Contrast between `published_date` and `publication_dates`
            // is to highlight `API` field name vs `Metadata` field name.
            //
            // Here `titles` is converting singular `title` into plural `titles`
            // by wrapping `title` into a `Vec`.
            //
            // `isbns` is simply renaming the variable.
            let title: Result<String, ReconError> =
                title.ok_or_else(|| de::Error::missing_field("title"))?;
            let titles: Vec<Result<String, ReconError>> = vec![title];

            let authors: Vec<Result<String, ReconError>> =
                authors.ok_or_else(|| de::Error::missing_field("authors"))?;

            let isbn: Vec<Result<Isbn, ReconError>> =
                isbn.ok_or_else(|| de::Error::missing_field("industryIdentifiers"))?;
            let isbns = isbn;

            let description: Result<String, ReconError> =
                description.ok_or_else(|| de::Error::missing_field("description"))?;
            let descriptions: Vec<Result<String, ReconError>> = vec![description];

            let page_count: Result<u16, ReconError> =
                page_count.ok_or_else(|| de::Error::missing_field("pageCount"))?;
            let page_count: Vec<Result<u16, ReconError>> = vec![page_count];

            Ok(Metadata::default()
                .titles(titles)
                .isbns(isbns)
                .authors(authors)
                .descriptions(descriptions)
                .page_count(page_count))
        }
    }

    deserializer.deserialize_struct("Metadata", FIELDS, MetadataVisitor)
}

fn deserialize_extra<'de, D>(deserializer: D) -> Result<Metadata, D::Error>
where
    D: Deserializer<'de>,
{
    enum Field {
        Title,
        Authors,
        Publisher,
        PublishedDate,
        Description,
        IndustryIdentifiers,
        Isbn,
        PageCount,
        Categories,
        ImageLinks,
        Language,
        Ignore,
    }

    const FIELDS: &[&str] = &[
        "title",
        "authors",
        "publisher",
        "publishedDate",
        "description",
        "industryIdentifiers",
        "isbn",
        "pageCount",
        "categories",
        "imageLinks",
        "language",
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
                        "publisher" => Ok(Field::Publisher),
                        "publishedDate" => Ok(Field::PublishedDate),
                        "description" => Ok(Field::Description),
                        "industryIdentifiers" => Ok(Field::IndustryIdentifiers),
                        "identifier" => Ok(Field::Isbn),
                        "pageCount" => Ok(Field::PageCount),
                        "categories" => Ok(Field::Categories),
                        "imageLinks" => Ok(Field::ImageLinks),
                        "language" => Ok(Field::Language),
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
            let mut isbn = None;
            let mut title = None;
            let mut authors = None;
            let mut description = None;
            let mut publisher = None;
            let mut published_date = None;
            let mut language = None;
            let mut page_count = None;
            let mut categories = None;
            let mut image_links = None;

            while let Some(key) = map.next_key()? {
                match key {
                    Field::Title => {
                        if title.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field("title")))
                                .map_err(V::Error::custom);
                        }
                        title = adaptor::parse_string(map.next_value()?);
                    }

                    Field::IndustryIdentifiers => {
                        if isbn.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "industryIdentifiers",
                            )))
                            .map_err(V::Error::custom);
                        }
                        isbn = adaptor::parse_google_books_isbn(map.next_value()?);
                    }

                    Field::Authors => {
                        if authors.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "authors",
                            )))
                            .map_err(V::Error::custom);
                        }
                        authors = adaptor::parse_vec(map.next_value()?);
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

                    Field::Publisher => {
                        if publisher.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "publisher",
                            )))
                            .map_err(V::Error::custom);
                        }
                        publisher = adaptor::parse_string(map.next_value()?);
                    }

                    Field::PublishedDate => {
                        if published_date.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "publishedDate",
                            )))
                            .map_err(V::Error::custom);
                        }
                        published_date = adaptor::parse_published_date(map.next_value()?);
                    }

                    Field::Language => {
                        if language.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "language",
                            )))
                            .map_err(V::Error::custom);
                        }
                        language = adaptor::parse_string(map.next_value()?);
                    }

                    Field::PageCount => {
                        if page_count.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "page_count",
                            )))
                            .map_err(V::Error::custom);
                        }
                        page_count = adaptor::parse_page_count(map.next_value()?);
                    }

                    Field::Categories => {
                        if categories.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "categories",
                            )))
                            .map_err(V::Error::custom);
                        }
                        categories = adaptor::parse_vec(map.next_value()?);
                    }

                    Field::ImageLinks => {
                        if image_links.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "imageLinks",
                            )))
                            .map_err(V::Error::custom);
                        }
                        image_links = adaptor::parse_image_links(map.next_value()?);
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
            // Contrast between `published_date` and `publication_dates`
            // is to highlight `API` field name vs `Metadata` field name.
            //
            // Here `titles` is converting singular `title` into plural `titles`
            // by wrapping `title` into a `Vec`.
            //
            // `isbns` is simply renaming the variable.
            let title: Result<String, ReconError> =
                title.ok_or_else(|| de::Error::missing_field("title"))?;
            let titles: Vec<Result<String, ReconError>> = vec![title];

            let authors: Vec<Result<String, ReconError>> =
                authors.ok_or_else(|| de::Error::missing_field("authors"))?;

            let isbn: Vec<Result<Isbn, ReconError>> =
                isbn.ok_or_else(|| de::Error::missing_field("industryIdentifiers"))?;
            let isbns = isbn;

            let description: Result<String, ReconError> =
                description.ok_or_else(|| de::Error::missing_field("description"))?;
            let descriptions: Vec<Result<String, ReconError>> = vec![description];

            let publisher: Result<String, ReconError> =
                publisher.ok_or_else(|| de::Error::missing_field("publisher"))?;
            let publishers: Vec<Result<String, ReconError>> = vec![publisher];

            let published_date: Result<NaiveDate, ReconError> =
                published_date.ok_or_else(|| de::Error::missing_field("publishedDate"))?;
            let publication_dates: Vec<Result<NaiveDate, ReconError>> = vec![published_date];

            let language: Result<String, ReconError> =
                language.ok_or_else(|| de::Error::missing_field("language"))?;
            let languages: Vec<Result<String, ReconError>> = vec![language];

            let page_count: Result<u16, ReconError> =
                page_count.ok_or_else(|| de::Error::missing_field("pageCount"))?;
            let page_count: Vec<Result<u16, ReconError>> = vec![page_count];

            let image_links: Vec<Result<String, ReconError>> =
                image_links.ok_or_else(|| de::Error::missing_field("imageLinks"))?;
            let cover_images: Vec<Result<String, ReconError>> = image_links;

            let categories: Vec<Result<String, ReconError>> =
                categories.ok_or_else(|| de::Error::missing_field("categories"))?;
            let tags: Vec<Result<String, ReconError>> = categories;

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
        use super::GoogleBooks;
        use isbn::Isbn;
        use log::debug;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();
        let resp = GoogleBooks::from_isbn(&isbn).await;
        debug!("Response: {:#?}", resp);
        assert!(resp.is_ok())
    }

    #[tokio::test]
    async fn parses_extra_from_isbn() {
        use super::GoogleBooks;
        use isbn::Isbn;
        use log::debug;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();
        let resp = GoogleBooks::from_isbn_extra(&isbn).await;
        debug!("Response: {:#?}", resp);
        assert!(resp.is_ok())
    }
}
