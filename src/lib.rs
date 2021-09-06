#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(missing_debug_implementations)]
#![warn(missing_debug_implementations)]
#![feature(result_flattening)]

/*!
# Usage

## Basic usage

There are two types of search `recon_metadata` can perform

### ISBN search

```
#[tokio::main]
async fn main() {
    use recon_metadata::{Metadata, Source, ReconError};
    use isbn2::Isbn;
    use std::str::FromStr;

    let isbn = Isbn::from_str("9781534431003").unwrap();

    let sources = [Source::GoogleBooks, Source::OpenLibrary];

    let res: Result<Metadata, ReconError> = Metadata::from_isbn(&sources, &isbn).await;

    assert!(res.is_ok());
}
```

### Descriptive search
Description search requires a primary source as well as a list of sources like `from_isbn`.

`from_description` search will first look for `ISBN` numbers associated with the description string given.
The sources will provide additional information about said `ISBN` numbers.

This way the search results remain consistent and reduce the risk of recursive search and duplicate results.
```
#[tokio::main]
async fn main() {
    use recon_metadata::{Metadata, Source, ReconError};

    let description = "This is how you lose the time war";

    let sources = [Source::GoogleBooks, Source::OpenLibrary];

    let res: Result<Vec<Metadata>, ReconError> =
        Metadata::from_description(&Source::GoogleBooks, &sources, description).await;

    assert!(res.is_ok());
}
```
*/

/// Book metadata returned by database and search APIs
pub mod metadata;
pub use metadata::Metadata;
/// Types required by `recon_metadata`
pub mod recon;
pub use recon::ReconError;
pub use recon::Source;
/// API and database sources
pub(crate) mod source;
/// Utility functions used for type conversion and field translation
pub(crate) mod util;

#[cfg(test)]
mod tests {
    use log::debug;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn parses_from_isbn() {
        use super::metadata::Metadata;
        use crate::recon::{ReconError, Source};
        use isbn2::Isbn;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();

        let sources = [Source::GoogleBooks, Source::OpenLibrary];

        let res: Result<Metadata, ReconError> = Metadata::from_isbn(&sources, &isbn).await;

        debug!("Response: {:#?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn parses_from_description() {
        use super::metadata::Metadata;
        use crate::recon::{ReconError, Source};

        init_logger();

        let description = "The way of kings by brandon sanderson";

        let sources = [Source::GoogleBooks, Source::OpenLibrary];

        let res: Result<Vec<Metadata>, ReconError> =
            Metadata::from_description(&Source::GoogleBooks, &sources, description).await;

        debug!("Response: {:#?}", res);
        assert!(res.is_ok());

        let res: Result<Vec<Metadata>, ReconError> =
            Metadata::from_description(&Source::OpenLibrary, &sources, description).await;

        debug!("Response: {:#?}", res);
        assert!(res.is_ok());
    }
}
