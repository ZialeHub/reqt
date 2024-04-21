use crate::query::Query;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum PaginationRule {
    #[default]
    None, // Only get the first page
    Fixed(u32), // Limit pages bundling to [u32]
    OneShot,    // Always compute all pages
}

#[derive(Debug, Clone)]
pub struct RequestPagination {
    pub(crate) size: u32,
    pub(crate) current_page: u32,
    pub(crate) pagination: PaginationRule,
}

impl Default for RequestPagination {
    fn default() -> Self {
        Self {
            size: 100,
            current_page: 0,
            pagination: PaginationRule::None,
        }
    }
}

impl Pagination for RequestPagination {
    fn size(mut self, size: u32) -> Self {
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

    fn current_page(&self) -> u32 {
        self.current_page
    }

    fn get_current_page(&self) -> Query {
        Query::new().add("page[number]", self.current_page)
    }

    fn get_size(&self) -> Query {
        Query::new().add("page[size]", self.size)
    }

    fn next(&mut self) {
        self.current_page += 1;
    }

    fn get_next(&mut self) -> Query {
        self.current_page += 1;
        Query::new().add("page[number]", self.current_page)
    }
}

pub trait Pagination {
    fn size(self, size: u32) -> Self;
    fn pagination(self, rule: PaginationRule) -> Self;
    fn get_pagination(&self) -> &PaginationRule;
    fn current_page(&self) -> u32;
    fn get_current_page(&self) -> Query;
    fn get_size(&self) -> Query;
    fn next(&mut self);
    fn get_next(&mut self) -> Query;
}
