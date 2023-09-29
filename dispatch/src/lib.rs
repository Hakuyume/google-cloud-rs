use bytes::{Bytes, BytesMut};
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use headers::{Authorization, HeaderMapExt};
use http::{Method, Request, Response, StatusCode, Uri};
use http_body::combinators::UnsyncBoxBody;
use http_body::{Body, Full};
use serde::{Deserialize, Serialize};
use std::future::{self, Future};
use std::pin;
use tower::buffer::Buffer;
use tower::util::BoxService;
use tower::{BoxError, Service, ServiceExt};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Service(BoxError),
    #[error("[{status:?}] {body:?}")]
    Http { status: StatusCode, body: Bytes },
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

type BoxBody = UnsyncBoxBody<Bytes, BoxError>;

#[derive(Clone)]
pub struct Client(
    Buffer<BoxService<Request<BoxBody>, Response<BoxBody>, BoxError>, Request<BoxBody>>,
);

impl Client {
    pub fn new<S, T, U>(service: S) -> Self
    where
        S: Service<Request<T>, Response = Response<U>> + Send + 'static,
        BoxError: From<S::Error>,
        S::Future: Send,
        T: From<BoxBody> + 'static,
        U: Body<Data = Bytes> + Send + 'static,
        BoxError: From<U::Error>,
    {
        Self(Buffer::new(
            BoxService::new(
                service
                    .map_request(|request: Request<BoxBody>| {
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
            hyper::Client::builder().build::<_, BoxBody>(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_webpki_roots()
                    .https_only()
                    .enable_http2()
                    .build(),
            ),
        )
    }

    pub fn send<T, U>(
        &self,
        method: Method,
        uri: Uri,
        body: T,
        bearer: Option<&str>,
    ) -> impl Future<Output = Result<U, Error>> + Send + 'static
    where
        T: IntoBody,
        U: FromBody + 'static,
    {
        let service = self.0.clone();
        let bearer = bearer.map(Authorization::bearer).transpose().unwrap();

        body.into_body()
            .and_then(move |body| {
                let mut request = Request::builder()
                    .method(method)
                    .uri(uri.clone())
                    .body(body)
                    .unwrap();
                if let Some(bearer) = bearer {
                    request.headers_mut().typed_insert(bearer);
                }

                service.oneshot(request).map_err(Error::Service)
            })
            .and_then(|response| {
                let (parts, body) = response.into_parts();
                if parts.status.is_success() {
                    U::from_body(body).boxed()
                } else {
                    Bytes::from_body(body)
                        .map(move |body| match body {
                            Ok(body) => Err(Error::Http {
                                status: parts.status,
                                body,
                            }),
                            Err(e) => Err(e),
                        })
                        .boxed()
                }
            })
    }
}

pub trait IntoBody {
    fn into_body(self) -> BoxFuture<'static, Result<BoxBody, Error>>;
}

#[async_trait::async_trait]
pub trait FromBody: Sized {
    async fn from_body(body: BoxBody) -> Result<Self, Error>;
}

impl IntoBody for Bytes {
    fn into_body(self) -> BoxFuture<'static, Result<BoxBody, Error>> {
        futures::future::ok(Full::new(self).map_err(BoxError::from).boxed_unsync()).boxed()
    }
}

#[async_trait::async_trait]
impl FromBody for Bytes {
    async fn from_body(body: BoxBody) -> Result<Self, Error> {
        let mut body = pin::pin!(body);
        let mut data = BytesMut::new();
        while let Some(chunk) = body.data().await.transpose().map_err(Error::Service)? {
            data.extend_from_slice(&chunk);
        }
        Ok(data.freeze())
    }
}

pub struct Json<T>(pub T);

impl<T> IntoBody for Json<T>
where
    T: Serialize + Send,
{
    fn into_body(self) -> BoxFuture<'static, Result<BoxBody, Error>> {
        future::ready(
            serde_json::to_vec(&self.0)
                .map(Bytes::from)
                .map_err(Error::from),
        )
        .and_then(IntoBody::into_body)
        .boxed()
    }
}

#[async_trait::async_trait]
impl<T> FromBody for Json<T>
where
    T: for<'de> Deserialize<'de> + Send,
{
    async fn from_body(body: BoxBody) -> Result<Self, Error> {
        Ok(Self(serde_json::from_slice(
            &Bytes::from_body(body).await?,
        )?))
    }
}
