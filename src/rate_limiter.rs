use std::{fmt::Display, thread::sleep, time::Duration};
use strum::{EnumIter, IntoEnumIterator};

use chrono::{NaiveDateTime, TimeDelta};

/// TimePeriod is used to define the time period for the rate limiter\
/// It can be Second, Minute, Hour or Day
///
/// # Default Api Rate Limiter
/// ```rust,ignore
/// // The default rate limiter is 1 request per second
/// RateLimiter::new(1, TimePeriod::Second)
/// ```
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
impl From<TimePeriod> for TimeDelta {
    fn from(val: TimePeriod) -> Self {
        match val {
            TimePeriod::Second => Self::seconds(1),
            TimePeriod::Minute => Self::minutes(1),
            TimePeriod::Hour => Self::hours(1),
            TimePeriod::Day => Self::days(1),
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
    /// Create a new rate limiter
    ///
    /// # Attributes
    /// * limit - The limit of requests per period
    /// * period - The time period for the rate limiter
    /// * is_adaptive - If the rate limiter should adapt to the server rate limit
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

    /// Set the rate limiter to be adaptive or not
    ///
    /// If the rate limiter is adaptive, it will adapt to the server rate limit,
    /// using the lowest rate found in the headers
    pub fn is_adaptive(mut self, is_adaptive: bool) -> Self {
        self.is_adaptive = is_adaptive;
        self
    }

    /// Update the rate limiter with the headers from the request
    pub fn update(&mut self, headers: &reqwest::header::HeaderMap) {
        if !self.is_adaptive {
            return;
        }
        for period in TimePeriod::iter() {
            if let Some(limit) = headers.get(format!("x-{}-ratelimit-limit", period)) {
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

    /// Sleep for the period of the rate limiter
    fn sleep(&self) {
        match self.period {
            TimePeriod::Second => sleep(Duration::from_secs(1)),
            TimePeriod::Minute => sleep(Duration::from_secs(60)),
            TimePeriod::Hour => sleep(Duration::from_secs(3600)),
            TimePeriod::Day => sleep(Duration::from_secs(86400)),
        }
    }

    /// Request once the rate limit is available
    pub fn request(&mut self) {
        loop {
            if chrono::Utc::now().naive_local() - self.timer >= self.period.clone().into() {
                self.timer = chrono::Utc::now().naive_local();
                self.remaining = self.limit;
            }
            if self.remaining > 0 {
                self.remaining -= 1;
                return;
            } else {
                self.is_asleep = true;
                eprintln!("Rate limit exceeded, sleeping for {:?}", self.period);
                self.sleep();
                self.is_asleep = false;
            }
        }
    }
}
