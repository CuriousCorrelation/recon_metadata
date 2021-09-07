# `recon_metadata`

[![pipeline status](https://gitlab.com/CuriousCorrelation/recon_metadata/badges/main/pipeline.svg)](https://gitlab.com/CuriousCorrelation/recon_metadata/-/commits/main) 

`recon_metadata`, book details and metadata search library written in [Rust](https://www.rust-lang.org/) using

- [`reqwest`](https://docs.rs/reqwest/0.11.4/reqwest/)
- [`futures`](https://docs.rs/futures/0.3.17/futures/)
- [`scraper`](https://docs.rs/scraper/0.12.0/scraper/)
- [`tokio`](https://tokio.rs/)
- [`serde`](serde.rs/)

## Installation

### Add as dependency in `Cargo.toml`

You can use it as a submodule or directly via `cargo`'s git option.

#### `git`

##### GitLab
``` toml
[dependencies]
recon_metadata = { git = "https://gitlab.com/CuriousCorrelation/recon_metadata" }
```

##### GitHub
``` toml
[dependencies]
recon_metadata = { git = "https://github.com/CuriousCorrelation/recon_metadata" }
```

#### Submodule
``` toml
[dependencies]
recon_metadata = { path = "recon_metadata" }
```

### Usage

There are two types of search `recon_metadata` can perform

#### ISBN search

``` rust
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

#### Descriptive search

Description search requires a primary source as well as a list of sources like `from_isbn`.

`from_description` search will first look for `ISBN` numbers associated with the description string given.
The sources will provide additional information about said `ISBN` numbers.

This way the search results remain consistent and reduce the risk of recursive search and duplicate results.
``` rust
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
