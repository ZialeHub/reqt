use crate::prelude::Query;
use range_derive::Range;

#[derive(Debug, Clone, Default, Range)]
pub struct RangeRule {
    pub pattern: String,
    pub ranges: Vec<(String, String)>,
}

impl From<&RangeRule> for Query {
    fn from(_value: &RangeRule) -> Self {
        Query::new()
    }
}

pub trait Range: Default + Clone {
    /// Set the pattern to match the range
    fn pattern(self, pattern: impl ToString) -> Self;

    /// Add a range to the list
    /// You should implement this method to override the property if already exists
    fn range(self, property: impl ToString, min: impl ToString, max: impl ToString) -> Self;
}
