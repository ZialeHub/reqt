[![Rust CI](https://github.com/ryse-rs/reqt/actions/workflows/ci.yaml/badge.svg)](https://github.com/ryse-rs/reqt/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/ryse-rs/reqt/graph/badge.svg?token=RRR841D0NT)](https://codecov.io/gh/ryse-rs/reqt)

**reqt** is an http request manager, providing high level features to connect to a web API and handle requests.

**reqt** goal is to make communication with web APIs easy.

**reqt** provides high level interfaces of the following features:

- [x] connection
- [x] authorization
- [x] requests (REST)
- [x] rate-limit (manual + automatic)
- [x] pagination
- [x] filters
- [x] range
- [x] sorting

# Connector

`Api<P: Pagination = RequestPagination, F: Filter = FilterRule, S: Sort = SortRule, R: Range = RangeRule>`\
[Api Connector](connector::Api) can be defined using the default Pagination, Filter, Sort and Range rules,
or by implementing your own types that implement the following traits:
- [Pagination](pagination::Pagination)
- [Filter](filter::Filter)
- [Sort](sort::Sort)
- [Range](range::Range)

Each rule can be overridden for a specific request if needed later.

# Authorization

[Authorization Type](connector::AuthorizationType) define how to use your token in each request:
- [None](connector::AuthorizationType::None) => No token
- [Basic](connector::AuthorizationType::Basic)\
username and password are Base64 encoded\
`Authorization: Basic username:password`
- [Bearer](connector::AuthorizationType::Bearer)\
`Authorization: Bearer <token>`
- [ApiKey](connector::AuthorizationType::ApiKey)\
`X-API-Key: 1234567890abcdef`
- [OAuth2](connector::AuthorizationType::OAuth2)\
`Authorization: Bearer <access_token>`\
`Authorization: Bearer <refresh_token>`
- [OIDC](connector::AuthorizationType::OIDC)
- [Keycloak](connector::AuthorizationType::Keycloak)

# Request

`Request<B: Serialize + Clone = (), P: Pagination = RequestPagination, F: Filter = FilterRule, S: Sort = SortRule, R: Range = RangeRule>`

The request allow you to override pagination, filter, sort and range rules from the connector.

# Pagination

Pagination defines the rule to manage multiple page requests depending on the API specifications.

By default [RequestPagination](pagination::RequestPagination) will be used and fields are set as follow:
- `size = 100` => Page size
- `current_page = 1`
- `pagination = PaginationRule::OneShot`

And the headers `page[number]=x` and `page[size]=y` are added to the final query of the request.

To implement your own pagination, you need to implement the [Pagination](pagination::Pagination) trait.

### [Pagination Rule](pagination::PaginationRule)
- [Fixed(X)](pagination::PaginationRule::Fixed)\
Where `X` is the number of page you want collected by one request
- [OneShot](pagination::PaginationRule::OneShot)

# Filter

Filter defines the way to filter resources with your request, and the list of filters you want to apply.

To implement your own filter rule, you need to implement the [Filter](filter::Filter) trait.

# Range

Range defines the way to range resources with your request, and the list of ranges you want to apply.

To implement your own range rule, you need to implement the [Range](range::Range) trait.

# Sort

Sort defines the way to sort resources with your request, and the list of sorts you want to apply.

To implement your own sort rule, you need to implement the [Sort](sort::Sort) trait.

### [SortOrder](sort::SortOrder)

- [Asc](sort::SortOrder::Asc)
- [Desc](sort::SortOrder::Desc)

# Rate limit

The rate limit can be set through the [ApiBuilder](connector_builder::ApiBuilder) (Default = `RateLimiter::new(1, TimePeriod::Second)`), and will allow the connector to respect a specific rate to avoid 429 HTTP errors.

# Derive Macros

To implement your own connector with ease, you have in your hands the following macros:

```rust,ignore
#[derive(Debug, Clone, Deserialize, Oauth2)]
#[pagination(PaginationTest)]
#[filter(FilterTest)]
#[sort(SortTest)]
#[range(RangeTest)]
struct TestApiConnector {
    client_id: String,
    client_secret: String,
    auth_endpoint: String,
    scopes: Vec<String>,
}
```
