use crate::core::error::AppError;
use crate::features::research::ResearchService;
use crate::features::research::dto::{ResearchRequestDto, ResearchResponseDto};

pub async fn handle_run_research(
    service: &ResearchService,
    request: ResearchRequestDto,
) -> Result<ResearchResponseDto, AppError> {
    service.run_research(request).await
}
