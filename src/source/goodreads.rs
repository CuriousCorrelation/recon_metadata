use std::{collections::HashSet, str::FromStr};

use crate::metadata::{CoverImage, Metadata};
use crate::recon::ReconError;
use isbn2::{Isbn, Isbn10, Isbn13};
use log::debug;
use scraper::{Html, Selector};

#[derive(Debug)]
/// A wrapper around [`Metadata`] for deserialization
pub struct Goodreads(Metadata);

impl Goodreads {
    /// Parses [`Metadata`] from `Goodreads` book details page
    /// This is an example of a book details page:
    /// <https://www.goodreads.com/book/show/53870787-this-is-how-you-lose-the-time-war>
    pub async fn from_web_page(page: &Html) -> Metadata {
        let title_selector = Selector::parse("h1#bookTitle").unwrap();
        let mut title = HashSet::new();
        for element in page.select(&title_selector) {
            title.insert(
                element
                    .inner_html()
                    .trim_matches(&['\n', ' '][..])
                    .to_string(),
            );
        }

        let author_selector = Selector::parse(r#"a.authorName span[itemprop="name"]"#).unwrap();
        let mut author = HashSet::new();
        for element in page.select(&author_selector) {
            author.insert(element.inner_html());
        }

        let tag_selector = Selector::parse("a.actionLinkLite.bookPageGenreLink").unwrap();
        let mut tag = HashSet::new();
        for element in page.select(&tag_selector) {
            tag.insert(element.inner_html());
        }

        let language_selector = Selector::parse(r#"div[itemprop="inLanguage"]"#).unwrap();
        let mut language = HashSet::new();
        for element in page.select(&language_selector) {
            language.insert(element.inner_html());
        }

        let isbn_selector = Selector::parse(r#"span[itemprop="isbn"]"#).unwrap();
        let mut isbn_10 = HashSet::new();
        let mut isbn_13 = HashSet::new();
        for element in page.select(&isbn_selector) {
            let isbn = element.inner_html();

            if isbn.len() == 13 {
                isbn_13.insert(Isbn13::from_str(&isbn).ok());
            } else if isbn.len() == 10 {
                isbn_10.insert(Isbn10::from_str(&isbn).ok());
            } else {
                continue;
            }
        }
        let isbn10 = isbn_10.into_iter().flatten().collect::<HashSet<_>>();
        let isbn13 = isbn_13.into_iter().flatten().collect::<HashSet<_>>();

        let description_selector =
            Selector::parse(r#"div#description span[style="display:none"]"#).unwrap();
        let mut description = HashSet::new();
        for element in page.select(&description_selector) {
            description.insert(element.inner_html());
        }

        let cover_image_selector = Selector::parse("img#coverImage").unwrap();
        let mut cover_image = HashSet::new();
        for element in page.select(&cover_image_selector) {
            cover_image.insert(element.value().attr("src"));
        }
        // TODO: Fix fallback
        let cover_image = CoverImage {
            thumbnail:       HashSet::default(),
            small_thumbnail: HashSet::default(),
            small:           HashSet::default(),
            medium:          HashSet::default(),
            large:           HashSet::default(),
            extra_large:     HashSet::default(),
        };

        let page_count_selector = Selector::parse(r#"span[itemprop="numberOfPages"]"#).unwrap();
        let mut page_count = HashSet::new();
        for element in page.select(&page_count_selector) {
            let page_count_parse = element
                .inner_html()
                .chars()
                .into_iter()
                .filter(|c| c.is_digit(10))
                .collect::<String>()
                .parse::<u16>()
                .ok();
            page_count.insert(page_count_parse);
        }
        let page_count = page_count.into_iter().flatten().collect::<HashSet<_>>();

        Metadata {
            isbn10,
            isbn13,
            title,
            author,
            description,
            page_count,
            language,
            tag,
            cover_image,
            publisher: HashSet::new(),
            publication_date: HashSet::new(),
        }
    }
}

impl Goodreads {
    /// Performs an ISBN search using Goodreads search
    pub async fn from_isbn(isbn: &isbn2::Isbn) -> Result<Metadata, ReconError> {
        let req = format!(
            "https://www.goodreads.com/search?q={}&search[source]=goodreads&search_type=books&tab=books",
            urlencoding::encode(&isbn.to_string())
        );

        debug!("ISBN: {:#?}", &isbn);
        debug!("Request: {:#?}", &req);

        let response = reqwest::get(req)
            .await
            .map_err(ReconError::Connection)?
            .text()
            .await
            .map_err(ReconError::Connection)?;

        debug!("Response: {:#?}", &response);

        let page = Html::parse_fragment(&response);

        Ok(Self::from_web_page(&page).await)
    }

    /// Performs a descriptive search using Goodreads search
    pub async fn from_description(_description: &str) -> Result<Vec<Isbn>, ReconError> {
        Err(ReconError::Message(
            "Goodreads cannot be a search source currently.".to_owned(),
        ))
    }
}

#[cfg(test)]
mod test {
    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn parses_from_isbn() {
        use super::Goodreads;
        use isbn2::Isbn;
        use log::debug;
        use std::str::FromStr;

        init_logger();

        let isbn = Isbn::from_str("9781534431003").unwrap();
        let resp = Goodreads::from_isbn(&isbn).await;
        debug!("Response: {:#?}", resp);
        println!("Response: {:#?}", resp);
        assert!(resp.is_ok())
    }

    #[tokio::test]
    async fn parses_from_description() {
        use super::Goodreads;

        init_logger();

        let description = "The way of kings";
        let resp = Goodreads::from_description(description).await;
        println!("Response: {:#?}", resp);
        assert!(resp.is_err())
    }
}
