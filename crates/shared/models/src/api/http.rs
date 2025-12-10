use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

use super::errors::InternalApiError;
use super::{
    AcceptedResponse, ApiError, CollectionResponse, CreatedResponse, ErrorCode, SingleResponse,
    SuccessResponse,
};

impl ErrorCode {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::ValidationError => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ConflictError => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<T: Serialize + 'static> IntoResponse for SingleResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl<T: Serialize + 'static> IntoResponse for CollectionResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl IntoResponse for SuccessResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl<T: Serialize + 'static> IntoResponse for CreatedResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::CREATED,
            [("Location", self.location.clone())],
            Json(self),
        )
            .into_response()
    }
}

impl IntoResponse for AcceptedResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::ACCEPTED, Json(self)).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.code.status_code();
        (status, Json(self)).into_response()
    }
}

impl IntoResponse for InternalApiError {
    fn into_response(self) -> axum::response::Response {
        let error: ApiError = self.into();
        error.into_response()
    }
}
