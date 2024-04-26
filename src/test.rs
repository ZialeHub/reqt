#[cfg(test)]
mod tests_api42_v2 {
    use std::collections::HashMap;

    use reqwest::{Client, StatusCode};
    use serde::{Deserialize, Serialize};

    use crate::prelude::*;

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

    #[derive(Debug, Clone, Deserialize)]
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

    impl Authentication for TestApiConnector {
        async fn connect(&self, url: &str) -> Result<Api<RequestPagination>> {
            let pagination = RequestPagination::default();
            let connector = ApiBuilder::new(url, pagination);
            let client = Client::new();

            let scopes = self
                .scopes
                .iter()
                .fold(String::new(), |acc, scope| format!("{} {}", acc, scope));
            let mut params = HashMap::new();
            params.insert("grant_type", "client_credentials");
            params.insert("client_id", &self.uid);
            params.insert("client_secret", &self.secret);
            params.insert("scope", &scopes);
            match client
                .post(&self.auth_endpoint)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .form(&params)
                .send()
                .await
            {
                Ok(response) => {
                    match response.status() {
                        StatusCode::OK
                        | StatusCode::CREATED
                        | StatusCode::ACCEPTED
                        | StatusCode::NO_CONTENT => {}
                        status => return Err(status.into()),
                    }
                    match response.text().await {
                        Ok(response_text) => {
                            let token: TokenResponse =
                                serde_json::from_str(&response_text).unwrap();
                            Ok(connector.bearer(token.access_token).build())
                        }
                        Err(e) => Err(ApiError::ResponseToText(e)),
                    }
                }
                Err(e) => Err(ApiError::ReqwestExecute(e)),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct User {
        pub login: String,
    }

    #[tokio::test]
    async fn get_method_default() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let mut request = connector.get("users", Query::new())?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), connector.pagination.size);
        Ok(())
    }

    #[tokio::test]
    async fn connector_none_pagination() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::default());
        let mut request =
            connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), connector.pagination.size);
        Ok(())
    }

    #[tokio::test]
    async fn connector_fixed_pagination() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::Fixed(3));
        let mut request =
            connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), connector.pagination.size * 3);
        Ok(())
    }

    #[tokio::test]
    async fn connector_one_shot_pagination() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::OneShot);
        let mut request =
            connector.get("users", Query::new().add("filter[primary_campus_id]", 31))?;
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 761);
        Ok(())
    }

    #[tokio::test]
    async fn request_none_pagination_override() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?
            .pagination(PaginationRule::OneShot);
        let mut request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::default());
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), connector.pagination.size);
        Ok(())
    }

    #[tokio::test]
    async fn request_fixed_pagination_override() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let mut request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::Fixed(3));
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), connector.pagination.size * 3);
        Ok(())
    }

    #[tokio::test]
    async fn request_one_shot_pagination_override() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let mut request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::OneShot);
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), 761);
        Ok(())
    }

    #[tokio::test]
    async fn request_consistency() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let mut request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::Fixed(2));
        let first_response = request.send::<Vec<User>>().await?;
        assert_eq!(first_response.len(), connector.pagination.size * 2);
        let second_response = request.send::<Vec<User>>().await?;
        assert_eq!(second_response.len(), connector.pagination.size * 2);
        assert_eq!(request.pagination.current_page(), 5);
        assert!(first_response
            .iter()
            .zip(second_response.iter())
            .all(|(a, b)| a.login != b.login));
        Ok(())
    }

    #[tokio::test]
    async fn request_consistency_after_reset() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        let mut request = connector
            .get("users", Query::new().add("filter[primary_campus_id]", 31))?
            .pagination(PaginationRule::Fixed(2));
        let first_response = request.send::<Vec<User>>().await?;
        assert_eq!(first_response.len(), connector.pagination.size * 2);
        request.pagination.reset();
        let second_response = request.send::<Vec<User>>().await?;
        assert_eq!(second_response.len(), connector.pagination.size * 2);
        assert_eq!(request.pagination.current_page(), 3);
        assert!(first_response
            .iter()
            .zip(second_response.iter())
            .all(|(a, b)| a.login == b.login));
        Ok(())
    }
}

#[cfg(test)]
mod tests_rest_country {
    use serde::{Deserialize, Serialize};

    use crate::prelude::{
        Api, ApiBuilder, Authentication, Connector, Query, RequestPagination, Result,
    };

    struct CountryConnector {}
    impl CountryConnector {
        pub fn new() -> Self {
            Self {}
        }
    }
    impl Authentication for CountryConnector {
        async fn connect(&self, url: &str) -> Result<Api<RequestPagination>> {
            Ok(ApiBuilder::new(url, RequestPagination::default()).build())
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CountryNameBis {
        pub common: String,
        pub official: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CountryName {
        pub name: CountryNameBis,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Country {
        pub name: CountryName,
    }

    #[tokio::test]
    async fn get_france() -> Result<()> {
        let connector = CountryConnector::new()
            .connect("https://restcountries.com/v3.1/")
            .await?;
        let mut request = connector.get("name/france", Query::new().add("fields", "name"))?;
        let response = request.send::<Country>().await?;
        assert_eq!(response.name.name.common, "France");
        Ok(())
    }

    #[tokio::test]
    async fn get_all() -> Result<()> {
        let connector = CountryConnector::new()
            .connect("https://restcountries.com/v3.1/")
            .await?;
        let mut request = connector.get("all", Query::new().add("fields", "name"))?;
        let response = request.send::<Vec<serde_json::Value>>().await?;
        assert_eq!(response.len(), 250);
        Ok(())
    }
}

#[cfg(test)]
mod tests_api42_v3 {
    use std::collections::HashMap;

    use base64::{engine::general_purpose, Engine as _};
    use reqwest::{Client, StatusCode};
    use serde::{Deserialize, Serialize};

    use crate::prelude::*;

    fn get_credentials() -> TestApiConnectorV3 {
        let connector: TestApiConnectorV3 = toml::from_str::<Env>(include_str!("../.env"))
            .unwrap()
            .intra_v3;
        connector
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Attendance {
        user_id: i32,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct TokenResponse {
        pub access_token: String,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct TestApiConnectorV3 {
        uid: String,
        secret: String,
        auth_endpoint: String,
        realm: String,
        user_login: String,
        user_pass: String,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct Env {
        intra_v3: TestApiConnectorV3,
    }

    impl Authentication for TestApiConnectorV3 {
        async fn connect(&self, url: &str) -> Result<Api<RequestPagination>> {
            let pagination = RequestPagination::default();
            let connector = ApiBuilder::new(url, pagination);
            let client = Client::new();

            let auth_header = format!(
                "Basic {}",
                general_purpose::STANDARD_NO_PAD.encode(format!("{}:{}", &self.uid, &self.secret))
            );
            let mut params = HashMap::new();
            params.insert("grant_type", "password");
            params.insert("username", &self.user_login);
            params.insert("password", &self.user_pass);
            match client
                .post(format!(
                    "{}realms/{}/protocol/openid-connect/token",
                    self.auth_endpoint, self.realm
                ))
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Authorization", auth_header)
                .form(&params)
                .send()
                .await
            {
                Ok(response) => {
                    eprintln!("{:?}", response);
                    match response.status() {
                        StatusCode::OK
                        | StatusCode::CREATED
                        | StatusCode::ACCEPTED
                        | StatusCode::NO_CONTENT => {}
                        status => return Err(status.into()),
                    }
                    match response.text().await {
                        Ok(response_text) => {
                            let token: TokenResponse =
                                serde_json::from_str(&response_text).unwrap();
                            Ok(connector.bearer(token.access_token).build())
                        }
                        Err(e) => Err(ApiError::ResponseToText(e)),
                    }
                }
                Err(e) => Err(ApiError::ReqwestExecute(e)),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct User {
        pub login: String,
    }

    #[tokio::test]
    async fn get_method_default() -> Result<()> {
        let data_connector = get_credentials();
        let connector = data_connector
            .connect("https://chronos.42.fr/api/v1/")
            .await?;
        let mut request = connector.get("users/vnaud/attendances", Query::new())?;
        let response = request.send::<Vec<Attendance>>().await?;
        response.iter().all(|attendance| {
            assert_eq!(attendance.user_id, 108323);
            true
        });
        Ok(())
    }
}
