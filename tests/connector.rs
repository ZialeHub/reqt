#[cfg(test)]
mod connector_tests {
    use base64::{Engine, engine::general_purpose};
    use reqt::{Keycloak, prelude::*};
    use reqwest::{Client, StatusCode};
    use serde::Deserialize;
    use std::collections::HashMap;

    fn get_credentials_oauth2() -> TestApiOauth2Connector {
        TestApiOauth2Connector {
            client_id: std::env!("REQT_OAUTH2_CLIENT_ID").to_string(),
            client_secret: std::env!("REQT_OAUTH2_CLIENT_SECRET").to_string(),
            auth_endpoint: std::env!("REQT_OAUTH2_AUTH_ENDPOINT").to_string(),
            scopes: std::env!("REQT_OAUTH2_SCOPES")
                .split(',')
                .map(|s| s.to_string())
                .collect(),
        }
    }

    fn get_credentials_keycloak() -> TestApiKeycloakConnector {
        let user_pass = String::from_utf8(
            general_purpose::STANDARD
                .decode(std::env!("REQT_KEYCLOAK_USER_PASS").to_string())
                .unwrap(),
        )
        .unwrap();
        TestApiKeycloakConnector {
            client_id: std::env!("REQT_KEYCLOAK_CLIENT_ID").to_string(),
            client_secret: std::env!("REQT_KEYCLOAK_CLIENT_SECRET").to_string(),
            auth_endpoint: std::env!("REQT_KEYCLOAK_AUTH_ENDPOINT").to_string(),
            realm: std::env!("REQT_KEYCLOAK_REALM").to_string(),
            user_login: std::env!("REQT_KEYCLOAK_USER_LOGIN").to_string(),
            user_pass,
        }
    }

    #[derive(Debug, Clone, Deserialize, Oauth2)]
    #[filter(FilterTest)]
    #[sort(SortTest)]
    #[range(RangeTest)]
    struct TestApiOauth2Connector {
        client_id: String,
        client_secret: String,
        auth_endpoint: String,
        scopes: Vec<String>,
    }

    #[derive(Debug, Clone, Deserialize, Keycloak)]
    #[auth_type(OAuth2)]
    struct TestApiKeycloakConnector {
        client_id: String,
        client_secret: String,
        auth_endpoint: String,
        realm: String,
        user_login: String,
        user_pass: String,
    }

    #[derive(Authorization)]
    struct TestApiNoAuthConnector {}
    impl TestApiNoAuthConnector {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[derive(Debug, Clone, Default)]
    struct RangeTest {
        pub pattern: String,
        pub ranges: Vec<(String, String)>,
    }
    impl From<&RangeTest> for Query {
        fn from(value: &RangeTest) -> Self {
            let mut query = Query::new();
            for (range, values) in value.ranges.iter() {
                query = query.add(range, values);
            }
            query
        }
    }
    impl Range for RangeTest {
        fn pattern(mut self, pattern: impl ToString) -> Self {
            self.pattern = pattern.to_string();
            self
        }

        fn range(
            mut self,
            property: impl ToString,
            min: impl ToString,
            max: impl ToString,
        ) -> Self {
            let mut range = self.pattern.clone();
            range = range.replace("property", &property.to_string());
            let values = format!("{},{}", min.to_string(), max.to_string());
            if let Some(old_range) = self.ranges.iter_mut().find(|(r, _)| r == &range) {
                old_range.1 = values;
                return self;
            }
            self.ranges.push((range, values));
            self
        }
    }

    #[derive(Debug, Clone, Default)]
    struct SortTest {
        pub pattern: String,
        pub sorts: Vec<String>,
    }
    impl From<&SortTest> for Query {
        fn from(value: &SortTest) -> Self {
            let mut query = Query::new();
            let mut sorts = String::new();
            for sort in value.sorts.iter() {
                sorts.push_str(sort);
                sorts.push(',');
            }
            sorts.pop();
            if !sorts.is_empty() {
                query = query.add("sort", sorts);
            }
            query
        }
    }
    impl Sort for SortTest {
        fn sort(mut self, property: impl ToString) -> Self {
            let mut sort = self.pattern.clone();
            sort = sort.replace("property", &property.to_string());

            if let Some(old_sort) = self.sorts.iter_mut().find(|s| {
                s == &&sort
                    || s == &&format!("-{sort}")
                    || s == &&format!("+{sort}")
                    || sort == format!("+{s}")
                    || sort == format!("-{s}")
            }) {
                *old_sort = sort;
                return self;
            }
            self.sorts.push(sort);
            self
        }

        fn sort_with(mut self, property: impl ToString, order: SortOrder) -> Self {
            let mut sort = self.pattern.clone();
            sort = sort.replace("property", &property.to_string());
            sort = sort.replace("order", &order.to_string());

            if let Some(old_sort) = self.sorts.iter_mut().find(|s| s == &&sort) {
                *old_sort = sort;
                return self;
            }
            self.sorts.push(sort);
            self
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
    impl From<&FilterTest> for Query {
        fn from(value: &FilterTest) -> Self {
            let mut query = Query::new();
            for (filter, values) in value.filters.iter() {
                query = query.add(filter, values);
            }
            query
        }
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
            if let Some(old_filter) = self.filters.iter_mut().find(|(f, _)| f == &filter) {
                old_filter.1 = values;
                return self;
            }
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
            if let Some(old_filter) = self.filters.iter_mut().find(|(f, _)| f == &filters) {
                old_filter.1 = values;
                return self;
            }
            self.filters.push((filters, values));
            self
        }

        fn pattern(mut self, pattern: impl ToString) -> Self {
            self.pattern = pattern.to_string();
            self
        }
    }

    #[tokio::test]
    async fn keycloak_connector() -> Result<()> {
        let data_connector = get_credentials_keycloak();
        let _ = data_connector
            .connect("https://chronos.42.fr/api/v1/")
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn oauth2_connector() -> Result<()> {
        let data_connector = get_credentials_oauth2();
        let _ = data_connector
            .connect("https://api.intra.42.fr/v2/")
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn no_auth_connector() -> Result<()> {
        let _ = TestApiNoAuthConnector::new()
            .connect("https://restcountries.com/v3.1/")
            .await?;
        Ok(())
    }
}
