use fake::{Dummy, Fake, Faker};
use httpmock::MockServer;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod request_tests {
    use reqt::prelude::*;
    use serial_test::serial;

    use super::*;

    const PAGINATION_SIZE: usize = 100;

    #[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
    struct User {
        id: u64,
        name: String,
        primary_campus_id: u64,
    }

    fn mock_server() -> MockServer {
        let server = MockServer::start();

        let mut users = Vec::<User>::new();
        for i in 0..1000 {
            let mut user: User = Faker.fake();
            user.id = i;
            user.primary_campus_id = if i % 31 == 0 { 31 } else { 35 };
            users.push(user);
        }
        users[992].name = String::from("jean");
        users[961].name = String::from("michel");
        users[930].name = String::from("pierre");
        users[899].name = String::from("jacques");
        users[868].name = String::from("francois");

        let jean_and_michel = vec![users[992].clone(), users[961].clone()];

        let mut asc_sorted_users = users.clone();
        asc_sorted_users.sort_by(|a, b| a.name.cmp(&b.name));

        let mut desc_sorted_users = asc_sorted_users.clone();
        desc_sorted_users.reverse();

        let users_from_31 = users
            .iter()
            .filter(|u| u.primary_campus_id == 31)
            .cloned()
            .collect::<Vec<User>>();

        let mut asc_sorted_users_31 = users_from_31.clone();
        asc_sorted_users_31.sort_by(|a, b| a.name.cmp(&b.name));

        let mut desc_sorted_users_31 = asc_sorted_users_31.clone();
        desc_sorted_users_31.reverse();

        server.mock(|when, then| {
            when.method("GET").path("/users/full");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&users);
        });
        server.mock(|when, then| {
            when.method("GET")
                .path("/users")
                .query_param("filter[name]", "jean,michel")
                .query_param("filter[primary_campus_id]", "31");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&jean_and_michel);
        });
        server.mock(|when, then| {
            when.method("GET")
                .path("/users")
                .query_param("filter[name]", "jean")
                .query_param("filter[primary_campus_id]", "31");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&vec![users[992].clone()]);
        });
        server.mock(|when, then| {
            when.method("GET")
                .path("/users")
                .query_param("filter[primary_campus_id]", "31")
                .query_param("sort", "name");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&asc_sorted_users_31);
        });
        server.mock(|when, then| {
            when.method("GET")
                .path("/users")
                .query_param("filter[primary_campus_id]", "31")
                .query_param("sort", "-name");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&desc_sorted_users_31);
        });
        server.mock(|when, then| {
            when.method("GET")
                .path("/users")
                .query_param("sort", "-name");
            then.status(200)
                .header("Content-Type", "application/json")
                .header("X-Total", users.len().to_string())
                .header("X-Per-Page", "100")
                .json_body_obj(&desc_sorted_users);
        });
        server.mock(|when, then| {
            when.method("GET")
                .path("/users")
                .query_param("sort", "name");
            then.status(200)
                .header("Content-Type", "application/json")
                .header("X-Total", users.len().to_string())
                .header("X-Per-Page", "100")
                .json_body_obj(&asc_sorted_users);
        });
        server.mock(|when, then| {
            when.method("GET")
                .path("/users")
                .query_param("filter[primary_campus_id]", "31");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&users_from_31);
        });
        server.mock(|when, then| {
            when.method("GET")
                .path("/users")
                .query_param("range[id]", "45,63");
            then.status(200)
                .header("Content-Type", "application/json")
                .header("X-Total", "18")
                .header("X-Per-Page", "100")
                .json_body_obj(
                    &users
                        .iter()
                        .filter(|u| u.id >= 45 && u.id < 63)
                        .cloned()
                        .collect::<Vec<User>>(),
                );
        });
        server.mock(|when, then| {
            when.method("GET")
                .path("/users")
                .query_param("range[id]", "546,736");
            then.status(200)
                .header("Content-Type", "application/json")
                .header("X-Total", "190")
                .header("X-Per-Page", "100")
                .json_body_obj(
                    &users
                        .iter()
                        .filter(|u| u.id >= 546 && u.id < 736)
                        .cloned()
                        .collect::<Vec<User>>(),
                );
        });
        for i in 0..(users.len() / 100) {
            server.mock(|when, then| {
                when.method("GET")
                    .path("/users")
                    .query_param("page[number]", (i + 1).to_string())
                    .query_param("page[size]", "100");
                then.status(200)
                    .header("Content-Type", "application/json")
                    .header("X-Total", users.len().to_string())
                    .header("X-Per-Page", "100")
                    .json_body_obj(&users[(i * 100)..((i + 1) * 100)].to_vec());
            });
        }
        server
    }
    #[derive(Authorization)]
    #[filter(FilterTest)]
    #[sort(SortTest)]
    #[range(RangeTest)]
    struct ConnectorApi;

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
    async fn request_1000_users() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi.connect(&server.base_url()).await?;
        let users: Vec<User> = api.get("/users/full")?.await?;
        assert_eq!(users.len(), 1000);
        Ok(())
    }

    #[tokio::test]
    async fn request_users_page_1() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi.connect(&server.base_url()).await?;
        let users: Vec<User> = api.get("/users")?.await?;
        assert_eq!(users.len(), 100);
        Ok(())
    }

    #[tokio::test]
    async fn request_users_full_pages() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi.connect(&server.base_url()).await?;
        let users: Vec<User> = api
            .get("/users")?
            .pagination(PaginationRule::OneShot)
            .await?;
        assert_eq!(users.len(), 1000);
        Ok(())
    }

    #[tokio::test]
    #[serial(pagination)]
    async fn get_method_default() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi.connect(&server.base_url()).await?;
        let request = api.get("/users")?;
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), PAGINATION_SIZE);
        Ok(())
    }

    #[tokio::test]
    #[serial(pagination)]
    async fn connector_none_pagination() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default());
        let request = api.get("/users")?;
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), PAGINATION_SIZE);
        Ok(())
    }

    #[tokio::test]
    #[serial(pagination)]
    async fn connector_fixed_pagination() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::Fixed(3));
        let request = api.get("/users")?;
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), PAGINATION_SIZE * 3);
        Ok(())
    }

    #[tokio::test]
    #[serial(pagination)]
    async fn connector_one_shot_pagination() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::OneShot);
        let request = api.get("/users")?;
        let response: Vec<User> = request.await?;
        assert!(response.len() == 1000);
        Ok(())
    }

    #[tokio::test]
    #[serial(pagination)]
    async fn request_none_pagination_override() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::OneShot);
        let request = api.get("/users")?.pagination(PaginationRule::default());
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), PAGINATION_SIZE);
        Ok(())
    }

    #[tokio::test]
    #[serial(pagination)]
    async fn request_fixed_pagination_override() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi.connect(&server.base_url()).await?;
        let request = api.get("/users")?.pagination(PaginationRule::Fixed(3));
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), PAGINATION_SIZE * 3);
        Ok(())
    }

    #[tokio::test]
    #[serial(pagination)]
    async fn request_one_shot_pagination_override() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi.connect(&server.base_url()).await?;
        let request = api.get("/users")?.pagination(PaginationRule::OneShot);
        let response: Vec<User> = request.await?;
        assert!(response.len() == 1000);
        Ok(())
    }

    #[tokio::test]
    #[serial(pagination)]
    async fn request_consistency() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi.connect(&server.base_url()).await?;
        let mut request = api
            .get::<Vec<User>>("/users")?
            .pagination(PaginationRule::Fixed(2));
        let first_response = request.send::<Vec<User>>().await?;
        assert_eq!(first_response.len(), PAGINATION_SIZE * 2);
        let second_response = request.send::<Vec<User>>().await?;
        assert_eq!(second_response.len(), PAGINATION_SIZE * 2);
        assert!(
            first_response
                .iter()
                .zip(second_response.iter())
                .all(|(a, b)| a.name != b.name)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial(pagination)]
    async fn request_consistency_after_reset() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi.connect(&server.base_url()).await?;
        let mut request = api
            .get::<Vec<User>>("/users")?
            .pagination(PaginationRule::Fixed(2));
        let first_response = request.send::<Vec<User>>().await?;
        assert_eq!(first_response.len(), PAGINATION_SIZE * 2);
        request.reset_pagination();
        let second_response = request.send::<Vec<User>>().await?;
        assert_eq!(second_response.len(), PAGINATION_SIZE * 2);
        assert!(
            first_response
                .iter()
                .zip(second_response.iter())
                .all(|(a, b)| a.name == b.name)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial(sort)]
    async fn connector_none_pagination_sort_name_asc() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .pattern_sort("property")
            .sort("name");
        let request = api.get("/users")?;
        let response: Vec<User> = request.await?;
        assert!(
            response
                .first()
                .unwrap()
                .name
                .lt(&response.last().unwrap().name)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial(sort)]
    async fn connector_none_pagination_sort_name_desc() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .pattern_sort("property")
            .sort("-name");
        let request = api.get("/users")?;
        let response: Vec<User> = request.await?;
        assert!(
            response
                .first()
                .unwrap()
                .name
                .gt(&response.last().unwrap().name)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial(sort)]
    async fn connector_none_pagination_sort_reset() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .pattern_sort("property")
            .sort("name");
        let request = api.get("/users")?.set_sort(SortTest::default());
        let response: Vec<User> = request.await?;
        assert!(
            response
                .first()
                .unwrap()
                .name
                .gt(&response.last().unwrap().name)
                || response
                    .first()
                    .unwrap()
                    .name
                    .lt(&response.last().unwrap().name)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial(sort)]
    async fn connector_none_pagination_request_sort_name_asc() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"]);
        let request = api.get("/users")?.pattern_sort("property").sort("name");
        let response: Vec<User> = request.await?;
        assert!(
            response
                .first()
                .unwrap()
                .name
                .lt(&response.last().unwrap().name)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial(sort)]
    async fn connector_none_pagination_request_sort_name_desc() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"]);
        let request = api.get("/users")?.pattern_sort("property").sort("-name");
        let response: Vec<User> = request.await?;
        assert!(
            response
                .first()
                .unwrap()
                .name
                .gt(&response.last().unwrap().name)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial(sort)]
    async fn connector_none_pagination_override_sort_name_asc() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .pattern_sort("property")
            .sort("-name");
        let request = api.get("/users")?.pattern_sort("property").sort("name");
        let response: Vec<User> = request.await?;
        assert!(
            response
                .first()
                .unwrap()
                .name
                .lt(&response.last().unwrap().name)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial(sort)]
    async fn connector_none_pagination_override_sort_name_desc() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .pattern_sort("property")
            .sort("name");
        let request = api.get("/users")?.pattern_sort("property").sort("-name");
        let response: Vec<User> = request.await?;
        assert!(
            response
                .first()
                .unwrap()
                .name
                .gt(&response.last().unwrap().name)
        );
        Ok(())
    }

    #[tokio::test]
    #[serial(filter)]
    async fn connector_none_pagination_filter() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .filter("name", vec!["jean"]);
        let request = api.get("/users")?;
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    #[serial(filter)]
    async fn connector_none_pagination_request_filter() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default());
        let request = api
            .get("/users")?
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .filter("name", vec!["jean"]);
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), 1);
        Ok(())
    }

    #[tokio::test]
    #[serial(filter)]
    async fn connector_none_pagination_filter_override() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .filter("name", vec!["jean"]);
        let request = api
            .get("/users")?
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .filter("name", vec!["jean,michel"]);
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), 2);
        Ok(())
    }

    #[tokio::test]
    #[serial(filter)]
    async fn connector_none_pagination_filter_reset() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_filter("filter[property]")
            .filter("primary_campus_id", vec!["31"])
            .filter("name", vec!["jean"]);
        let mut request = api
            .get::<Vec<User>>("/users")?
            .set_filter(FilterTest::default());
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), PAGINATION_SIZE);
        Ok(())
    }

    #[tokio::test]
    #[serial(range)]
    async fn connector_none_pagination_range() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_range("range[property]")
            .range("id", "45", "63");
        let request = api.get("/users")?;
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), 18);
        Ok(())
    }

    #[tokio::test]
    #[serial(range)]
    async fn connector_none_pagination_request_range() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default());
        let request = api
            .get("/users")?
            .pattern_range("range[property]")
            .range("id", "45", "63");
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), 18);
        Ok(())
    }

    #[tokio::test]
    #[serial(range)]
    async fn connector_none_pagination_range_override() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_range("range[property]")
            .range("id", "45", "63");
        let request = api.get("/users")?.range("id", "546", "736");
        let response: Vec<User> = request.await?;
        assert_eq!(response.len(), 190);
        Ok(())
    }

    #[tokio::test]
    #[serial(range)]
    async fn connector_none_pagination_range_reset() -> Result<()> {
        let server = mock_server();
        let api = ConnectorApi
            .connect(&server.base_url())
            .await?
            .pagination(PaginationRule::default())
            .pattern_range("range[property]")
            .range("id", "45", "63");
        let mut request = api
            .get::<Vec<User>>("/users")?
            .set_range(RangeTest::default());
        let response = request.send::<Vec<User>>().await?;
        assert_eq!(response.len(), PAGINATION_SIZE);
        Ok(())
    }
}
