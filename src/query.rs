use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::error::ApiError;

/// Query parameters for the request
#[derive(Clone, Default)]
pub struct Query(Vec<String>);

impl Query {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new query from a key-value pair
    pub fn from(key: impl ToString, value: impl ToString) -> Self {
        Self::default().add(key, value)
    }

    /// Add a key-value pair to the query
    pub fn add(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.0
            .push(format!("{}={}", key.to_string(), value.to_string()));
        self
    }

    /// Join two queries into one
    pub fn join(mut self, query: Query) -> Self {
        self.0.extend(query.0);
        self
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.0.is_empty() {
            write!(f, "?{}", self.0.join("&"))?;
        }
        Ok(())
    }
}

impl Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?{}", self.0.join("&"))
    }
}

impl FromStr for Query {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Query::new());
        }

        let split_params: Vec<String> = s.split('&').map(|s| s.to_owned()).collect();
        Ok(Query(split_params))
    }
}

impl From<&str> for Query {
    fn from(s: &str) -> Self {
        Query::from_str(s).unwrap()
    }
}

impl From<String> for Query {
    fn from(s: String) -> Self {
        Query::from_str(&s).unwrap()
    }
}

impl<T: ToString, U: ToString> From<(T, U)> for Query {
    fn from((key, value): (T, U)) -> Self {
        Query::from(key, value)
    }
}

impl From<&[&str]> for Query {
    fn from(params: &[&str]) -> Self {
        let mut query = Query::new();
        for param in params {
            query = query.join(Query::from_str(param).unwrap());
        }
        query
    }
}

impl From<&[String]> for Query {
    fn from(params: &[String]) -> Self {
        let mut query = Query::new();
        for param in params {
            query = query.join(Query::from_str(param).unwrap());
        }
        query
    }
}

impl<T: ToString, U: ToString> From<&[(T, U)]> for Query {
    fn from(params: &[(T, U)]) -> Self {
        let mut query = Query::new();
        for (key, value) in params {
            query = query.add(key.to_string(), value.to_string());
        }
        query
    }
}
