use reqwest::{Method, Url};

use crate::{
    error::{ApiError, Result},
    pagination::Pagination,
    query::Query,
};

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

    pub fn as_url<U: Pagination + Clone>(&self, pagination: &U) -> Result<Url> {
        let mut query = self.query.clone();

        if pagination.current_page() > 0 {
            query = query.join(pagination.get_current_page());
        }

        Url::parse(&format!("{}{}{}", self.endpoint, self.route, query))
            .map_err(|e| ApiError::WrongUrlFormat(e))
    }
}
