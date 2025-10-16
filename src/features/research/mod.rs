pub mod dto;
pub mod handler;
mod helpers;
pub mod service;

#[allow(unused_imports)]
pub use dto::{
    BillSummaryDto, DebateSummaryDto, LegislationSummaryDto, PartyBreakdownDto, ResearchRequestDto,
    ResearchResponseDto, SpeechSummaryDto, StateOfPartiesDto, VoteSummaryDto,
};
pub use handler::handle_run_research;
pub use service::{ParliamentDataSource, ResearchService};
