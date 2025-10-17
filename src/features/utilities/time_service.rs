use chrono::{DateTime, Utc};
use chrono_tz::Europe::London;

use crate::features::utilities::dto::CurrentDatetimeDto;

pub struct DateTimeService;

impl Default for DateTimeService {
    fn default() -> Self {
        Self::new()
    }
}

impl DateTimeService {
    pub fn new() -> Self {
        Self
    }

    pub fn current_datetime(&self) -> CurrentDatetimeDto {
        let utc_now: DateTime<Utc> = Utc::now();
        let london_time = utc_now.with_timezone(&London);

        CurrentDatetimeDto {
            utc: utc_now.to_rfc3339(),
            local: london_time.to_rfc3339(),
        }
    }
}
