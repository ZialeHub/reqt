use proc_macro_api_manager::Filter;

use crate::query::Query;

#[derive(Debug, Clone, Default, PartialEq, Filter)]
pub struct FilterRule {
    pub pattern: String,
    pub to_replace: String,
}

pub trait Filter: Default + Clone
where
    Self: Sized,
{
    fn filter<T: IntoIterator>(self, _property: String, _value: T) -> Self;

    fn to_query(&self) -> Query;
}
// property[filter]=value
// filter = lte, gte, exists, regex, before, and after
//
// property=filter:value
// filter = lte, gte, exists, regex, before, and after
//
// filter[property]=valueSEPvalue2
// SEP is the separator ',', ...
