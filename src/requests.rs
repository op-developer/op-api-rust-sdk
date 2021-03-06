use crate::options::Options;
use log::debug;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub struct Requests;

/// Single error from OP API.
#[derive(Deserialize, Debug, Clone)]
pub struct ApiError {
    pub id: String,
    pub level: String,
    pub r#type: String,
    pub message: String,
}

/// Container for API errors from OP API.
#[derive(Deserialize, Debug, Clone)]
pub struct ApiErrors {
    pub errors: Vec<ApiError>,
}

/// Implement functionality to display ApiErrors.
impl fmt::Display for ApiErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "API error occurred, reasons: {:?}", self.errors)
    }
}

/// Implement std::error::Error for ApiErrors.
impl Error for ApiErrors {}

/// Constructs string URL from base url and API url.
fn get_request_url(options: &Options, url: &str) -> String {
    format!("{base_url}{url}", base_url = options.base_url(), url = url)
}

/// Sets necessary headers for the request.
fn set_headers(options: &Options, builder: RequestBuilder) -> RequestBuilder {
    builder
        .header("x-api-key", options.api_key())
        .header(
            "Authorization",
            format!("{} {}", "Bearer", options.authorization()),
        )
        .header("Accept", "application/json")
}

/// Sets query parameters for the request.
fn set_query_params<T: Serialize>(query: Option<T>, builder: RequestBuilder) -> RequestBuilder {
    match query {
        Some(q) => builder.query(&q),
        None => builder,
    }
}

/// Checks for possible API errors from the response
async fn check_errors(response: Response) -> Result<Response, Box<dyn Error>> {
    match response.status() {
        StatusCode::OK => Ok(response),
        _ => {
            let errors: ApiErrors = response.json().await?;
            Err(Box::new(errors))
        }
    }
}

/// Internal requests functionality to ease client development.
///
/// These functions set up all necessary headers and run the request
/// asynchronously.
impl Requests {
    /// Performs GET request to API specified with url.
    pub async fn get<T: Serialize>(
        options: &Options,
        url: &str,
        query: Option<T>,
    ) -> Result<Response, Box<dyn Error>> {
        let request_url = get_request_url(options, url);
        let builder = Client::new().get(&request_url);
        let client = set_headers(options, set_query_params(query, builder));
        debug!("Sending request: {:?}", client);
        let response = client.send().await?;
        Ok(check_errors(response).await?)
    }
}
