use axum::extract::Path;
use axum::http::HeaderMap;
use axum::response::Html;
use axum::{routing::get, Router};
use bopo_wiki::transform_page;
use reqwest::Client;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(redirect))
        .route("/{*path}", get(mirror));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn redirect(headers: HeaderMap) -> Html<String> {
    mirror(headers, Path("/zh-tw/Wikipedia:首页".to_string())).await
}

fn is_mobile(user_agent: Option<&str>) -> bool {
    if let Some(ua) = user_agent {
        let ua_lower = ua.to_lowercase();
        ua_lower.contains("mobile")
            || ua_lower.contains("android")
            || ua_lower.contains("iphone")
            || ua_lower.contains("ipad")
            || ua_lower.contains("windows phone")
    } else {
        false
    }
}

async fn mirror(headers: HeaderMap, Path(path): Path<String>) -> Html<String> {
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok());
    let is_mobile = is_mobile(user_agent);

    let mut body = get_page(path, is_mobile)
        .await
        .expect("response from wikipedia")
        .text()
        .await
        .expect("converted response to text");

    body = transform_page(body);

    Html(body)
}

async fn get_page(path: String, is_mobile: bool) -> Result<reqwest::Response, reqwest::Error> {
    let client = Client::builder()
        .user_agent("BopoWiki/0.0 (github link)")
        .build()
        .unwrap();

    let base_url = if is_mobile {
        "https://zh.m.wikipedia.org"
    } else {
        "https://zh.wikipedia.org"
    };

    client.get(format!("{}/{}", base_url, path)).send().await
}
