use reqwest::{Method, Url};

use crate::{
    error::{ApiError, Result},
    filter::Filter,
    pagination::Pagination,
    query::Query,
    range::Range,
    sort::Sort,
};

/// Structure to create a request URL
///
/// # Attributes
/// * endpoint - Endpoint to be used in the request
/// * route - Route to be used in the request
/// * query - Query to be used in the request
/// * method - HTTP method to be used in the request
#[derive(Debug, Clone)]
pub struct RequestUrl {
    pub(crate) endpoint: String,
    pub(crate) route: String,
    pub(crate) query: Query,
    pub(crate) method: Method,
}

impl RequestUrl {
    pub fn new(endpoint: impl ToString) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            route: String::new(),
            query: Query::new(),
            method: Method::GET,
        }
    }

    /// Set the HTTP method to be used in the request
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Set the route to be used in the request
    pub fn route(mut self, route: impl ToString) -> Self {
        self.route = route.to_string();
        self
    }

    /// Set the query to be used in the request
    pub fn query(mut self, query: Query) -> Self {
        self.query = query;
        self
    }

    /// Convert the request URL to a URL
    /// that can be used in a request (Contains the query with pagination)
    pub fn as_url<P: Pagination, F: Filter, S: Sort, R: Range>(
        &self,
        pagination: &P,
        filter: &F,
        sort: &S,
        range: &R,
    ) -> Result<Url>
    where
        Query: for<'a> From<&'a F> + for<'a> From<&'a S> + for<'a> From<&'a R>,
    {
        let mut query = self.query.clone();

        query = query.join(pagination.get_current_page());
        query = query.join(filter.into());
        query = query.join(sort.into());
        query = query.join(range.into());

        Url::parse(&format!("{}{}{}", self.endpoint, self.route, query))
            .map_err(ApiError::WrongUrlFormat)
    }
}
