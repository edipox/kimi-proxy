use actix_web::{http::header, http::Method, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web::web::Bytes;
use futures::stream::StreamExt;
use reqwest::Client;
use std::net::SocketAddr;

const KIMI_BASE: &str = "https://api.kimi.com/coding/v1";
const USER_AGENT: &str = "claude-code/0.1.0";

/// Build headers for the upstream request
fn build_upstream_headers(req: &HttpRequest) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();

    for (key, value) in req.headers() {
        let key_str = key.as_str();
        // Skip host header - reqwest will set it automatically
        if key_str.eq_ignore_ascii_case("host") {
            continue;
        }
        // Skip connection header
        if key_str.eq_ignore_ascii_case("connection") {
            continue;
        }

        if let Ok(name) = reqwest::header::HeaderName::from_bytes(key.as_ref()) {
            if let Ok(val) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                headers.insert(name, val);
            }
        }
    }

    // Spoof the User-Agent
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(USER_AGENT),
    );

    headers
}

/// Handle all HTTP methods and proxy to Kimi API
async fn proxy_handler(
    client: web::Data<Client>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Bytes,
) -> Result<HttpResponse> {
    let query_string = req.query_string();
    let url = if query_string.is_empty() {
        format!("{}/{}", KIMI_BASE, path)
    } else {
        format!("{}/{}?{}", KIMI_BASE, path, query_string)
    };

    let method = match req.method().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    };

    let upstream_headers = build_upstream_headers(&req);

    let upstream_req = client
        .request(method, &url)
        .headers(upstream_headers)
        .body(body.to_vec())
        .send();

    let upstream_resp = match upstream_req.await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Request to upstream failed: {}", e);
            return Ok(HttpResponse::BadGateway()
                .body(format!("Failed to connect to upstream: {}", e)));
        }
    };

    let status = upstream_resp.status();
    let mut resp_builder = HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status.as_u16()).unwrap_or(actix_web::http::StatusCode::OK),
    );

    // Copy headers from upstream response
    for (key, value) in upstream_resp.headers() {
        let key_str = key.as_str();
        // Skip hop-by-hop headers
        if key_str.eq_ignore_ascii_case("transfer-encoding")
            || key_str.eq_ignore_ascii_case("content-encoding")
        {
            continue;
        }

        if let Ok(name) = header::HeaderName::from_bytes(key.as_ref()) {
            if let Ok(val) = header::HeaderValue::from_bytes(value.as_bytes()) {
                resp_builder.insert_header((name, val));
            }
        }
    }

    // Stream the response body
    let stream = upstream_resp.bytes_stream().map(|result| {
        result
            .map(|bytes| Bytes::from(bytes.to_vec()))
            .map_err(|e| {
                log::error!("Stream error: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            })
    });

    Ok(resp_builder.streaming(stream))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8787);

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("Invalid address");

    println!("Starting kimi-proxy on http://{}", addr);
    println!("Proxying requests to {}", KIMI_BASE);

    let client = web::Data::new(
        Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client"),
    );

    HttpServer::new(move || {
        App::new()
            .app_data(client.clone())
            .wrap(middleware::Logger::default())
            .route("/{path:.*}", web::get().to(proxy_handler))
            .route("/{path:.*}", web::post().to(proxy_handler))
            .route("/{path:.*}", web::put().to(proxy_handler))
            .route("/{path:.*}", web::delete().to(proxy_handler))
            .route("/{path:.*}", web::patch().to(proxy_handler))
            .route("/{path:.*}", web::head().to(proxy_handler))
            .route("/{path:.*}", web::method(Method::OPTIONS).to(proxy_handler))
    })
    .bind(addr)?
    .run()
    .await
}
