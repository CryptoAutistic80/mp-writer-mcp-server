use crate::features::utilities::{CurrentDatetimeDto, DateTimeService};

pub fn handle_current_datetime(service: &DateTimeService) -> CurrentDatetimeDto {
    service.current_datetime()
}
