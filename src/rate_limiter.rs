use std::{fmt::Display, thread::sleep, time::Duration};
use strum::{EnumIter, IntoEnumIterator};

use chrono::{NaiveDateTime, TimeDelta};

#[derive(Debug, Clone, Default, EnumIter, PartialEq)]
pub enum TimePeriod {
    #[default]
    Second,
    Minute,
    Hour,
    Day,
}
impl Display for TimePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimePeriod::Second => write!(f, "secondly"),
            TimePeriod::Minute => write!(f, "minute"),
            TimePeriod::Hour => write!(f, "hourly"),
            TimePeriod::Day => write!(f, "dayly"),
        }
    }
}
impl Into<TimeDelta> for TimePeriod {
    fn into(self) -> TimeDelta {
        match self {
            TimePeriod::Second => TimeDelta::seconds(1),
            TimePeriod::Minute => TimeDelta::minutes(1),
            TimePeriod::Hour => TimeDelta::hours(1),
            TimePeriod::Day => TimeDelta::days(1),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RateLimiter {
    pub limit: u32,
    pub remaining: u32,
    pub period: TimePeriod,
    pub is_asleep: bool,
    pub is_adaptive: bool,
    pub timer: NaiveDateTime,
}
impl RateLimiter {
    pub fn new(limit: u32, period: TimePeriod) -> RateLimiter {
        RateLimiter {
            limit,
            remaining: limit,
            period,
            is_asleep: false,
            is_adaptive: true,
            timer: chrono::Utc::now().naive_local(),
        }
    }
    pub fn is_adaptive(mut self, is_adaptive: bool) -> Self {
        self.is_adaptive = is_adaptive;
        self
    }

    pub fn update(&mut self, headers: &reqwest::header::HeaderMap) {
        if !self.is_adaptive {
            return;
        }
        for period in TimePeriod::iter() {
            if let Some(limit) = headers.get(format!("x-{}-ratelimit-limit", period.to_string())) {
                let Ok(limit) = limit.to_str().unwrap().parse::<u32>() else {
                    return;
                };
                if self.period == period && limit < self.limit {
                    self.limit = limit;
                    return;
                }
                self.period = period;
                if limit < self.limit {
                    self.limit = limit;
                }
                return;
            }
        }
    }

    fn sleep(&self) {
        match self.period {
            TimePeriod::Second => sleep(Duration::from_secs(1)),
            TimePeriod::Minute => sleep(Duration::from_secs(60)),
            TimePeriod::Hour => sleep(Duration::from_secs(3600)),
            TimePeriod::Day => sleep(Duration::from_secs(86400)),
        }
    }

    pub fn request(&mut self) {
        loop {
            if chrono::Utc::now().naive_local() - self.timer >= self.period.clone().into() {
                self.timer = chrono::Utc::now().naive_local();
                self.remaining = self.limit;
            }
            if self.remaining > 0 {
                self.remaining -= 1;
                return;
            // } else if self.is_asleep {
            //     return false;
            } else {
                self.is_asleep = true;
                eprintln!("Rate limit exceeded, sleeping for {:?}", self.period);
                self.sleep();
                self.is_asleep = false;
            }
        }
    }
}
