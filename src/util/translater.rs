use crate::metadata::CoverImage;
use chrono::NaiveDate;
use isbn2::{Isbn10, Isbn13};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

pub(crate) fn optional_to_hashset<T: std::hash::Hash + std::cmp::Eq>(
    value: Option<T>,
) -> HashSet<T> {
    match value {
        Some(value) => {
            let mut h = HashSet::new();
            h.insert(value);
            h
        }
        None => HashSet::new(),
    }
}

pub(crate) fn hashset_fallback<T: std::hash::Hash + std::cmp::Eq>(
    value: Option<HashSet<T>>,
) -> HashSet<T> {
    match value {
        Some(value) => value,
        None => HashSet::new(),
    }
}

pub(crate) fn number(n: Option<u16>) -> HashSet<u16> {
    optional_to_hashset(n)
}

pub(crate) fn empty() -> HashSet<String> {
    HashSet::new()
}

pub(crate) fn string(s: Option<String>) -> HashSet<String> {
    optional_to_hashset(s)
}

pub(crate) fn vec(vec: Option<Vec<&str>>) -> HashSet<String> {
    hashset_fallback(vec.map(|vec| vec.into_iter().map(|v| v.to_owned()).collect()))
}

pub(crate) fn vec_hashmap_field(
    vec_hashmap: Option<Vec<HashMap<&str, &str>>>,
    field: &str,
) -> HashSet<String> {
    hashset_fallback(vec_hashmap.map(|vec_hashmap| {
        vec_hashmap
            .into_iter()
            .map(|mut h| h.remove(field))
            .flatten()
            .map(|s| s.to_owned())
            .collect()
    }))
}

pub(crate) fn vec_hashmap_field_split(
    vec_hashmap: Option<Vec<HashMap<&str, &str>>>,
    field: &str,
) -> HashSet<String> {
    hashset_fallback(vec_hashmap.map(|vec_hashmap| {
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
    }))
}

pub(crate) fn openlibrary_isbn10(
    hashmap_vec: &Option<HashMap<&str, Vec<&str>>>,
) -> HashSet<Isbn10> {
    hashset_fallback(hashmap_vec.as_ref().map(|hashmap_vec| {
        hashmap_vec
            .iter()
            .filter(|(k, _)| k.starts_with("isbn_10"))
            .map(|(_, v)| v)
            .flatten()
            .map(|s| Isbn10::from_str(s))
            .flatten() // discarding `Err`
            .collect()
    }))
}

pub(crate) fn openlibrary_isbn13(
    hashmap_vec: &Option<HashMap<&str, Vec<&str>>>,
) -> HashSet<Isbn13> {
    hashset_fallback(hashmap_vec.as_ref().map(|hashmap_vec| {
        hashmap_vec
            .iter()
            .filter(|(k, _)| k.starts_with("isbn_13"))
            .map(|(_, v)| v)
            .flatten()
            .map(|s| Isbn13::from_str(s))
            .flatten() // discarding `Err`
            .collect()
    }))
}

pub(crate) fn openlibrary_cover_images(hashmap: Option<HashMap<&str, &str>>) -> CoverImage {
    hashmap
        .map(|mut hashmap| {
            let small_thumbnail = HashSet::default();
            let thumbnail = HashSet::default();
            let small = hashmap
                .get_mut("small")
                .map(|sm| -> HashSet<String> {
                    let mut hs = HashSet::new();
                    hs.insert(sm.to_owned());
                    hs
                })
                .unwrap_or_default();
            let medium = hashmap
                .get_mut("medium")
                .map(|sm| -> HashSet<String> {
                    let mut hs = HashSet::new();
                    hs.insert(sm.to_owned());
                    hs
                })
                .unwrap_or_default();
            let large = hashmap
                .get_mut("large")
                .map(|sm| -> HashSet<String> {
                    let mut hs = HashSet::new();
                    hs.insert(sm.to_owned());
                    hs
                })
                .unwrap_or_default();
            let extra_large = hashmap
                .get_mut("extraLarge")
                .map(|sm| -> HashSet<String> {
                    let mut hs = HashSet::new();
                    hs.insert(sm.to_owned());
                    hs
                })
                .unwrap_or_default();

            CoverImage {
                small_thumbnail,
                thumbnail,
                small,
                medium,
                large,
                extra_large,
            }
        })
        .unwrap_or_default()
}

pub(crate) fn googlebooks_isbn10(
    hashmap_vec: &Option<Vec<HashMap<&str, &str>>>,
) -> HashSet<Isbn10> {
    hashset_fallback(hashmap_vec.as_ref().map(|hashmap_vec| {
        hashmap_vec
            .iter()
            .filter(|h| h.get("type") == Some("ISBN_10").as_ref())
            .map(|h| h.get("identifier"))
            .flatten()
            .map(|s| Isbn10::from_str(s))
            .flatten() // discarding `Err`
            .collect()
    }))
}

pub(crate) fn googlebooks_cover_images(hashmap: Option<HashMap<&str, &str>>) -> CoverImage {
    hashmap
        .map(|mut hashmap| {
            let small_thumbnail = hashmap
                .get_mut("smallThumbnail")
                .map(|sm| -> HashSet<String> {
                    let mut hs = HashSet::new();
                    hs.insert(sm.to_owned());
                    hs
                })
                .unwrap_or_default();
            let thumbnail = hashmap
                .get_mut("thumbnail")
                .map(|sm| -> HashSet<String> {
                    let mut hs = HashSet::new();
                    hs.insert(sm.to_owned());
                    hs
                })
                .unwrap_or_default();
            let small = hashmap
                .get_mut("small")
                .map(|sm| -> HashSet<String> {
                    let mut hs = HashSet::new();
                    hs.insert(sm.to_owned());
                    hs
                })
                .unwrap_or_default();
            let medium = hashmap
                .get_mut("medium")
                .map(|sm| -> HashSet<String> {
                    let mut hs = HashSet::new();
                    hs.insert(sm.to_owned());
                    hs
                })
                .unwrap_or_default();
            let large = hashmap
                .get_mut("large")
                .map(|sm| -> HashSet<String> {
                    let mut hs = HashSet::new();
                    hs.insert(sm.to_owned());
                    hs
                })
                .unwrap_or_default();
            let extra_large = hashmap
                .get_mut("extraLarge")
                .map(|sm| -> HashSet<String> {
                    let mut hs = HashSet::new();
                    hs.insert(sm.to_owned());
                    hs
                })
                .unwrap_or_default();

            CoverImage {
                small_thumbnail,
                thumbnail,
                small,
                medium,
                large,
                extra_large,
            }
        })
        .unwrap_or_default()
}

pub(crate) fn googlebooks_isbn13(
    hashmap_vec: &Option<Vec<HashMap<&str, &str>>>,
) -> HashSet<Isbn13> {
    hashset_fallback(hashmap_vec.as_ref().map(|hashmap_vec| {
        hashmap_vec
            .iter()
            .filter(|h| h.get("type") == Some("ISBN_13").as_ref())
            .map(|h| h.get("identifier"))
            .flatten()
            .map(|s| Isbn13::from_str(s))
            .flatten() // discarding `Err`
            .collect()
    }))
}

pub(crate) fn publication_date(s: Option<&str>) -> HashSet<NaiveDate> {
    let possible_formats = ["%B %d, %Y", "%Y-%m-%d", "%B, %d %Y"];

    match s {
        Some(s) => possible_formats
            .iter()
            .map(|fmt| NaiveDate::parse_from_str(s, fmt))
            .filter(|s| s.is_ok())
            .map(|s| s.unwrap())
            .collect::<HashSet<NaiveDate>>(),

        None => HashSet::new(),
    }
}
