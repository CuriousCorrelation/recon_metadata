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
    isbns:            Vec<String>,
    title:            String,
    authors:          Vec<String>,
    description:      String,
    publisher:        String,
    publication_date: String,
    language:         String,
    page_count:       u16,
    tags:             Vec<String>,
    cover_images:     Vec<String>,
}

impl GoogleBooks {
    pub fn isbns(mut self, isbns: Vec<String>) -> Self {
        self.isbns = isbns;
        info!("Field `isbns` is set to: {:#?}", self.isbns);
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
                // let mut authors = None;
                // let mut description = None;
                // let mut publishers = None;
                // let mut publication_dates = None;
                // let mut languages = None;
                // let mut pages = None;
                // let mut tags = None;
                // let mut cover_images = None;

                let parse_title = |title: String| Some(title);
                let parse_isbns = |isbns: Vec<HashMap<String, String>>| {
                    Some(
                        isbns
                            .into_iter()
                            .map(|h| h.values().cloned().collect::<Vec<String>>())
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
                            title = parse_title(map.next_value()?);
                        }

                        Field::IndustryIdentifiers => {
                            if isbns.is_some() {
                                return Err(de::Error::duplicate_field("industryIdentifiers"));
                            }
                            isbns = parse_isbns(map.next_value::<Vec<HashMap<String, String>>>()?);
                        }

                        _ => {}
                    }
                }

                let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
                let isbns = isbns.ok_or_else(|| de::Error::missing_field("industryIdentifiers"))?;

                Ok(GoogleBooks::default().title(title).isbns(isbns))
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
