use chrono::NaiveDate;
use isbn2::{Isbn10, Isbn13};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

pub(crate) fn number(n: u16) -> HashSet<u16> {
    let mut h = HashSet::new();
    h.insert(n);
    h
}

pub(crate) fn empty() -> HashSet<String> {
    HashSet::new()
}

pub(crate) fn string(s: &str) -> HashSet<String> {
    let mut h = HashSet::new();
    h.insert(s.to_owned());
    h
}

pub(crate) fn hashmap(hashmap: HashMap<&str, &str>) -> HashSet<String> {
    hashmap.into_iter().map(|(_, v)| v.to_owned()).collect()
}

pub(crate) fn vec(vec: Vec<&str>) -> HashSet<String> {
    vec.into_iter().map(|v| v.to_owned()).collect()
}

pub(crate) fn vec_hashmap_field(
    vec_hashmap: Vec<HashMap<&str, &str>>,
    field: &str,
) -> HashSet<String> {
    vec_hashmap
        .into_iter()
        .map(|mut h| h.remove(field))
        .flatten()
        .map(|s| s.to_owned())
        .collect()
}

pub(crate) fn vec_hashmap_field_split(
    vec_hashmap: Vec<HashMap<&str, &str>>,
    field: &str,
) -> HashSet<String> {
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
        .collect()
}

pub(crate) fn openlibrary_isbn10(hashmap_vec: &HashMap<&str, Vec<&str>>) -> HashSet<Isbn10> {
    hashmap_vec
        .iter()
        .filter(|(k, _)| k.starts_with("isbn_10"))
        .map(|(_, v)| v)
        .flatten()
        .map(
            |s| Isbn10::from_str(s).unwrap(), // assuming ISBN is valid
        )
        .collect()
}

pub(crate) fn openlibrary_isbn13(hashmap_vec: &HashMap<&str, Vec<&str>>) -> HashSet<Isbn13> {
    hashmap_vec
        .iter()
        .filter(|(k, _)| k.starts_with("isbn_13"))
        .map(|(_, v)| v)
        .flatten()
        .map(
            |s| Isbn13::from_str(s).unwrap(), // assuming ISBN is valid
        )
        .collect()
}

pub(crate) fn googlebooks_isbn10(hashmap_vec: &Vec<HashMap<&str, &str>>) -> HashSet<Isbn10> {
    hashmap_vec
        .iter()
        .filter(|h| h.get("type") == Some("ISBN_10").as_ref())
        .map(|h| h.get("identifier"))
        .flatten()
        .map(
            |s| Isbn10::from_str(s).unwrap(), // assuming ISBN is valid
        )
        .collect()
}

pub(crate) fn googlebooks_isbn13(hashmap_vec: &Vec<HashMap<&str, &str>>) -> HashSet<Isbn13> {
    hashmap_vec
        .iter()
        .filter(|h| h.get("type") == Some("ISBN_13").as_ref())
        .map(|h| h.get("identifier"))
        .flatten()
        .map(
            |s| Isbn13::from_str(s).unwrap(), // assuming ISBN is valid
        )
        .collect()
}

pub(crate) fn publication_date(s: &str) -> HashSet<NaiveDate> {
    let possible_formats = ["%B %d, %Y", "%Y-%m-%d", "%B, %d %Y"];

    possible_formats
        .iter()
        .map(|fmt| NaiveDate::parse_from_str(s, fmt))
        .filter(|s| s.is_ok())
        .map(|s| s.unwrap())
        .collect::<HashSet<NaiveDate>>()
}
