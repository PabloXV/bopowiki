use axum::extract::Path;
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

async fn redirect() -> Html<String> {
    mirror(Path("/zh-tw/Wikipedia:首页".to_string())).await
}

async fn mirror(Path(path): Path<String>) -> Html<String> {
    let mut body = get_page(path)
        .await
        .expect("response from wikipedia")
        .text()
        .await
        .expect("converted response to text");

    body = transform_page(body);

    Html(body)
}

async fn get_page(path: String) -> Result<reqwest::Response, reqwest::Error> {
    let client = Client::builder()
        .user_agent("BopoWiki/0.0 (github link)")
        .build()
        .unwrap();

    client
        .get(format!("https://zh.wikipedia.org/{}", path))
        .send()
        .await
}
