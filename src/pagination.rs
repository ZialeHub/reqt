use crate::query::Query;

/// Pagination rule to be used in the API
///
/// # Variants
/// * `Fixed` - Limit pages bundling to [usize]
/// * `OneShot` - Always compute all pages
///
/// # Default
/// * `Fixed(1)` - Default to one page
#[derive(Debug, Clone, PartialEq)]
pub enum PaginationRule {
    Fixed(usize),
    OneShot,
}
impl Default for PaginationRule {
    fn default() -> Self {
        Self::Fixed(1)
    }
}

/// Default pagination rule
///
/// # Attributes
/// * size - Number of items per page
/// * current_page - Current page number
/// * pagination - Pagination rule to be used
///
/// # Default
/// * size - 100
/// * current_page - 1
/// * pagination - PaginationRule::default()
///
/// The default pagination rule is:
/// 'page[number]=x' with x starting at 1
/// 'page[size]=y' with y starting at 100
#[derive(Debug, Clone)]
pub struct RequestPagination {
    pub(crate) size: usize,
    pub(crate) current_page: usize,
    pub(crate) pagination: PaginationRule,
}

impl Default for RequestPagination {
    fn default() -> Self {
        Self {
            size: 100,
            current_page: 1,
            pagination: PaginationRule::default(),
        }
    }
}

impl Pagination for RequestPagination {
    fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = pagination;
        self
    }

    fn get_pagination(&self) -> &PaginationRule {
        &self.pagination
    }

    fn current_page(&self) -> usize {
        self.current_page
    }

    fn get_current_page(&self) -> Query {
        Query::new()
            .add("page[number]", self.current_page)
            .add("page[size]", self.size)
    }

    fn get_size(&self) -> Query {
        Query::new().add("page[size]", self.size)
    }

    fn next(&mut self) {
        self.current_page += 1;
    }

    fn get_next_page(&mut self) -> Query {
        self.current_page += 1;
        Query::new()
            .add("page[number]", self.current_page)
            .add("page[size]", self.size)
    }

    fn reset(&mut self) {
        self.current_page = 1;
    }
}

/// Pagination trait to be implemented by the user
/// to allow custom pagination rules for the API
pub trait Pagination: Clone {
    fn size(self, size: usize) -> Self;
    fn reset(&mut self);
    fn pagination(self, rule: PaginationRule) -> Self;
    fn get_pagination(&self) -> &PaginationRule;
    fn current_page(&self) -> usize;
    fn get_current_page(&self) -> Query;
    fn get_size(&self) -> Query;
    fn next(&mut self);
    fn get_next_page(&mut self) -> Query;
}
