use crate::interface::recon::ReconError;
use core::fmt;
use log::info;
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct GoogleBooks {
    // "industryIdentifiers"/["identifier"]
    isbn:             Vec<String>,
    // "title"
    title:            String,
    // "authors"
    author:           Vec<String>,
    // "description"
    description:      String,
    // "publisher"
    publisher:        String,
    // "publishedDate"
    publication_date: String,
    // "language"
    language:         String,
    // "pageCount"
    page_count:       u16,
    // "categories"
    tag:              Vec<String>,
    // "imageLinks"/["smallThumbnail", "thumbnail", ... ]
    cover_image:      Vec<String>,
}

impl GoogleBooks {
    pub fn isbn(mut self, isbn: Vec<String>) -> Self {
        self.isbn = isbn;
        info!("Field `isbn` is set to: {:#?}", self.isbn);
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        info!("Field `title` is set to: {:#?}", self.title);
        self
    }

    pub fn authors(mut self, authors: Vec<String>) -> Self {
        self.authors = authors;
        info!("Field `authors` is set to: {:#?}", self.authors);
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        info!("Field `description` is set to: {:#?}", self.description);
        self
    }

    pub fn publisher(mut self, publisher: String) -> Self {
        self.publisher = publisher;
        info!("Field `publisher` is set to: {:#?}", self.publisher);
        self
    }

    pub fn publication_date(mut self, publication_date: String) -> Self {
        self.publication_date = publication_date;
        info!(
            "Field `publication_date` is set to: {:#?}",
            self.publication_date
        );
        self
    }

    pub fn language(mut self, language: String) -> Self {
        self.language = language;
        info!("Field `language` is set to: {:#?}", self.language);
        self
    }

    pub fn pages(mut self, page_count: u16) -> Self {
        self.page_count = page_count;
        info!("Field `page_count` is set to: {:#?}", self.page_count);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        info!("Field `tags` is set to: {:#?}", self.tags);
        self
    }

    pub fn cover_images(mut self, cover_images: Vec<String>) -> Self {
        self.cover_images = cover_images;
        info!("Field `cover_images` is set to: {:#?}", self.cover_images);
        self
    }
}

impl GoogleBooks {
    pub async fn from_isbn(isbn: &isbn::Isbn) -> Result<Vec<Self>, ReconError> {
        let base_url = "https://www.googleapis.com/books/v1/volumes?q=isbn:";
        let req = format!("{}{}", base_url, urlencoding::encode(&isbn.to_string()));

        info!("ISBN: {:#?}", isbn);
        info!("Base URL: {:#?}", base_url);
        info!("Request: {:#?}", req);

        serde_json::from_value::<Vec<serde_json::Value>>(
            reqwest::get(req)
                .await
                .map_err(ReconError::Connection)?
                .json::<serde_json::Value>()
                .await
                .map_err(ReconError::Connection)?["items"]
                .take(),
        )
        .map_err(ReconError::JSONParse)?
        .iter_mut()
        .map(|v: &mut serde_json::Value| {
            serde_json::from_value(v["volumeInfo"].take()).map_err(ReconError::JSONParse)
        })
        .collect::<Result<Vec<Self>, ReconError>>()
    }
}

impl<'de> Deserialize<'de> for GoogleBooks {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Title,
            Authors,
            Publisher,
            PublicationDate,
            Description,
            IndustryIdentifiers,
            Isbn,
            PageCount,
            Tags,
            CoverImages,
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
            "tags",
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
                        formatter.write_str("Any of `GoogleBooks` fields.")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "title" => Ok(Field::Title),
                            "authors" => Ok(Field::Authors),
                            "publisher" => Ok(Field::Publisher),
                            "publishedDate" => Ok(Field::PublicationDate),
                            "description" => Ok(Field::Description),
                            "industryIdentifiers" => Ok(Field::IndustryIdentifiers),
                            "identifier" => Ok(Field::Isbn),
                            "pageCount" => Ok(Field::PageCount),
                            "categories" => Ok(Field::Tags),
                            "imageLinks" => Ok(Field::CoverImages),
                            "language" => Ok(Field::Language),
                            _ => Ok(Field::Ignore),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct GoogleBooksVisitor;

        impl<'de> Visitor<'de> for GoogleBooksVisitor {
            type Value = GoogleBooks;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct GoogleBooks")
            }

            fn visit_map<V>(self, mut map: V) -> Result<GoogleBooks, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut isbns = None;
                let mut title = None;
                let mut authors = None;
                let mut description = None;
                let mut publisher = None;
                let mut publication_date = None;
                let mut language = None;
                let mut page_count = None;
                let mut tags = None;
                let mut cover_images = None;

                // Helper functions.
                //
                // Functions named after fields such as `parse_to_page_count`
                // are unique function that isn't used by any other field.
                //
                // Generic function such as `parse_to_string` are used by multiple fields.
                let parse_to_string = |string: String| Some(string);
                let parse_to_page_count = |page_count: u16| Some(page_count);
                let parse_to_vec_of_string = |vec_of_string: Vec<String>| Some(vec_of_string);
                let parse_to_cover_images = |cover_images: HashMap<String, String>| {
                    Some(cover_images.into_iter().map(|(_, v)| v).collect())
                };
                let parse_isbns = |mut isbns: Vec<HashMap<String, String>>| {
                    Some(
                        isbns
                            .iter_mut()
                            .map(|h| h.remove("identifier"))
                            .flatten()
                            .collect::<Vec<String>>(),
                    )
                };

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Title => {
                            if title.is_some() {
                                return Err(de::Error::duplicate_field("title"));
                            }
                            title = parse_to_string(map.next_value()?);
                        }

                        Field::IndustryIdentifiers => {
                            if isbns.is_some() {
                                return Err(de::Error::duplicate_field("industryIdentifiers"));
                            }
                            isbns = parse_isbns(map.next_value()?);
                        }

                        Field::Authors => {
                            if authors.is_some() {
                                return Err(de::Error::duplicate_field("authors"));
                            }
                            authors = parse_to_vec_of_string(map.next_value()?);
                        }

                        Field::Description => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }
                            description = parse_to_string(map.next_value()?);
                        }

                        Field::Publisher => {
                            if publisher.is_some() {
                                return Err(de::Error::duplicate_field("publisher"));
                            }
                            publisher = parse_to_string(map.next_value()?);
                        }

                        Field::PublicationDate => {
                            if publication_date.is_some() {
                                return Err(de::Error::duplicate_field("publicationDate"));
                            }
                            publication_date = parse_to_string(map.next_value()?);
                        }

                        Field::Language => {
                            if language.is_some() {
                                return Err(de::Error::duplicate_field("language"));
                            }
                            language = parse_to_string(map.next_value()?);
                        }

                        Field::PageCount => {
                            if page_count.is_some() {
                                return Err(de::Error::duplicate_field("pageCount"));
                            }
                            page_count = parse_to_page_count(map.next_value()?);
                        }

                        Field::Tags => {
                            if tags.is_some() {
                                return Err(de::Error::duplicate_field("tags"));
                            }
                            tags = parse_to_vec_of_string(map.next_value()?);
                        }

                        Field::CoverImages => {
                            if cover_images.is_some() {
                                return Err(de::Error::duplicate_field("imageLinks"));
                            }
                            cover_images = parse_to_cover_images(map.next_value()?);
                        }
                        _ => {}
                    }
                }

                let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
                let isbns = isbns.ok_or_else(|| de::Error::missing_field("industryIdentifiers"))?;
                let authors = authors.ok_or_else(|| de::Error::missing_field("authors"))?;
                let description =
                    description.ok_or_else(|| de::Error::missing_field("description"))?;
                let publisher = publisher.ok_or_else(|| de::Error::missing_field("publisher"))?;
                let language = language.ok_or_else(|| de::Error::missing_field("language"))?;
                let publication_date =
                    publication_date.ok_or_else(|| de::Error::missing_field("publicationDate"))?;
                let page_count = page_count.ok_or_else(|| de::Error::missing_field("pageCount"))?;
                let tags = tags.ok_or_else(|| de::Error::missing_field("categories"))?;
                let cover_images =
                    cover_images.ok_or_else(|| de::Error::missing_field("imageLinks"))?;

                Ok(GoogleBooks::default()
                    .title(title)
                    .isbns(isbns)
                    .authors(authors)
                    .description(description)
                    .publisher(publisher)
                    .publication_date(publication_date)
                    .language(language)
                    .pages(page_count)
                    .tags(tags)
                    .cover_images(cover_images))
            }
        }

        deserializer.deserialize_struct("GoogleBooks", FIELDS, GoogleBooksVisitor)
    }
}

mod test {
    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn parses_from_isbn() {
        use super::GoogleBooks;
        use isbn::Isbn;
        use log::info;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();
        let resp = GoogleBooks::from_isbn(&isbn).await;
        info!("Response: {:#?}", resp);
        assert!(resp.is_ok())
    }
}
