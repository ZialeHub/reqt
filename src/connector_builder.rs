use crate::{
    connector::TokenType,
    pagination::{Pagination, PaginationRule, RequestPagination},
    prelude::ApiConnector,
};

#[derive(Debug, Clone)]
pub struct ApiConnectorBuilder<T: Pagination = RequestPagination> {
    pub(crate) token_type: TokenType,
    pub(crate) token: String,
    pub(crate) endpoint: String,
    pub(crate) pagination: T,
}
impl<T: Pagination + Clone> ApiConnectorBuilder<T> {
    pub fn new(endpoint: impl ToString, pagination: T) -> Self {
        Self {
            token_type: TokenType::None,
            token: String::new(),
            endpoint: endpoint.to_string(),
            pagination,
        }
    }

    pub fn token(mut self, token: impl ToString) -> Self {
        self.token = token.to_string();
        self
    }

    pub fn endpoint(mut self, endpoint: impl ToString) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }

    pub fn token_type(mut self, token_type: TokenType) -> Self {
        self.token_type = token_type;
        self
    }

    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.pagination(pagination);
        self
    }

    pub fn build(self) -> ApiConnector<T> {
        ApiConnector {
            token_type: self.token_type,
            token: self.token,
            endpoint: self.endpoint,
            pagination: self.pagination,
        }
    }
}
