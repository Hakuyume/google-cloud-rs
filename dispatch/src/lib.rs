use bytes::{Bytes, BytesMut};
use headers::{Authorization, HeaderMapExt};
use http::{Method, Request, Response, Uri};
use http_body::combinators::UnsyncBoxBody;
use http_body::{Body, Full};
use serde::{Deserialize, Serialize};
use std::pin;
use tower::buffer::Buffer;
use tower::util::BoxService;
use tower::{BoxError, Service, ServiceExt};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Service(BoxError),
    #[error("[{status:?}] {body:?}")]
    Http {
        status: http::StatusCode,
        body: Bytes,
    },
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct Client(
    Buffer<
        BoxService<
            Request<UnsyncBoxBody<Bytes, BoxError>>,
            Response<UnsyncBoxBody<Bytes, BoxError>>,
            BoxError,
        >,
        Request<UnsyncBoxBody<Bytes, BoxError>>,
    >,
);

impl Client {
    pub fn new<S, T, U>(service: S) -> Self
    where
        S: Service<Request<T>, Response = Response<U>> + Send + 'static,
        BoxError: From<S::Error>,
        S::Future: Send,
        T: From<UnsyncBoxBody<Bytes, BoxError>> + 'static,
        U: Body<Data = Bytes> + Send + 'static,
        BoxError: From<U::Error>,
    {
        Self(Buffer::new(
            BoxService::new(
                service
                    .map_request(|request: Request<UnsyncBoxBody<Bytes, BoxError>>| {
                        let (parts, body) = request.into_parts();
                        Request::from_parts(parts, T::from(body))
                    })
                    .map_response(|response| {
                        let (parts, body) = response.into_parts();
                        Response::from_parts(parts, body.map_err(BoxError::from).boxed_unsync())
                    })
                    .map_err(BoxError::from),
            ),
            4096,
        ))
    }

    pub fn hyper() -> Self {
        Self::new(
            hyper::Client::builder().build::<_, UnsyncBoxBody<Bytes, BoxError>>(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_webpki_roots()
                    .https_only()
                    .enable_http2()
                    .build(),
            ),
        )
    }

    pub async fn send<T, U>(
        &self,
        method: Method,
        uri: Uri,
        body: T,
        bearer: Option<&str>,
    ) -> Result<U, Error>
    where
        T: IntoBody,
        U: FromBody,
    {
        let body = body.into_body().await?;

        let mut request = Request::builder()
            .method(method)
            .uri(uri.clone())
            .body(body)
            .unwrap();
        if let Some(bearer) = bearer {
            request
                .headers_mut()
                .typed_insert(Authorization::bearer(bearer).unwrap());
        }

        let mut service = self.0.clone();
        futures::future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .map_err(Error::Service)?;
        let response = service.call(request).await.map_err(Error::Service)?;

        let (parts, body) = response.into_parts();
        if parts.status.is_success() {
            U::from_body(body).await
        } else {
            Err(Error::Http {
                status: parts.status,
                body: Bytes::from_body(body).await?,
            })
        }
    }
}

#[async_trait::async_trait]
pub trait IntoBody {
    async fn into_body(self) -> Result<UnsyncBoxBody<Bytes, BoxError>, Error>;
}

#[async_trait::async_trait]
pub trait FromBody: Sized {
    async fn from_body(body: UnsyncBoxBody<Bytes, BoxError>) -> Result<Self, Error>;
}

#[async_trait::async_trait]
impl IntoBody for Bytes {
    async fn into_body(self) -> Result<UnsyncBoxBody<Bytes, BoxError>, Error> {
        Ok(Full::new(self).map_err(BoxError::from).boxed_unsync())
    }
}

#[async_trait::async_trait]
impl FromBody for Bytes {
    async fn from_body(body: UnsyncBoxBody<Bytes, BoxError>) -> Result<Self, Error> {
        let mut body = pin::pin!(body);
        let mut data = BytesMut::new();
        while let Some(chunk) = body.data().await.transpose().map_err(Error::Service)? {
            data.extend_from_slice(&chunk);
        }
        Ok(data.freeze())
    }
}

pub struct Json<T>(pub T);

#[async_trait::async_trait]
impl<T> IntoBody for Json<T>
where
    T: Serialize + Send,
{
    async fn into_body(self) -> Result<UnsyncBoxBody<Bytes, BoxError>, Error> {
        let body = serde_json::to_vec(&self.0)?;
        Bytes::from(body).into_body().await
    }
}

#[async_trait::async_trait]
impl<T> FromBody for Json<T>
where
    T: for<'de> Deserialize<'de> + Send,
{
    async fn from_body(body: UnsyncBoxBody<Bytes, BoxError>) -> Result<Self, Error> {
        Ok(Self(serde_json::from_slice(
            &Bytes::from_body(body).await?,
        )?))
    }
}
