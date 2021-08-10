use crate::recon::ReconError;
use crate::sources::adaptor;
use crate::types::base::{
    Authors, CoverImages, Descriptions, Languages, PageCount, PublicationDates, Publishers, Tags,
    Titles,
};
use crate::{metadata::Metadata, types::base::ISBNs};
use core::fmt;
use log::debug;
use serde::{
    de::{self, Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, Default)]
pub struct OpenLibrary(Metadata);

impl OpenLibrary {
    pub async fn from_isbn(isbn: &isbn::Isbn) -> Result<Metadata, ReconError> {
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

        let ISBNResponse(metadata) =
            serde_json::from_value(response[isbn_key].take()).map_err(ReconError::JSONParse)?;

        Ok(metadata)
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
        Publishers,
        PublishDate,
        Description,
        Identifiers,
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
        "publish_date",
        "description",
        "identifiers",
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
                        "publish_date" => Ok(Field::PublishDate),
                        "description" => Ok(Field::Description),
                        "identifiers" => Ok(Field::Identifiers),
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
            let mut identifiers = None;
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

                        title = adaptor::parse_string(map.next_value()?);
                    }

                    Field::Identifiers => {
                        if identifiers.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "identifiers",
                            )))
                            .map_err(V::Error::custom);
                        }

                        identifiers = adaptor::parse_open_library_identifiers(map.next_value()?);
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

                    Field::NumberOfPages => {
                        if number_of_pages.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "number_of_pages",
                            )))
                            .map_err(V::Error::custom);
                        }

                        number_of_pages = adaptor::parse_number_of_pages(map.next_value()?);
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

                        publishers = adaptor::parse_vec_hashmap_field(map.next_value()?, "name");
                    }

                    Field::PublishDate => {
                        if publish_date.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "publish_date",
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

                        languages = adaptor::parse_vec_hashmap_field(map.next_value()?, "name");
                    }

                    Field::Subjects => {
                        if subjects.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field(
                                "subjects",
                            )))
                            .map_err(V::Error::custom);
                        }

                        subjects = adaptor::parse_vec_hashmap_field(map.next_value()?, "name");
                    }

                    Field::Cover => {
                        if cover.is_some() {
                            return Err(ReconError::JSONParse(de::Error::duplicate_field("cover")))
                                .map_err(V::Error::custom);
                        }

                        cover = adaptor::parse_hashmap(map.next_value()?);
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
            let titles: Titles = vec![title];

            let authors: Authors = match authors {
                Some(authors) => authors,
                None => vec![],
            };

            let isbns: ISBNs =
                identifiers.ok_or_else(|| de::Error::missing_field("identifiers"))?;

            let descriptions: Descriptions = match description {
                Some(description) => vec![description],
                None => vec![],
            };

            let publishers: Publishers =
                publishers.ok_or_else(|| de::Error::missing_field("publishers"))?;

            let publication_dates: PublicationDates = match publish_date {
                Some(publish_date) => vec![publish_date],
                None => vec![],
            };

            let languages: Languages = match languages {
                Some(languages) => languages,
                None => vec![],
            };

            let number_of_pages: Result<u16, ReconError> =
                number_of_pages.ok_or_else(|| de::Error::missing_field("number_of_pages"))?;
            let page_count: PageCount = vec![number_of_pages];

            let cover_images: CoverImages = match cover {
                Some(cover) => cover,
                None => vec![],
            };

            let tags: Tags = match subjects {
                Some(subjects) => subjects,
                None => vec![],
            };

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
}
