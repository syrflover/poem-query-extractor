use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use poem::{error::ResponseError, http::StatusCode, FromRequest, Request, RequestBody};
use serde::de::DeserializeOwned;

#[derive(Debug, thiserror::Error)]
pub enum QueryRejection {
    #[error("Deserialize querystring: {0}")]
    Deserialize(#[from] serde_qs::Error),
}

impl ResponseError for QueryRejection {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Query<T>(pub T);

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Query<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> TryFrom<&str> for Query<T>
where
    T: DeserializeOwned,
{
    type Error = QueryRejection;

    fn try_from(query: &str) -> Result<Self, Self::Error> {
        let value = serde_qs::from_str(query)?;
        Ok(Query(value))
    }
}

impl<'a, T> TryFrom<Cow<'a, str>> for Query<T>
where
    T: DeserializeOwned,
{
    type Error = QueryRejection;

    fn try_from(query: Cow<str>) -> Result<Self, Self::Error> {
        match query {
            Cow::Borrowed(query) => query.try_into(),
            Cow::Owned(query) => query.as_str().try_into(),
        }
    }
}

impl<T: DeserializeOwned> Query<T> {
    async fn internal_from_request(req: &Request) -> Result<Self, QueryRejection> {
        let query = req
            .uri()
            .query()
            .and_then(|query| urlencoding::decode(query).ok())
            .unwrap_or_default();

        query.try_into()
    }
}

impl<'a, T: DeserializeOwned> FromRequest<'a> for Query<T> {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        Self::internal_from_request(req).await.map_err(Into::into)
    }
}
