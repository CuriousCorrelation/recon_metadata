use crate::recon::ReconError;
use chrono::NaiveDate;
use isbn::Isbn;

pub type Generic = Result<String, ReconError>;
pub type Numeric = Result<u16, ReconError>;
pub type ISBN = Result<Isbn, ReconError>;
pub type Date = Result<NaiveDate, ReconError>;

pub type ISBNs = Vec<ISBN>;
pub type Titles = Vec<Generic>;
pub type Authors = Vec<Generic>;
pub type Descriptions = Vec<Generic>;
pub type Publishers = Vec<Generic>;
pub type PublicationDates = Vec<Date>;
pub type Languages = Vec<Generic>;
pub type Tags = Vec<Generic>;
pub type PageCount = Vec<Numeric>;
pub type CoverImages = Vec<Generic>;
