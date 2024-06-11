#[cfg(test)]
mod tests_api42_v2 {
    use std::collections::HashMap;

    use reqwest::{Client, StatusCode};
    use serde::{Deserialize, Serialize};

    use crate::{prelude::*, sort::SortOrder};
    use authorization_derive::Oauth2;

    fn get_credentials() -> TestApiConnector {
        let connector: TestApiConnector = toml::from_str::<Env>(include_str!("../.env"))
            .unwrap()
            .intra_v2;
        connector
    }

    #[derive(Debug, Clone, Deserialize)]
    struct TokenResponse {
        pub access_token: String,
    }

    #[derive(Debug, Clone, Deserialize, Oauth2)]
    #[pagination(PaginationTest)]
    #[filter(FilterTest)]
    #[sort(SortTest)]
    struct TestApiConnector {
        uid: String,
        secret: String,
        auth_endpoint: String,
        scopes: Vec<String>,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct Env {
        intra_v2: TestApiConnector,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct User {
        pub login: String,
    }

    #[derive(Debug, Clone, Default)]
    struct SortTest {
        pub pattern: String,
        pub sorts: Vec<String>,
    }
    impl Sort for SortTest {
        fn sort(mut self, property: impl ToString) -> Self {
            let mut sort = self.pattern.clone();
            sort = sort.replace("property", &property.to_string());

            self.sorts.push(sort);
            self
        }

        fn sort_with(mut self, property: impl ToString, order: SortOrder) -> Self {
            let mut sort = self.pattern.clone();
            sort = sort.replace("property", &property.to_string());
            sort = sort.replace("order", &order.to_string());

            self.sorts.push(sort);
            self
        }

        fn to_query(&self) -> Query {
            let mut query = Query::new();
            let sorts = self.sorts.join(",");
            query = query.add("sort", sorts);
            query
        }

        fn pattern(mut self, pattern: impl ToString) -> Self {
            self.pattern = pattern.to_string();
            self
        }
    }

    #[derive(Debug, Clone, Default)]
    struct FilterTest {
        pub pattern: String,
        pub filters: Vec<(String, String)>,
    }
    impl Filter for FilterTest {
        fn filter<T: IntoIterator>(mut self, property: impl ToString, value: T) -> Self
        where
            T::Item: ToString,
        {
            let mut filter = self.pattern.clone();
            let mut values = String::new();
            filter = filter.replace("property", &property.to_string());

            for v in value.into_iter() {
                values.push_str(&v.to_string());
                values.push(',');
            }
            values.pop();
            self.filters.push((filter, values));
            self
        }

        fn filter_with<T: IntoIterator>(
            mut self,
            property: impl ToString,
            filter: impl ToString,
            value: T,
        ) -> Self
        where
            T::Item: ToString,
        {
            let mut filters = self.pattern.clone();
            let mut values = String::new();
            filters = filters.replace("property", &property.to_string());
            filters = filters.replace("filter", &filter.to_string());

            for v in value.into_iter() {
                values.push_str(&v.to_string());
                values.push(',');
            }
            values.pop();
            self.filters.push((filters, values));
            self
        }

        fn to_query(&self) -> Query {
            let mut query = Query::new();
            for (filter, values) in self.filters.iter() {
                query = query.add(filter, values);
            }
            query
        }

        fn pattern(mut self, pattern: impl ToString) -> Self {
            self.pattern = pattern.to_string();
            self
        }
    }

    #[derive(Debug, Clone)]
    struct PaginationTest {
        pub size: usize,
        pub current_page: usize,
        pub pagination: PaginationRule,
    }
    impl Default for PaginationTest {
        fn default() -> Self {
            Self {
                size: 100,
                current_page: 1,
                pagination: PaginationRule::default(),
            }
        }
    }
    impl Pagination for PaginationTest {
        fn size(mut self, size: usize) -> Self {
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

        fn current_page(&self) -> usize {
            self.current_page
        }

        fn get_current_page(&self) -> Query {
            Query::new()
                .add("page[number]", self.current_page)
                .add("page[size]", self.size)
        }

        fn get_size(&self) -> Query {
            Query::new().add("page[size]", self.size)
        }

        fn next(&mut self) {
            self.current_page += 1;
        }

        fn get_next_page(&mut self) -> Query {
            self.current_page += 1;
            Query::new()
                .add("page[number]", self.current_page)
                .add("page[size]", self.size)
        }

        fn reset(&mut self) {
            self.current_page = 1;
        }
    }

    // #[tokio::test]
    // async fn get_method_default() -> Result<()> {
    //     let data_connector = get_credentials();
    //     let connector = data_connector
    //         .connect("https://api.intra.42.fr/v2/")
    //         .await?;
    //     let mut request = connector.get("users", Query::new())?;
    //     let response = request.send::<Vec<User>>().await?;
    //     assert_eq!(response.len(), connector.pagination.size);
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn connector_none_pagination() -> Result<()> {
    //     let data_connector = get_credentials();
    //     let connector = data_connector
    //         .connect("https://api.intra.42.fr/v2/")
    //         .await?
    //         .pagination(PaginationRule::default());
    //     let mut request =
    //         connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
    //     let response = request.send::<Vec<User>>().await?;
    //     assert_eq!(response.len(), connector.pagination.size);
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn connector_fixed_pagination() -> Result<()> {
    //     let data_connector = get_credentials();
    //     let connector = data_connector
    //         .connect("https://api.intra.42.fr/v2/")
    //         .await?
    //         .pagination(PaginationRule::Fixed(3));
    //     let mut request =
    //         connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
    //     let response = request.send::<Vec<User>>().await?;
    //     assert_eq!(response.len(), connector.pagination.size * 3);
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn connector_one_shot_pagination() -> Result<()> {
    //     let data_connector = get_credentials();
    //     let connector = data_connector
    //         .connect("https://api.intra.42.fr/v2/")
    //         .await?
    //         .pagination(PaginationRule::OneShot);
    //     let mut request =
    //         connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
    //     let response = request.send::<Vec<User>>().await?;
    //     assert_eq!(response.len(), 761);
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn request_none_pagination_override() -> Result<()> {
    //     let data_connector = get_credentials();
    //     let connector = data_connector
    //         .connect("https://api.intra.42.fr/v2/")
    //         .await?
    //         .pagination(PaginationRule::OneShot);
    //     let mut request = connector
    //         .get("users", Query::new().add("filter[primary_campus_id]", 31))?
    //         .pagination(PaginationRule::default());
    //     let response = request.send::<Vec<User>>().await?;
    //     assert_eq!(response.len(), connector.pagination.size);
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn request_fixed_pagination_override() -> Result<()> {
    //     let data_connector = get_credentials();
    //     let connector = data_connector
    //         .connect("https://api.intra.42.fr/v2/")
    //         .await?;
    //     let mut request = connector
    //         .get("users", Query::new().add("filter[primary_campus_id]", 31))?
    //         .pagination(PaginationRule::Fixed(3));
    //     let response = request.send::<Vec<User>>().await?;
    //     assert_eq!(response.len(), connector.pagination.size * 3);
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn request_one_shot_pagination_override() -> Result<()> {
    //     let data_connector = get_credentials();
    //     let connector = data_connector
    //         .connect("https://api.intra.42.fr/v2/")
    //         .await?;
    //     let mut request = connector
    //         .get("users", Query::new().add("filter[primary_campus_id]", 31))?
    //         .pagination(PaginationRule::OneShot);
    //     let response = request.send::<Vec<User>>().await?;
    //     assert_eq!(response.len(), 761);
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn request_consistency() -> Result<()> {
    //     let data_connector = get_credentials();
    //     let connector = data_connector
    //         .connect("https://api.intra.42.fr/v2/")
    //         .await?;
    //     let mut request = connector
    //         .get("users", Query::new().add("filter[primary_campus_id]", 31))?
    //         .pagination(PaginationRule::Fixed(2));
    //     let first_response = request.send::<Vec<User>>().await?;
    //     assert_eq!(first_response.len(), connector.pagination.size * 2);
    //     let second_response = request.send::<Vec<User>>().await?;
    //     assert_eq!(second_response.len(), connector.pagination.size * 2);
    //     assert_eq!(request.pagination.current_page(), 5);
    //     assert!(first_response
    //         .iter()
    //         .zip(second_response.iter())
    //         .all(|(a, b)| a.login != b.login));
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn request_consistency_after_reset() -> Result<()> {
    //     let data_connector = get_credentials();
    //     let connector = data_connector
    //         .connect("https://api.intra.42.fr/v2/")
    //         .await?;
    //     let mut request = connector
    //         .get("users", Query::new().add("filter[primary_campus_id]", 31))?
    //         .pagination(PaginationRule::Fixed(2));
    //     let first_response = request.send::<Vec<User>>().await?;
    //     assert_eq!(first_response.len(), connector.pagination.size * 2);
    //     request.pagination.reset();
    //     let second_response = request.send::<Vec<User>>().await?;
    //     assert_eq!(second_response.len(), connector.pagination.size * 2);
    //     assert_eq!(request.pagination.current_page(), 3);
    //     assert!(first_response
    //         .iter()
    //         .zip(second_response.iter())
    //         .all(|(a, b)| a.login == b.login));
    //     Ok(())
    // }

    #[tokio::test]
    async fn connector_none_pagination_sort_login_asc() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .pattern_sort("property")
            .sort("login");
        let mut request = connector.get("users", Query::new())?;
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination_sort_login_desc() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .pattern_sort("property")
            .sort("-login");
        let mut request = connector.get("users", Query::new())?;
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination_sort_reset() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .pattern_sort("property")
            .sort("login");
        let mut request = connector
            .get("users", Query::new())?
            .set_sort(SortTest::default());
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination_request_sort_login_asc() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"]);
        let mut request = connector
            .get("users", Query::new())?
            .pattern_sort("property")
            .sort("login");
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination_request_sort_login_desc() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"]);
        let mut request = connector
            .get("users", Query::new())?
            .pattern_sort("property")
            .sort("-login");
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination_override_sort_login_asc() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .pattern_sort("property")
            .sort("-login");
        let mut request = connector
            .get("users", Query::new())?
            .pattern_sort("property")
            .sort("login");
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination_override_sort_login_desc() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .pattern_sort("property")
            .sort("login");
        let mut request = connector
            .get("users", Query::new())?
            .pattern_sort("property")
            .sort("-login");
        let response = request.send::<Vec<User>>().await?;
        eprintln!("{:?}", response);
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination_filter() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .filter("login", vec!["vnaud"]);
        let mut request = connector.get("users", Query::new())?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination_request_filter() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default());
        let mut request = connector
            .get("users", Query::new())?
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .filter("login", vec!["vnaud"]);
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination_filter_override() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .filter("login", vec!["vnaud"]);
        let mut request = connector
            .get("users", Query::new())?
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .filter("login", vec!["vnaud,pmieuzet"]);
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination_filter_reset() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .filter("login", vec!["vnaud"]);
        let mut request = connector
            .get("users", Query::new())?
            .set_filter(FilterTest::default());
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), request.pagination.size);
        Ok(())
    }
}

// #[cfg(test)]
// mod tests_rest_country {
//     use serde::{Deserialize, Serialize};
//
//     use crate::prelude::*;
//
//     #[derive(proc_macro_api_manager::Authorization)]
//     struct CountryConnector {}
//     impl CountryConnector {
//         pub fn new() -> Self {
//             Self {}
//         }
//     }
//
//     #[derive(Debug, Serialize, Deserialize)]
//     struct CountryNameBis {
//         pub common: String,
//         pub official: String,
//     }
//
//     #[derive(Debug, Serialize, Deserialize)]
//     struct CountryName {
//         pub name: CountryNameBis,
//     }
//
//     #[derive(Debug, Serialize, Deserialize)]
//     struct Country {
//         pub name: CountryName,
//     }
//
//     #[tokio::test]
//     async fn get_france() -> Result<()> {
//         let connector = CountryConnector::new()
//             .connect("https://restcountries.com/v3.1/")
//             .await?;
//         let mut request = connector.get("name/france", Query::new().add("fields", "name"))?;
//         let response = request.send::<Country>().await?;
//         assert_eq!(response.name.name.common, "France");
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn get_all() -> Result<()> {
//         let connector = CountryConnector::new()
//             .connect("https://restcountries.com/v3.1/")
//             .await?;
//         let mut request = connector.get("all", Query::new().add("fields", "name"))?;
//         let response = request.send::<Vec<serde_json::Value>>().await?;
//         assert_eq!(response.len(), 250);
//         Ok(())
//     }
// }
//
// #[cfg(test)]
// mod tests_api42_v3 {
//     use std::collections::HashMap;
//
//     use base64::{engine::general_purpose, Engine as _};
//     use reqwest::{Client, StatusCode};
//     use serde::{Deserialize, Serialize};
//
//     use crate::prelude::*;
//
//     fn get_credentials() -> TestApiConnectorV3 {
//         let connector: TestApiConnectorV3 = toml::from_str::<Env>(include_str!("../.env"))
//             .unwrap()
//             .intra_v3;
//         connector
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     struct Attendance {
//         user_id: i32,
//     }
//
//     #[derive(Debug, Clone, Deserialize)]
//     struct TokenResponse {
//         pub access_token: String,
//     }
//
//     #[derive(Debug, Clone, Deserialize)]
//     struct TestApiConnectorV3 {
//         uid: String,
//         secret: String,
//         auth_endpoint: String,
//         realm: String,
//         user_login: String,
//         user_pass: String,
//     }
//
//     #[derive(Debug, Clone, Deserialize)]
//     struct Env {
//         intra_v3: TestApiConnectorV3,
//     }
//
//     impl Authorization for TestApiConnectorV3 {
//         async fn connect(&self, url: &str) -> Result<Api<RequestPagination>> {
//             let connector = ApiBuilder::new(url);
//             let client = Client::new();
//
//             let auth_header = format!(
//                 "Basic {}",
//                 general_purpose::STANDARD_NO_PAD.encode(format!("{}:{}", &self.uid, &self.secret))
//             );
//             let mut params = HashMap::new();
//             params.insert("grant_type", "password");
//             params.insert("username", &self.user_login);
//             params.insert("password", &self.user_pass);
//             match client
//                 .post(format!(
//                     "{}realms/{}/protocol/openid-connect/token",
//                     self.auth_endpoint, self.realm
//                 ))
//                 .header("Content-Type", "application/x-www-form-urlencoded")
//                 .header("Authorization", auth_header)
//                 .form(&params)
//                 .send()
//                 .await
//             {
//                 Ok(response) => {
//                     eprintln!("{:?}", response);
//                     match response.status() {
//                         StatusCode::OK
//                         | StatusCode::CREATED
//                         | StatusCode::ACCEPTED
//                         | StatusCode::NO_CONTENT => {}
//                         status => return Err(status.into()),
//                     }
//                     match response.text().await {
//                         Ok(response_text) => {
//                             let token: TokenResponse =
//                                 serde_json::from_str(&response_text).unwrap();
//                             Ok(connector.bearer(token.access_token).build())
//                         }
//                         Err(e) => Err(ApiError::ResponseToText(e)),
//                     }
//                 }
//                 Err(e) => Err(ApiError::ReqwestExecute(e)),
//             }
//         }
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     struct User {
//         pub login: String,
//     }
//
//     #[tokio::test]
//     async fn get_method_default() -> Result<()> {
//         let data_connector = get_credentials();
//         let connector = data_connector
//             .connect("https://chronos.42.fr/api/v1/")
//             .await?;
//         let mut request = connector.get("users/vnaud/attendances", Query::new())?;
//         let response = request.send::<Vec<Attendance>>().await?;
//         response.iter().all(|attendance| {
//             assert_eq!(attendance.user_id, 108323);
//             true
//         });
//         Ok(())
//     }
// }
