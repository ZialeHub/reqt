use crate::{
    connector::{Api, Authorization},
    pagination::{Pagination, PaginationRule, RequestPagination},
};

#[derive(Debug, Clone)]
pub struct ApiConnectorBuilder<T: Pagination = RequestPagination> {
    pub(crate) authorization: Authorization,
    pub(crate) endpoint: String,
    pub(crate) pagination: T,
}

impl<T: Pagination + Clone> ApiConnectorBuilder<T> {
    pub fn new(endpoint: impl ToString, pagination: T) -> Self {
        Self {
            authorization: Authorization::None,
            endpoint: endpoint.to_string(),
            pagination,
        }
    }

    pub fn endpoint(mut self, endpoint: impl ToString) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }

    pub fn bearer(mut self, token: impl ToString) -> Self {
        self.authorization = Authorization::Bearer(token.to_string());
        self
    }

    pub fn oauth2(mut self, token: impl ToString) -> Self {
        self.authorization = Authorization::OAuth2(token.to_string());
        self
    }

    pub fn basic(mut self, token: impl ToString) -> Self {
        self.authorization = Authorization::Basic(token.to_string());
        self
    }

    pub fn api_key(mut self, token: impl ToString) -> Self {
        self.authorization = Authorization::ApiKey(token.to_string());
        self
    }

    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.pagination(pagination);
        self
    }

    pub fn build(self) -> Api<T> {
        Api {
            authorization: self.authorization,
            endpoint: self.endpoint,
            pagination: self.pagination,
        }
    }
}
