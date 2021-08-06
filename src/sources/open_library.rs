use crate::interface::recon::ReconError;
use core::fmt;
use log::info;
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct OpenLibrary {
    // "isbn_10" | "isbn_13"
    isbn:             Vec<String>,
    // "title"
    title:            String,
    // "authors"/["name"]
    author:           Option<Vec<String>>,
    // ""
    description:      Option<String>,
    // "publishers"
    publisher:        Vec<String>,
    // "publish_date"
    publication_date: String,
    // "language"["key"]
    language:         Vec<String>,
    // "number_of_pages"
    page_count:       u16,
    // "subjects"
    tag:              Option<Vec<String>>,
    // "thumbnail_url"
    cover_image:      Vec<String>,
}

impl OpenLibrary {
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

    pub fn author(mut self, author: Option<Vec<String>>) -> Self {
        self.author = author;
        info!("Field `author` is set to: {:#?}", self.author);
        self
    }

    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        info!("Field `description` is set to: {:#?}", self.description);
        self
    }

    pub fn publisher(mut self, publisher: Vec<String>) -> Self {
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

    pub fn language(mut self, language: Vec<String>) -> Self {
        self.language = language;
        info!("Field `language` is set to: {:#?}", self.language);
        self
    }

    pub fn page_count(mut self, page_count: u16) -> Self {
        self.page_count = page_count;
        info!("Field `page_count` is set to: {:#?}", self.page_count);
        self
    }

    pub fn tag(mut self, tag: Option<Vec<String>>) -> Self {
        self.tag = tag;
        info!("Field `tag` is set to: {:#?}", self.tag);
        self
    }

    pub fn cover_image(mut self, cover_image: Vec<String>) -> Self {
        self.cover_image = cover_image;
        info!("Field `cover_image` is set to: {:#?}", self.cover_image);
        self
    }
}

impl OpenLibrary {
    pub async fn from_isbn(isbn: &isbn::Isbn) -> Result<Self, ReconError> {
        let isbn_key = format!("ISBN:{}", urlencoding::encode(&isbn.to_string()));
        let req = format!(
            "https://openlibrary.org/api/books?bibkeys={}&jscmd=details&format=json",
            isbn_key
        );

        info!("ISBN: {:#?}", isbn);
        info!("Request: {:#?}", req);

        serde_json::from_value::<serde_json::Value>(
            reqwest::get(req)
                .await
                .map_err(ReconError::Connection)?
                .json::<serde_json::Value>()
                .await
                .map_err(ReconError::Connection)?[isbn_key]["details"]
                .take(),
        )
        .map_err(ReconError::JSONParse)
        .map(|v| serde_json::from_value::<Self>(v).map_err(ReconError::JSONParse))?
    }
}

impl<'de> Deserialize<'de> for OpenLibrary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Title,
            Author,
            Publishers,
            PublishDate,
            Description,
            Isbn10,
            Isbn13,
            NumberOfPages,
            Tag,
            CoverImages,
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
            "tag",
            "imageLinks",
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
                        formatter.write_str("Any of `OpenLibrary` fields.")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "title" => Ok(Field::Title),
                            "authors" => Ok(Field::Author),
                            "publishers" => Ok(Field::Publishers),
                            "publish_date" => Ok(Field::PublishDate),
                            "description" => Ok(Field::Description),
                            "isbn_10" => Ok(Field::Isbn10),
                            "isbn_13" => Ok(Field::Isbn13),
                            "number_of_pages" => Ok(Field::NumberOfPages),
                            "categories" => Ok(Field::Tag),
                            "imageLinks" => Ok(Field::CoverImages),
                            "languages" => Ok(Field::Languages),
                            _ => Ok(Field::Ignore),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct OpenLibraryVisitor;

        impl<'de> Visitor<'de> for OpenLibraryVisitor {
            type Value = OpenLibrary;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct OpenLibrary")
            }

            fn visit_map<V>(self, mut map: V) -> Result<OpenLibrary, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut isbn_10 = None;
                let mut isbn_13 = None;
                let mut title = None;
                let mut author = None;
                let description = None;
                let mut publishers = None;
                let mut publish_date = None;
                let mut languages = None;
                let mut number_of_pages = None;
                // let mut tag = None;
                // let mut cover_image = None;

                // Helper functions.
                //
                // Functions named after fields such as `parse_to_languages`
                // are unique function that isn't used by any other field.
                //
                // Generic function such as `parse_to_string` are used by multiple fields.
                let parse_to_u16 = |v: u16| Some(v);
                let parse_to_string = |string: String| Some(string);
                let parse_to_vec = |vec: Vec<String>| Some(vec);
                let parse_to_opt_vec = |vec: Option<Vec<String>>| vec;
                let parse_to_languages = |vec_of_map: Vec<HashMap<String, String>>| {
                    Some(
                        vec_of_map
                            .into_iter()
                            .map(|h| h.into_iter().map(|(_, v)| v).collect::<Vec<String>>())
                            .flatten()
                            .map(|mut s| s.split_off(s.len() - 3)) // "/languages/eng" -> "eng"
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

                        Field::Isbn10 => {
                            if isbn_10.is_some() {
                                return Err(de::Error::duplicate_field("isbn_10"));
                            }
                            isbn_10 = parse_to_vec(map.next_value()?);
                        }

                        Field::Isbn13 => {
                            if isbn_13.is_some() {
                                return Err(de::Error::duplicate_field("isbn_13"));
                            }
                            isbn_13 = parse_to_vec(map.next_value()?);
                        }

                        Field::Author => {
                            if author.is_some() {
                                return Err(de::Error::duplicate_field("authors"));
                            }
                            author = parse_to_opt_vec(map.next_value()?);
                        }

                        Field::Publishers => {
                            if publishers.is_some() {
                                return Err(de::Error::duplicate_field("publishers"));
                            }
                            publishers = parse_to_vec(map.next_value()?);
                        }

                        Field::PublishDate => {
                            if publish_date.is_some() {
                                return Err(de::Error::duplicate_field("publish_date"));
                            }
                            publish_date = parse_to_string(map.next_value()?);
                        }

                        Field::Languages => {
                            if languages.is_some() {
                                return Err(de::Error::duplicate_field("languages"));
                            }
                            languages = parse_to_languages(map.next_value()?);
                        }

                        Field::NumberOfPages => {
                            if number_of_pages.is_some() {
                                return Err(de::Error::duplicate_field("number_of_pages"));
                            }
                            number_of_pages = parse_to_u16(map.next_value()?);
                        }

                        _ => {}
                    }
                }

                let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
                let author = author;
                let isbn = match (isbn_10, isbn_13) {
                    (Some(mut isbn_10), Some(isbn_13)) => {
                        isbn_10.extend(isbn_13);
                        isbn_10
                    }
                    (Some(isbn_10), None) => isbn_10,
                    (None, Some(isbn_13)) => isbn_13,
                    _ => vec![],
                };
                let publisher = publishers.ok_or_else(|| de::Error::missing_field("publisher"))?;
                let description = description;
                let publish_date =
                    publish_date.ok_or_else(|| de::Error::missing_field("publish_date"))?;
                let languages = languages.ok_or_else(|| de::Error::missing_field("languages"))?;
                let number_of_pages =
                    number_of_pages.ok_or_else(|| de::Error::missing_field("number_of_pages"))?;

                Ok(OpenLibrary::default()
                    .title(title)
                    .isbn(isbn)
                    .publisher(publisher)
                    .author(author)
                    .description(description)
                    .publication_date(publish_date)
                    .language(languages)
                    .page_count(number_of_pages))
            }
        }

        deserializer.deserialize_struct("OpenLibrary", FIELDS, OpenLibraryVisitor)
    }
}

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
}
