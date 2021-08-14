use crate::metadata::Metadata;
use crate::recon::ReconError;
use crate::util::translater;
use log::debug;
use serde::de;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct OpenLibrary(Metadata);

impl<'de> Deserialize<'de> for OpenLibrary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Identifiers,
            Title,
            Authors,
            NumberOfPages,
            Publishers,
            PublishDate,
            Subjects,
            Cover,
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
                    "identifiers" => Ok(Field::Identifiers),
                    "title" => Ok(Field::Title),
                    "authors" => Ok(Field::Authors),
                    "number_of_pages" => Ok(Field::NumberOfPages),
                    "publishers" => Ok(Field::Publishers),
                    "publish_date" => Ok(Field::PublishDate),
                    "subjects" => Ok(Field::Subjects),
                    "cover" => Ok(Field::Cover),
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
            marker:   PhantomData<OpenLibrary>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de> de::Visitor<'de> for Visitor<'de> {
            type Value = OpenLibrary;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "struct OpenLibrary")
            }

            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut identifiers = None;
                let mut title = None;
                let mut authors = None;
                let mut number_of_pages = None;
                let mut publishers = None;
                let mut publish_date = None;
                let mut subjects = None;
                let mut cover = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Identifiers => {
                            if identifiers.is_some() {
                                return Err(de::Error::duplicate_field("identifiers"));
                            }
                            identifiers = Some(map.next_value()?);
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
                        Field::NumberOfPages => {
                            if number_of_pages.is_some() {
                                return Err(de::Error::duplicate_field("number_of_pages"));
                            }
                            number_of_pages = Some(map.next_value::<u16>()?);
                        }
                        Field::Publishers => {
                            if publishers.is_some() {
                                return Err(de::Error::duplicate_field("publishers"));
                            }
                            publishers = Some(map.next_value()?);
                        }
                        Field::PublishDate => {
                            if publish_date.is_some() {
                                return Err(de::Error::duplicate_field("publish_date"));
                            }
                            publish_date = Some(map.next_value()?);
                        }
                        Field::Subjects => {
                            if subjects.is_some() {
                                return Err(de::Error::duplicate_field("subjects"));
                            }
                            subjects = Some(map.next_value()?);
                        }
                        Field::Cover => {
                            if cover.is_some() {
                                return Err(de::Error::duplicate_field("cover"));
                            }
                            cover = Some(map.next_value()?);
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

                let identifiers =
                    identifiers.ok_or_else(|| de::Error::missing_field("identifiers"))?;
                let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
                let authors = authors.ok_or_else(|| de::Error::missing_field("authors"))?;
                let number_of_pages =
                    number_of_pages.ok_or_else(|| de::Error::missing_field("number_of_pages"))?;
                let publishers =
                    publishers.ok_or_else(|| de::Error::missing_field("publishers"))?;
                let publish_date =
                    publish_date.ok_or_else(|| de::Error::missing_field("publish_date"))?;
                let subjects = subjects.ok_or_else(|| de::Error::missing_field("subjects"))?;
                let cover = cover.ok_or_else(|| de::Error::missing_field("cover"))?;

                Ok(OpenLibrary(Metadata {
                    isbns:             translater::openlibrary_isbn(identifiers),
                    titles:            translater::string(title),
                    authors:           translater::vec_hashmap_field(authors, "name"),
                    descriptions:      translater::empty(),
                    page_count:        translater::number(number_of_pages),
                    publishers:        translater::vec_hashmap_field(publishers, "name"),
                    publication_dates: translater::publication_date(publish_date),
                    languages:         translater::empty(),
                    cover_images:      translater::hashmap(cover),
                    tags:              translater::vec_hashmap_field_split_lowercase(
                        subjects, "name",
                    ),
                }))
            }
        }
        const FIELDS: &[&str] = &[
            "identifiers",
            "title",
            "authors",
            "number_of_pages",
            "publishers",
            "publish_date",
            "subjects",
            "cover",
        ];
        Deserializer::deserialize_struct(
            deserializer,
            "OpenLibrary",
            FIELDS,
            Visitor {
                marker:   PhantomData::<OpenLibrary>,
                lifetime: PhantomData,
            },
        )
    }
}

impl OpenLibrary {
    pub async fn from_isbn(isbn: &isbn::Isbn) -> Result<Metadata, ReconError> {
        let req = format!(
            "https://openlibrary.org/api/books?bibkeys=ISBN:{}&jscmd=data&format=json",
            urlencoding::encode(&isbn.to_string())
        );

        debug!("ISBN: {:#?}", &isbn);
        debug!("Request: {:#?}", &req);

        let response = reqwest::get(req)
            .await
            .map_err(ReconError::Connection)?
            .json::<HashMap<String, OpenLibrary>>()
            .await
            .map_err(ReconError::Connection)?;

        debug!("Response: {:#?}", &response);

        let metadata = response
            .into_iter()
            .map(|(_, v)| v.0)
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
        use super::OpenLibrary;
        use isbn::Isbn;
        use log::debug;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();
        let resp = OpenLibrary::from_isbn(&isbn).await;
        debug!("Response: {:#?}", resp);
        assert!(resp.is_ok())
    }
}
