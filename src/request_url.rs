use reqwest::{Method, Url};

use crate::{
    error::{ApiError, Result},
    pagination::Pagination,
    query::Query,
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

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn route(mut self, route: impl ToString) -> Self {
        self.route = route.to_string();
        self
    }

    pub fn query(mut self, query: Query) -> Self {
        self.query = query;
        self
    }

    /// Convert the request URL to a URL
    /// that can be used in a request (Contains the query with pagination)
    pub fn as_url<U: Pagination>(&self, pagination: &U) -> Result<Url> {
        let mut query = self.query.clone();

        query = query.join(pagination.get_current_page());

        Url::parse(&format!("{}{}{}", self.endpoint, self.route, query))
            .map_err(ApiError::WrongUrlFormat)
    }
}
