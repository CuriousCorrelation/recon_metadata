use crate::metadata::Metadata;
use crate::recon::ReconError;
use crate::util::translater;
use isbn2::Isbn;
use log::debug;
use serde::de;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

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

                Ok(OpenLibrary(Metadata {
                    isbn10:           translater::openlibrary_isbn10(&identifiers),
                    isbn13:           translater::openlibrary_isbn13(&identifiers),
                    title:            translater::string(title),
                    author:           translater::vec_hashmap_field(authors, "name"),
                    description:      translater::empty(),
                    page_count:       translater::number(number_of_pages),
                    publisher:        translater::vec_hashmap_field(publishers, "name"),
                    publication_date: translater::publication_date(publish_date),
                    language:         translater::empty(),
                    cover_image:      translater::hashmap(cover),
                    tag:              translater::vec_hashmap_field_split(subjects, "name"),
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
    pub async fn from_isbn(isbn: &isbn2::Isbn) -> Result<Metadata, ReconError> {
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

        let metadata = response.into_iter().map(|(_, v)| v.0).next();

        Ok(metadata.unwrap_or_default())
    }

    pub async fn from_description(description: &str) -> Result<Vec<Isbn>, ReconError> {
        let req = format!(
            "https://openlibrary.org/search.json?q={}",
            urlencoding::encode(description)
        );

        debug!("Description: {:#?}", &description);
        debug!("Request: {:#?}", &req);

        #[derive(Deserialize, Debug)]
        struct Docs {
            docs: Vec<OLIsbn>,
        }

        #[derive(Deserialize, Debug)]
        struct OLIsbn {
            isbn: Option<Vec<String>>,
        }

        let response = reqwest::get(req)
            .await
            .map_err(ReconError::Connection)?
            .json::<Docs>()
            .await
            .map_err(ReconError::Connection)?;

        debug!("Response: {:#?}", &response);

        let mut isbns = response
            .docs
            .iter()
            .map(|h| h.isbn.as_ref().map(|v| v.get(0)))
            .flatten()
            .flatten()
            .collect::<Vec<_>>();

        isbns.truncate(3); // first 3 results

        let mut isbn_list = Vec::new();

        for isbn in isbns {
            isbn_list.push(Isbn::from_str(isbn));
        }

        let isbn_list = isbn_list
            .into_iter()
            .filter(|isbn| isbn.is_ok())
            .map(|isbn| isbn.unwrap())
            .collect::<Vec<_>>();

        Ok(isbn_list)
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
        use isbn2::Isbn;
        use log::debug;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();
        let resp = OpenLibrary::from_isbn(&isbn).await;
        debug!("Response: {:#?}", resp);
        assert!(resp.is_ok())
    }

    #[tokio::test]
    async fn parses_from_description() {
        use super::OpenLibrary;
        use log::debug;

        init_logger();

        let description = "This is how you lose the time war";
        let resp = OpenLibrary::from_description(&description).await;
        debug!("Response: {:#?}", resp);
        assert!(resp.is_ok())
    }
}
