pub mod client;
pub mod dto;
pub mod handler;

pub use client::ParliamentClient;
pub use dto::{
    FetchBillsArgs, FetchCoreDatasetArgs, FetchHistoricHansardArgs, FetchLegislationArgs,
};
pub use handler::{
    handle_fetch_bills, handle_fetch_core_dataset, handle_fetch_historic_hansard, handle_fetch_legislation,
};
