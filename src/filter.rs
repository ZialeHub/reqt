use proc_macro_api_manager::Filter;

use crate::query::Query;

#[derive(Debug, Clone, Default, Filter)]
pub struct FilterRule {
    pub pattern: String,
    pub filters: Vec<(String, String)>,
}

pub trait Filter: Default + Clone
where
    Self: Sized,
{
    /// Set the pattern to match the filter
    /// The pattern must contains the words "property" and can contains "filter"
    /// property will be replaced by the property name you want to filter
    /// filter will be replaced if the API supports multiple filters
    /// Filter example: lte, gte, exists, regex, before, after, ...
    /// The pattern will be joined with the values separated by '='
    ///
    /// Example: "property[filter]"
    /// Example: "filter[property]"
    /// Example: "property"
    fn pattern(self, pattern: impl ToString) -> Self;

    /// Add a a specific filter on a property to the list
    fn filter_with<T: IntoIterator>(
        self,
        property: impl ToString,
        filter: impl ToString,
        value: T,
    ) -> Self
    where
        T::Item: ToString;

    /// Add a filter to the list
    fn filter<T: IntoIterator>(self, property: impl ToString, value: T) -> Self
    where
        T::Item: ToString;

    /// Convert the filter to a query
    /// each filter will be joined with the values separated by '='
    /// The query will be joined with the values separated by '&'
    fn to_query(&self) -> Query;
}
