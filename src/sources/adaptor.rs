/*! Apator functions
Functions named after fields such as `parse_number_of_pages`
are unique function that isn't used by any other field.

Generic function such as `parse_string` are used by multiple fields.

API specific function essentially do similar things but for different JSON structures.
`parse_google_books_isbn` vs `parse_open_library_isbn`

These functions are responsible for converting JSON field
into corrosponding serde type
and those serde types into `Self`.
Essentially JSON -> Serde -> Metadata
For example - book title
JSON - "title"
Serde - String
Metadata - Vec<Result<String, ReconError>>
So JSON -> Serde is `| string | -> Some(sring)`
Serde -> Metadata is `| string | -> Some(Ok(string))`

JSON -> Serde return type should be `Option< ... >`
Serde -> Metadata almost always `Option<Result< ... , ... >>`
*/

use crate::interface::recon::ReconError;
use chrono::{NaiveDate, ParseResult};
use isbn::Isbn;
use log::debug;
use std::{collections::HashMap, str::FromStr};

pub(crate) fn parse_string(string: String) -> Option<Result<String, ReconError>> {
    debug!(
        "`fn parse_string` arg(s) `string` is: {:#?}, expecting `String`",
        string
    );

    Some(Ok(string))
}

pub(crate) fn parse_languages(
    languages: Vec<HashMap<String, String>>,
) -> Option<Vec<Result<String, ReconError>>> {
    debug!(
        "`fn parse_languages` arg(s) `languages` is: {:#?}, expecting `Vec<HashMap<String, String>>,
`",
        languages
    );

    Some(
        languages
            .into_iter()
            .map(|h| {
                h.into_iter()
                    .map(|(_, mut v)| {
                        Ok(
                            v.split_off(v.len() - 3), // "/language/eng" -> "eng"
                        )
                    })
                    .collect::<Vec<Result<String, ReconError>>>()
            })
            .flatten()
            .collect::<Vec<Result<String, ReconError>>>(),
    )
}

pub(crate) fn parse_covers(covers: Vec<u32>) -> Option<Vec<Result<String, ReconError>>> {
    debug!(
        "`fn parse_covers` arg(s) `covers` is: {:#?}, expecting `Vec<u32>`",
        covers
    );

    let possible_sizes = ["S", "M", "L"];

    Some(
        covers
            .into_iter()
            .map(|cover: u32| {
                possible_sizes
                    .iter()
                    .map(|size| {
                        Ok(format!(
                            "https://covers.openlibrary.org/b/id/{}-{}.jpg",
                            cover.to_string(),
                            size
                        ))
                    })
                    .collect::<Vec<Result<String, ReconError>>>()
            })
            .flatten()
            .collect::<Vec<Result<String, ReconError>>>(),
    )
}

pub(crate) fn parse_number_of_pages(number_of_pages: u16) -> Option<Result<u16, ReconError>> {
    debug!(
        "`fn parse_number_of_pages` arg(s) `number_of_pages` is: {:#?}, expecting `u16`",
        number_of_pages
    );

    Some(Ok(number_of_pages))
}

pub(crate) fn parse_publish_date(
    publish_date: Option<String>,
) -> Option<Result<NaiveDate, ReconError>> {
    debug!(
        "`fn parse_publish_date` arg(s) `publish_date` is: {:#?}, expecting `String`",
        publish_date
    );

    let possible_formats = ["%Y", "%B %Y", "%B, %d %Y", "%Y-%m-%d"]
        .iter_mut()
        .map(|fmt| {
            publish_date
                .as_ref()
                .map(|publish_date| NaiveDate::parse_from_str(publish_date, fmt))
        })
        .flatten()
        .collect::<Vec<ParseResult<NaiveDate>>>()
        .pop();

    possible_formats.map(|r| r.map_err(ReconError::DateParse))
}

pub(crate) fn parse_vec(vecs: Vec<String>) -> Option<Vec<Result<String, ReconError>>> {
    debug!(
        "`fn parse_vec` arg(s) `vecs` is: {:#?}, expecting `Vec<String>`",
        vecs
    );

    Some(
        vecs.into_iter()
            .map(Ok)
            .collect::<Vec<Result<String, ReconError>>>(),
    )
}

pub(crate) fn parse_open_library_isbn(
    isbn: Option<Vec<String>>,
) -> Option<Vec<Result<Isbn, ReconError>>> {
    debug!(
        "`fn parse_open_library_isbn` arg(s) `isbn` is: {:#?}, expecting `Option<Vec<String>>,
`",
        isbn
    );

    isbn.map(|isbn| {
        isbn.into_iter()
            .map(|s| Isbn::from_str(&s).map_err(ReconError::ISBNParse))
            .collect::<Vec<Result<Isbn, ReconError>>>()
    })
}

pub(crate) fn parse_image_links(
    image_links: HashMap<String, String>,
) -> Option<Vec<Result<String, ReconError>>> {
    debug!(
        "`fn parse_image_links` arg(s) `image_links` is: {:#?}, expecting `HashMap<String, String>,
`",
        image_links
    );

    Some(
        image_links
            .into_iter()
            .map(|(_, v)| Ok(v))
            .collect::<Vec<Result<String, ReconError>>>(),
    )
}

pub(crate) fn parse_page_count(page_count: u16) -> Option<Result<u16, ReconError>> {
    debug!(
        "`fn parse_page_count` arg(s) `page_count` is: {:#?}, expecting `u16`",
        page_count
    );

    Some(Ok(page_count))
}

pub(crate) fn parse_published_date(
    published_date: String,
) -> Option<Result<NaiveDate, ReconError>> {
    debug!(
        "`fn parse_published_date` arg(s) `published_date` is: {:#?}, expecting `String,
`",
        published_date
    );

    Some(NaiveDate::parse_from_str(&published_date, "%Y-%m-%d").map_err(ReconError::DateParse))
}

pub(crate) fn parse_google_books_isbn(
    mut isbn: Vec<HashMap<String, String>>,
) -> Option<Vec<Result<Isbn, ReconError>>> {
    debug!(
        "`fn parse_google_books_isbn` arg(s) `isbn` is: {:#?}, expecting `Vec<HashMap<String, String>>,
`",
        isbn
    );

    Some(
        isbn.iter_mut()
            .map(|h| h.remove("identifier"))
            .flatten()
            .map(|s| Isbn::from_str(&s).map_err(ReconError::ISBNParse))
            .collect::<Vec<Result<Isbn, ReconError>>>(),
    )
}

pub(crate) fn parse_authors(
    authors: Option<Vec<HashMap<String, String>>>,
) -> Option<Vec<Result<String, ReconError>>> {
    debug!(
        "`fn parse_authors` arg(s) `authors` is: {:#?}, expecting `Option<Vec<HashMap<String, String>>>,
`",
        authors
    );

    authors.map(|authors| {
        authors
            .into_iter()
            .map(|mut h| h.remove("name"))
            .flatten()
            .map(Ok)
            .collect::<Vec<Result<String, ReconError>>>()
    })
}

pub(crate) fn parse_works(
    works: Vec<HashMap<String, String>>,
) -> Option<Vec<Result<String, ReconError>>> {
    debug!(
        "`fn parse_works` arg(s) `works` is: {:#?}, expecting `Option<Vec<HashMap<String, String>>>,
`",
        works
    );

    Some(
        works
            .into_iter()
            .map(|h| {
                h.into_iter()
                    .map(|(_, v)| Ok(v))
                    .collect::<Vec<Result<String, ReconError>>>()
            })
            .flatten()
            .collect::<Vec<Result<String, ReconError>>>(),
    )
}
