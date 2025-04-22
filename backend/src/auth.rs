use crate::datastore::DataStore;
use actix_session::SessionExt;
use actix_web::{
    Error,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    web,
};
use actix_web::{
    HttpResponse,
    http::{self},
};
use futures::future::ok;
use futures_util::future::{FutureExt, LocalBoxFuture};
use log;
use std::rc::Rc;

pub async fn auth_chain(ip: String, key: String, ds: &DataStore) -> bool {
    // log::info!("AuthMiddleware: checking if auth is required");
    if !is_auth_required(ds).await {
        return true;
    }
    // log::info!("AuthMiddleware: auth is required, checking user");
    if allowed_session(ip, key, ds).await {
        log::info!("AuthMiddleware: auth passed, session is valid");
        return true;
    }
    // log::info!("AuthMiddleware: auth failed, redirecting to login");

    return false;
}

pub async fn is_auth_required(ds: &DataStore) -> bool {
    match ds.get("Authenticate".to_string()) {
        Some(a) => a == "true",
        None => false,
    }
}

pub async fn allowed_user(password: String, ds: &DataStore) -> bool {
    match ds.get("Password".to_string()) {
        Some(p) => p == password,
        None => false,
    }
}

pub async fn allowed_session(ip: String, key: String, ds: &DataStore) -> bool {
    let id = match ds.get(ip.clone()) {
        Some(id) => id,
        None => return false,
    };

    let k = match ds.get(id.clone()) {
        Some(key) => key,
        None => return false,
    };
    // log::info!(
    //     "ip : {}, id : {} key : {}, k : {}, key == k => {}",
    //     ip,
    //     id,
    //     key,
    //     k,
    //     key == k
    // );

    k == key
}

pub struct AuthMiddleware;

impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    // <-- only allow BoxBody, not a generic B
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            inner: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    inner: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    // now an async boxed future
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(inner);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // pull in the DataStore
        let ds = req
            .app_data::<web::Data<DataStore>>()
            .expect("Missing DataStore")
            .clone();

        // extract IP + session key
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        let maybe_key: Option<String> = req.get_session().get("session_key").unwrap_or(None);

        // clone the inner service pointer
        let inner = self.inner.clone();

        // Check if the URL is "/login" and allow the request to pass through
        if req.path() == "/login"
            || req.path().starts_with("/cdn")
            || req.path().starts_with("/favicon.ico")
        {
            log::info!(
                "AuthMiddleware: request to {}, forwarding without auth",
                req.path()
            );
            return inner.call(req).boxed_local();
        }

        async move {
            // if no session → redirect immediately
            let key = match maybe_key {
                Some(key) => key,
                None => "".to_string(),
            };
            // now **await** your async auth check
            if auth_chain(ip, key, &mut ds.get_ref()).await {
                // allowed → forward
                log::info!("AuthMiddleware: allowed session, forwarding request");
                inner.call(req).await
            } else {
                // denied → redirect
                let (req, _pl) = req.into_parts();
                let resp = HttpResponse::SeeOther()
                    .insert_header((http::header::LOCATION, "/login"))
                    .finish();
                Ok(ServiceResponse::new(req, resp))
            }
        }
        .boxed_local()
    }
}
