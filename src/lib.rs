//#![warn(missing_docs)]
//#![warn(clippy::all)]
//#![warn(missing_debug_implementations)]
//#![warn(missing_debug_implementations)]
//#![feature(result_flattening)]

/*!
# Usage

## Basic usage

### ISBN search

```
#[tokio::main]
async fn main() {
    use recon_metadata::Metadata;
    use std::str::FromStr;

    let isbn = isbn::Isbn::from_str("9781534431003").unwrap();
    let resp = Metadata::from_isbn(&isbn)
        .source(Source::default())
        .await;

    assert!(resp.is_ok())
}
```

### Descriptive search

```
#[tokio::main]
async fn main() {
    use recon_metadata::Metadata;
    use std::str::FromStr;

    let description = "This is how you lose the time war";
    let resp = Metadata::from_description(&description)
        .search_provider(SearchProvider::default())
        .source(Source::default())
        .await;

    assert!(resp.is_ok())
}
```

*/

pub mod metadata;
pub mod recon;
pub mod sources;
pub mod types;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    // #[tokio::main]
    // fn parses_metadata_from_isbn() {
    //     use recon_metadata::Metadata;
    //     use std::str::FromStr;
    //
    //     let isbn = isbn::Isbn::from_str("9781534431003").unwrap();
    //     let resp = Metadata::from_isbn(&isbn).source(Source::default()).await;
    //
    //     assert!(resp.is_ok())
    // }
}
