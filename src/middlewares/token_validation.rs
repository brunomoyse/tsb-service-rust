use std::{future::{ready, Future, Ready}, pin::Pin};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, 
    Error, HttpResponse,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use crate::models::user::{Claims, get_secret_key};

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication 
where 
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S> 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Some(auth_data) = req.headers().get("Authorization") {
            match auth_data.to_str() {
                Ok(auth_value) => {
                    // Extract the token from the "Bearer <token>" format
                    let token = auth_value.trim_start_matches("Bearer ").trim();

                    // Perform JWT validation
                    let validation = Validation::new(Algorithm::HS256);
                    let secret = get_secret_key();
                    match decode::<Claims>(&token, &DecodingKey::from_secret(secret.as_ref()), &validation) {
                        Ok(_) => {
                            // If the authorization is valid, continue to the next service:
                            let fut = self.service.call(req);
                            return Box::pin(async move {
                                let res = fut.await?;
                                Ok(res)
                            });
                        },
                        Err(_) => {
                            // Handle invalid token
                            return Box::pin(async move {
                                let response = HttpResponse::Unauthorized()
                                    .json(serde_json::json!({"error": "Invalid token"}));
                                Err(actix_web::error::InternalError::from_response("Invalid token", response).into())
                            });
                        }
                    }
                },
                Err(e) => {
                    // Handle header value parsing error
                    return Box::pin(async move {
                        let response = HttpResponse::Unauthorized()
                            .json(serde_json::json!({"error": format!("Invalid header format: {}", e)}));
                        Err(actix_web::error::InternalError::from_response("Invalid header format", response).into())
                    });
                },
            }
        } else {
            // No Authorization header found
            return Box::pin(async {
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({"error": "Authorization header not found"}));
                Err(actix_web::error::InternalError::from_response("Authorization header not found", response).into())
            });
        }
    }
}