use proc_macro_api_manager::Sort;

use crate::query::Query;

#[derive(Debug, Clone, Default)]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}
impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "asc"),
            SortOrder::Desc => write!(f, "desc"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Sort)]
pub struct SortRule {
    pub pattern: String,
    pub sorts: Vec<String>,
}

pub trait Sort: Default + Clone
where
    Self: Sized,
{
    /// Set the pattern to match the sort
    /// The pattern must contains the words "property" and can contains "order"
    /// property will be replaced by the property name you want to sort
    ///
    /// Example: "property"
    /// Example: "order(property)"
    /// Example: "property.order"
    fn pattern(self, pattern: impl ToString) -> Self;

    /// Add a sort on a property to the list
    /// Usage: in case of : sort=-property1,property2
    fn sort(self, property: impl ToString) -> Self;

    /// Add a sort with order on a property to the list
    fn sort_with(self, property: impl ToString, order: SortOrder) -> Self;

    /// Convert the sort to a query
    /// The query will be joined with the values separated by '&'
    fn to_query(&self) -> Query;
}
