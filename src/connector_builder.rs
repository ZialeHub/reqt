use crate::connector::{ApiConnector, PaginationRule, TokenType};

#[derive(Debug, Clone)]
pub struct ApiConnectorBuilder {
    pub(crate) token_type: TokenType,
    pub(crate) token: String,
    pub(crate) endpoint: String,
    pub(crate) pagination: PaginationRule,
}
impl ApiConnectorBuilder {
    pub fn new(endpoint: impl ToString) -> Self {
        Self {
            token_type: TokenType::None,
            token: String::new(),
            endpoint: endpoint.to_string(),
            pagination: PaginationRule::None,
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
        self.pagination = pagination;
        self
    }

    pub fn build(self) -> ApiConnector {
        ApiConnector {
            token_type: self.token_type,
            token: self.token,
            endpoint: self.endpoint,
            pagination: self.pagination,
        }
    }
}
