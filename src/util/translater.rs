//! Translates API response data chunks into `Metadata` types

/// Different book API responses are usually similar in shape so this module's job
/// is to provide multipurpose functions that can be applied to a piece of `JSON` data
/// provided by `serde` via `Source` module and translate them into `Metadata` type
use crate::metadata::CoverImage;
use chrono::NaiveDate;
use isbn2::{Isbn10, Isbn13};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

/// Helper function that takes an [`Option`] value and converts it into an [`HashSet`]
/// by mapping [`None`] to empty [`HashSet`] and [`Some`] to an inserted element.
/// `Metadata` struct contains a [`HashSet`] for each of its fields
/// where incomplete or unavailable information is avoided in-favor of information
/// from more sources defined in `Source` struct in `recon.rs`.
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

/// A fallback function for [`HashSet`]. Works on the same principle as `optional_to_hashset`
/// mapping [`Some`] to the value inside and [`None`] to an empty container.
pub(crate) fn hashset_fallback<T: std::hash::Hash + std::cmp::Eq>(
    value: Option<HashSet<T>>,
) -> HashSet<T> {
    match value {
        Some(value) => value,
        None => HashSet::new(),
    }
}

pub(crate) fn empty() -> HashSet<String> {
    HashSet::new()
}

/// Following functions translate `serde` values into values compatible with `Metadata` field.
/// "..." means 'doesn't matter' or handled by `serde` `deserialize` implementation elsewhere.

/// Example use-case:
/// { "...": 42 } -> Serde { 42 } -> [42]
pub(crate) fn number(n: Option<u16>) -> HashSet<u16> {
    optional_to_hashset(n)
}

/// Example use-case:
/// { "...": "some string" } -> Serde { "some string" } -> ["some string"]
pub(crate) fn string(s: Option<String>) -> HashSet<String> {
    optional_to_hashset(s)
}

/// Example use-case:
/// { "...": ["some string", "some other string", "some string"] }
///   -> Serde { ["some string", "some other string", "some string"] }
///   -> ["some string", "some other string"]
pub(crate) fn vec(vec: Option<Vec<&str>>) -> HashSet<String> {
    hashset_fallback(vec.map(|vec| vec.into_iter().map(|v| v.to_owned()).collect()))
}

/// Example use-case:
/// { "...":
///    [
///       { "...": "value1" }
///       { "...": "value2" }
///    ],
///    [
///       { "...": "value3" }
///       { "...": "value4" }
///       { "...": "value5" }
///    ],
///    ...
/// }
///   -> ["value1", "value2", "value3", "value4", "value5"]
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

/// Function call: translater::vec_hashmap_field_split(opt_vec_hmap, "name"),
/// Example use-case:
///
/// "...":
///   [
///      {
///        "name": "science fiction",
///        "url": "...",
///        "...": "..."
///      },
///      {
///        "name": "time-traveling",
///        "url": "...",
///        "...": "..."
///      },
///      {
///        "name": "epistolary",
///        "url": "...",
///        "...": "..."
///      },
///      {
///        "name": "Fiction, science fiction, general",
///        "url": "...",
///        "...": "..."
///      }
///   ]
///
///   -> ["science-fiction", "time-traveling", "epistolary", "fiction", "general"]
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

/// Example use-case:
///
/// "...":
///   {
///      "isbn_10":
///      [
///        "isbn1",
///        "isbn2"
///      ],
///      "...": "..."
///   }
///
///   -> [Isbn10(isbn1), Isbn10(isbn2)]
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

/// Example use-case:
///
/// "...":
///   {
///      "isbn_13":
///      [
///        "isbn1",
///        "isbn2"
///      ],
///      "...": "..."
///   }
///
///   -> [Isbn13(isbn1), Isbn13(isbn2)]
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

/// Example use-case:
///
/// "...":
///   {
///     "small": "a.jpg",
///     "medium": "b.jpg",
///     "large": "c.jpg"
///   }
///
///   -> [`CoverImage`]
///   {
///    small_thumbnail: [],
///    thumbnail:       [],
///    small:           ["a.jpg"],
///    medium:          ["b.jpg"],
///    large:           ["c.jpg"],
///    extra_large:     [],
///   }
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

/// Example use-case:
///
/// "...":
///   [
///     {
///       "type": "ISBN_13",
///       "identifier": "isbn13"
///     },
///     {
///       "type": "ISBN_10",
///       "identifier": "isbn10"
///     }
///   ],
///
///   -> Isbn10(isbn10)
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

/// Example use-case:
///
/// "...":
///   {
///     "smallThumbnail": "a.jpg",
///     "extraLarge": "b.jpg"
///   }
///
///   -> [`CoverImage`]
///   {
///    small_thumbnail: ["a.jpg"],
///    thumbnail:       [],
///    small:           [],
///    medium:          [],
///    large:           [],
///    extra_large:     ["b.jpg"],
///   }
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

/// Example use-case:
///
/// "...":
///   [
///     {
///       "type": "ISBN_13",
///       "identifier": "isbn13"
///     },
///     {
///       "type": "ISBN_10",
///       "identifier": "isbn10"
///     }
///   ],
///
///   -> Isbn13(isbn13)
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

/// Example use-case:
///
/// { "...": "2019-07-16" }
///
/// -> [NaiveDate(2019-07-16)]
///
/// { "...": "May 07 16" }
///
/// -> [NaiveDate(2016-05-07)]
///
/// { "...": "Not a date" }
///
/// -> []
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
