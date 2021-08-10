/*! Transforms API response by `serde` into `Metadata` elements.

Functions responsible for converting JSON field
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

Functions named after fields such as `parse_number_of_pages`
are unique functions not used by any other field.

Generic function such as `parse_string` may be used to parse many different fields.

API specific function essentially do similar things but for different JSON structures.
`parse_google_books_isbn` vs `parse_open_library_isbn`
*/

use crate::{
    recon::ReconError,
    types::base::{self, ISBNs, Numeric},
};
use chrono::NaiveDate;
use isbn::Isbn;
use log::debug;
use std::{collections::HashMap, str::FromStr};

/// Transforms [`String`] type into [`base::Generic`]
///
/// Essentially a wrapper around [`String`] type
pub(crate) fn parse_string(string: String) -> Option<base::Generic> {
    debug!(
        "`fn parse_string` arg(s) `string` is: {:#?}, expecting `String`",
        string
    );

    Some(Ok(string))
}

/// Transforms [`u16`] type into [`base::Numeric`]
///
/// Essentially a wrapper around [`u16`] type
pub(crate) fn parse_number_of_pages(number_of_pages: u16) -> Option<Numeric> {
    debug!(
        "`fn parse_number_of_pages` arg(s) `number_of_pages` is: {:#?}, expecting `u16`",
        number_of_pages
    );

    Some(Ok(number_of_pages))
}

/// Transforms [`crate::source::OpenLibrary`] source's "publish_date" key value into [`base::Date`]
///
/// Objects can have varying formats.
/// ```
/// "2011"
/// "March 2009"
/// "July 16, 2019"
/// ```
pub(crate) fn parse_publish_date(publish_date: Option<String>) -> Option<base::Date> {
    debug!(
        "`fn parse_publish_date` arg(s) `publish_date` is: {:#?}, expecting `String`",
        publish_date
    );

    // Possible format only include complete dates.
    let possible_formats = ["%B %d, %Y", "%Y-%m-%d", "%B, %d %Y"];

    match publish_date {
        Some(s) => possible_formats
            .iter()
            .map(|fmt| NaiveDate::parse_from_str(&s, fmt))
            .filter(|date| date.is_ok())
            .map(|date| date.map_err(ReconError::DateParse))
            .collect::<Vec<base::Date>>()
            .pop(),
        None => None,
    }
}

/// Transforms [`Vec<String>`] into [`base::Generic`]
pub(crate) fn parse_vec(vecs: Vec<String>) -> Option<Vec<base::Generic>> {
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

pub(crate) fn parse_hashmap(
    hashmap: HashMap<String, String>,
) -> Option<Vec<Result<String, ReconError>>> {
    debug!(
        "`fn parse_hashmap` arg(s) `hashmap` is: {:#?}, expecting `HashMap<String, String>,
`",
        hashmap
    );

    Some(
        hashmap
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

pub(crate) fn parse_vec_hashmap_field(
    vec_hashmap: Option<Vec<HashMap<String, String>>>,
    field: &str,
) -> Option<Vec<Result<String, ReconError>>> {
    debug!(
        "`fn parse_vec_hashmap` arg(s) `vec_hashmap` is: {:#?}, expecting `Option<Vec<HashMap<String, String>>>,
`, `field` is: {:#?}",
        vec_hashmap,
        field
    );

    vec_hashmap.map(|vec_hashmap| {
        vec_hashmap
            .into_iter()
            .map(|mut h| h.remove(field))
            .flatten()
            .map(Ok)
            .collect::<Vec<Result<String, ReconError>>>()
    })
}

pub(crate) fn parse_open_library_identifiers(
    hashmap_vec: HashMap<String, Vec<String>>,
) -> Option<ISBNs> {
    debug!(
        "`fn parse_open_library_identifiers` arg(s) `hashmap_vec` is: {:#?}, expecting `HashMap<String, Vec<String>>`",
        hashmap_vec
        );

    Some(
        hashmap_vec
            .into_iter()
            .filter(|(k, _)| k == "isbn_10" || k == "isbn_13")
            .map(|(_, v)| v)
            .flatten()
            .map(|isbn| Isbn::from_str(&isbn).map_err(ReconError::ISBNParse))
            .collect(),
    )
}
