use crate::metadata::Metadata;
use crate::recon::ReconError;
use crate::util::translater;
use log::debug;
use serde::de;
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct GoogleBooks(Metadata);

impl<'de> Deserialize<'de> for GoogleBooks {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            IndustryIdentifiers,
            Title,
            Authors,
            Description,
            PageCount,
            Publisher,
            PublishedDate,
            Categories,
            ImageLinks,
            Language,
            Ignore,
        }
        struct FieldVisitor;
        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = Field;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "field identifier")
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "industryIdentifiers" => Ok(Field::IndustryIdentifiers),
                    "title" => Ok(Field::Title),
                    "authors" => Ok(Field::Authors),
                    "description" => Ok(Field::Description),
                    "pageCount" => Ok(Field::PageCount),
                    "publisher" => Ok(Field::Publisher),
                    "publishedDate" => Ok(Field::PublishedDate),
                    "categories" => Ok(Field::Categories),
                    "imageLinks" => Ok(Field::ImageLinks),
                    "language" => Ok(Field::Language),
                    _ => Ok(Field::Ignore),
                }
            }
        }
        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }
        struct Visitor<'de> {
            marker:   PhantomData<GoogleBooks>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de> de::Visitor<'de> for Visitor<'de> {
            type Value = GoogleBooks;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "struct GoogleBooks")
            }

            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut industry_identifiers = None;
                let mut title = None;
                let mut authors = None;
                let mut description = None;
                let mut page_count = None;
                let mut publisher = None;
                let mut published_date = None;
                let mut categories = None;
                let mut image_links = None;
                let mut language = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::IndustryIdentifiers => {
                            if industry_identifiers.is_some() {
                                return Err(de::Error::duplicate_field("industryIdentifiers"));
                            }
                            industry_identifiers = Some(map.next_value()?);
                        }
                        Field::Title => {
                            if title.is_some() {
                                return Err(de::Error::duplicate_field("title"));
                            }
                            title = Some(map.next_value()?);
                        }
                        Field::Authors => {
                            if authors.is_some() {
                                return Err(de::Error::duplicate_field("authors"));
                            }
                            authors = Some(map.next_value()?);
                        }
                        Field::Description => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }
                            description = Some(map.next_value()?);
                        }
                        Field::PageCount => {
                            if page_count.is_some() {
                                return Err(de::Error::duplicate_field("pageCount"));
                            }
                            page_count = Some(map.next_value::<u16>()?);
                        }
                        Field::Publisher => {
                            if publisher.is_some() {
                                return Err(de::Error::duplicate_field("publisher"));
                            }
                            publisher = Some(map.next_value()?);
                        }
                        Field::PublishedDate => {
                            if published_date.is_some() {
                                return Err(de::Error::duplicate_field("publishedDate"));
                            }
                            published_date = Some(map.next_value()?);
                        }
                        Field::Categories => {
                            if categories.is_some() {
                                return Err(de::Error::duplicate_field("categories"));
                            }
                            categories = Some(map.next_value()?);
                        }
                        Field::ImageLinks => {
                            if image_links.is_some() {
                                return Err(de::Error::duplicate_field("imageLinks"));
                            }
                            image_links = Some(map.next_value()?);
                        }
                        Field::Language => {
                            if language.is_some() {
                                return Err(de::Error::duplicate_field("language"));
                            }
                            language = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = match A::next_value::<de::IgnoredAny>(&mut map) {
                                Ok(val) => val,
                                Err(err) => {
                                    return Err(err);
                                }
                            };
                        }
                    }
                }

                let industry_identifiers = industry_identifiers
                    .ok_or_else(|| de::Error::missing_field("industryIdentifiers"))?;
                let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
                let authors = authors.ok_or_else(|| de::Error::missing_field("authors"))?;
                let description =
                    description.ok_or_else(|| de::Error::missing_field("description"))?;
                let page_count = page_count.ok_or_else(|| de::Error::missing_field("pageCount"))?;
                let publisher = publisher.ok_or_else(|| de::Error::missing_field("publisher"))?;
                let published_date =
                    published_date.ok_or_else(|| de::Error::missing_field("publishedDate"))?;
                let categories =
                    categories.ok_or_else(|| de::Error::missing_field("categories"))?;
                let image_links =
                    image_links.ok_or_else(|| de::Error::missing_field("imageLinks"))?;
                let language = language.ok_or_else(|| de::Error::missing_field("language"))?;

                Ok(GoogleBooks(Metadata {
                    isbns:             translater::googlebooks_isbn(industry_identifiers),
                    titles:            translater::string(title),
                    authors:           translater::vec(authors),
                    descriptions:      translater::string(description),
                    page_count:        translater::number(page_count),
                    publishers:        translater::string(publisher),
                    publication_dates: translater::publication_date(published_date),
                    languages:         translater::string(language),
                    tags:              translater::vec(categories),
                    cover_images:      translater::hashmap(image_links),
                }))
            }
        }
        const FIELDS: &[&str] = &[
            "industryIdentifiers",
            "title",
            "authors",
            "description",
            "pageCount",
            "publisher",
            "publishedDate",
            "categories",
            "imageLinks",
            "language",
        ];
        Deserializer::deserialize_struct(
            deserializer,
            "GoogleBooks",
            FIELDS,
            Visitor {
                marker:   PhantomData::<GoogleBooks>,
                lifetime: PhantomData,
            },
        )
    }
}

impl GoogleBooks {
    pub async fn from_isbn(isbn: &isbn::Isbn) -> Result<Metadata, ReconError> {
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
            #[serde(rename = "volumeInfo")]
            volume_info: GoogleBooks,
        }

        let response = reqwest::get(req)
            .await
            .map_err(ReconError::Connection)?
            .json::<Items>()
            .await
            .map_err(ReconError::Connection)?;

        debug!("Response: {:#?}", &response);

        let metadata = response
            .items
            .into_iter()
            .map(|v| v.volume_info.0)
            .collect::<Vec<Metadata>>()
            .remove(0);

        Ok(metadata)
    }
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
}
