use std::fmt::Display;

use reqwest::{Method, Url};

use crate::prelude::{ApiError, Query};

#[derive(Debug, Clone)]
pub struct RequestUrl {
    pub(crate) endpoint: String,
    pub(crate) route: String,
    pub(crate) query: Query,
    pub(crate) current_page_index: u32,
    pub(crate) method: Method,
}
impl RequestUrl {
    pub fn new(endpoint: impl ToString) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            route: String::new(),
            query: Query::build(),
            current_page_index: 0,
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

    pub fn request_next_page(&mut self) {
        self.current_page_index += 1;
    }

    pub fn as_url(&self) -> Result<Url, ApiError> {
        Url::parse(&self.to_string()).map_err(|e| ApiError::WrongUrlFormat(e))
    }
}
impl Display for RequestUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut query = self.query.clone();
        if self.current_page_index > 0 {
            query = query.add("page[number]", self.current_page_index);
        }
        write!(f, "{}{}{}", self.endpoint, self.route, query)
    }
}
