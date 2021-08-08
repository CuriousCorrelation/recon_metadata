use crate::recon::ReconError;
use chrono::NaiveDate;
use isbn::Isbn;

pub type ISBNs = Vec<Result<Isbn, ReconError>>;
pub type Titles = Vec<Result<String, ReconError>>;
pub type Authors = Vec<Result<String, ReconError>>;
pub type Descriptions = Vec<Result<String, ReconError>>;
pub type Publishers = Vec<Result<String, ReconError>>;
pub type PublicationDates = Vec<Result<NaiveDate, ReconError>>;
pub type Languages = Vec<Result<String, ReconError>>;
pub type Tags = Vec<Result<String, ReconError>>;
pub type PageCount = Vec<Result<u16, ReconError>>;
pub type CoverImages = Vec<Result<String, ReconError>>;
