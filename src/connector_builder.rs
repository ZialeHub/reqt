use crate::{
    connector::{Api, AuthorizationType},
    filter::{Filter, FilterRule},
    pagination::{Pagination, PaginationRule, RequestPagination},
    sort::{Sort, SortRule},
};

#[derive(Debug, Clone)]
pub struct ApiBuilder<P: Pagination = RequestPagination, F: Filter = FilterRule, S: Sort = SortRule>
{
    pub(crate) authorization: AuthorizationType,
    pub(crate) endpoint: String,
    pub(crate) pagination: P,
    pub(crate) filter: F,
    pub(crate) sort: S,
}

impl<P: Pagination, F: Filter, S: Sort> ApiBuilder<P, F, S> {
    pub fn new(endpoint: impl ToString) -> Self {
        Self {
            authorization: AuthorizationType::None,
            endpoint: endpoint.to_string(),
            pagination: P::default(),
            filter: F::default(),
            sort: S::default(),
        }
    }

    pub fn endpoint(mut self, endpoint: impl ToString) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }

    pub fn bearer(mut self, token: impl ToString) -> Self {
        self.authorization = AuthorizationType::Bearer(token.to_string());
        self
    }

    pub fn oauth2(mut self, token: impl ToString) -> Self {
        self.authorization = AuthorizationType::OAuth2(token.to_string());
        self
    }

    pub fn basic(mut self, token: impl ToString) -> Self {
        self.authorization = AuthorizationType::Basic(token.to_string());
        self
    }

    pub fn api_key(mut self, token: impl ToString) -> Self {
        self.authorization = AuthorizationType::ApiKey(token.to_string());
        self
    }

    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.pagination(pagination);
        self
    }

    pub fn filter(mut self, filter: F) -> Self {
        self.filter = filter;
        self
    }

    pub fn sort(mut self, sort: S) -> Self {
        self.sort = sort;
        self
    }

    pub fn build(self) -> Api<P, F, S> {
        Api {
            authorization: self.authorization,
            endpoint: self.endpoint,
            pagination: self.pagination,
            filter: F::default(),
            sort: S::default(),
        }
    }
}
