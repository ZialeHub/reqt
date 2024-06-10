use proc_macro_api_manager::Sort;

use crate::query::Query;

#[derive(Debug, Clone, Default, PartialEq, Sort)]
pub struct SortRule {
    pub pattern: String,
    pub to_replace: String,
}

pub trait Sort: Default + Clone
where
    Self: Sized,
{
    fn sort<T: IntoIterator>(self, _sort: T) -> Self;
    fn to_query(&self) -> Query;
}

// sort=-property1,property2
// '-' is descending order
// '+' or nothing is ascending order
//
// sort=asc(property1),desc(property2)
//
// sort=property1.asc,property2.desc
//
// sort_by=property&order_by=asc
