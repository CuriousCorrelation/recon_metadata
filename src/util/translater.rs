use crate::{
    recon::ReconError,
    types::base::{Generic, ISBNs, PublicationDates, Titles},
};
use chrono::NaiveDate;
use isbn::Isbn;
use std::{collections::HashMap, str::FromStr};

pub(crate) fn number(n: u16) -> Vec<Result<u16, ReconError>> {
    vec![Ok(n)]
}

pub(crate) fn empty() -> Vec<Generic> {
    vec![]
}

pub(crate) fn string(s: &str) -> Titles {
    vec![Ok(s.to_owned())]
}

pub(crate) fn hashmap(hashmap: HashMap<&str, &str>) -> Vec<Generic> {
    hashmap.into_iter().map(|(_, v)| Ok(v.to_owned())).collect()
}

pub(crate) fn vec(vec: Vec<&str>) -> Vec<Generic> {
    vec.into_iter().map(|v| Ok(v.to_owned())).collect()
}

pub(crate) fn vec_hashmap_field(
    vec_hashmap: Vec<HashMap<&str, &str>>,
    field: &str,
) -> Vec<Generic> {
    vec_hashmap
        .into_iter()
        .map(|mut h| h.remove(field))
        .flatten()
        .map(|s| Ok(s.to_owned()))
        .collect()
}

pub(crate) fn vec_hashmap_field_split(
    vec_hashmap: Vec<HashMap<&str, &str>>,
    field: &str,
) -> Vec<Generic> {
    vec_hashmap
        .into_iter()
        .map(|mut h| h.remove(field))
        .flatten()
        .map(|s| {
            s.split(',')
                .into_iter()
                .map(|s| s.trim().replace(" ", "-").to_lowercase())
        })
        .flatten()
        .map(Ok)
        .collect()
}

pub(crate) fn openlibrary_isbn(hashmap_vec: HashMap<&str, Vec<&str>>) -> ISBNs {
    hashmap_vec
        .into_iter()
        .filter(|(k, _)| k.starts_with("isbn_"))
        .map(|(_, v)| v)
        .flatten()
        .map(|s| Isbn::from_str(s).map_err(ReconError::ISBNParse))
        .collect()
}

pub(crate) fn googlebooks_isbn(hashmap_vec: Vec<HashMap<&str, &str>>) -> ISBNs {
    hashmap_vec
        .into_iter()
        .map(|mut h| h.remove("identifier"))
        .flatten()
        .map(|s| Isbn::from_str(s).map_err(ReconError::ISBNParse))
        .collect()
}

pub(crate) fn publication_date(s: &str) -> PublicationDates {
    let possible_formats = ["%B %d, %Y", "%Y-%m-%d", "%B, %d %Y"];

    possible_formats
        .iter()
        .map(|fmt| NaiveDate::parse_from_str(s, fmt).map_err(ReconError::DateParse))
        .filter(|s| s.is_ok())
        .collect::<Vec<Result<NaiveDate, ReconError>>>()
}
