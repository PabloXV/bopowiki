use axum::extract::Path;
use axum::response::Html;
use axum::{Router, routing::get};
use jieba_rs::Jieba;
use lol_html::{RewriteStrSettings, element, rewrite_str, text};
use once_cell::sync::Lazy;
use reqwest::Client;
use std::collections::HashMap;

fn load_dictionary() -> HashMap<String, String> {
    let mut dict = HashMap::new();
    let csv = std::fs::read_to_string("data/tsi.csv").expect("failed to load dict csv");
    for line in csv.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            dict.insert(parts[0].to_string(), parts[2].to_string());
        }
    }
    dict
}

static BOPOMOFO_DICT: Lazy<HashMap<String, String>> = Lazy::new(load_dictionary);
static JIEBA: Lazy<Jieba> = Lazy::new(|| Jieba::new());

#[tokio::main]
async fn main() {
    let app = Router::new().route("/{*path}", get(mirror));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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

fn transform_page(body: String) -> String {
    let mut element_content_handlers = vec![
        element!("*[href]", |el| {
            if let Some(href) = el.get_attribute("href") {
                let new_href = match href.as_str() {
                    url if url.starts_with("http://") || url.starts_with("https://") => href,
                    url if url.starts_with("//") => format!("https:{}", url),
                    url if url.starts_with("/wiki/") => url.replace("/wiki/", "/zh-tw/"),
                    url if url.starts_with("/") => format!("https://zh.wikipedia.org{}", url),
                    _ => href,
                };

                el.set_attribute("href", &new_href).unwrap();
            }

            Ok(())
        }),
        element!("*[src]", |el| {
            if let Some(src) = el.get_attribute("src") {
                let new_src = match src.as_str() {
                    url if url.starts_with("http://") || url.starts_with("https://") => src,
                    url if url.starts_with("//") => format!("https:{}", url),
                    url if url.starts_with("/") => format!("https://zh.wikipedia.org{}", url),
                    _ => src,
                };
                el.set_attribute("src", &new_src)?;
            }
            Ok(())
        }),
        element!("*[srcset]", |el| {
            if let Some(srcset) = el.get_attribute("srcset") {
                let new_srcset = srcset
                    .split(',')
                    .map(|part| {
                        let trimmed = part.trim();
                        if let Some(url) = trimmed.split_whitespace().next() {
                            let new_url = match url {
                                u if u.starts_with("//") => format!("https:{}", u),
                                u if u.starts_with("/") => format!("https://zh.wikipedia.org{}", u),
                                u => u.to_string(),
                            };
                            trimmed.replacen(url, &new_url, 1)
                        } else {
                            trimmed.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                el.set_attribute("srcset", &new_srcset)?;
            }
            Ok(())
        }),
        element!("head", |el| {
            el.append(
                r#"<style>
                    ruby {
                        display: inline-flex;
                        align-items: center;
                    }
                    rt {
                        writing-mode: vertical-rl;
                        text-orientation: upright;
                        margin-left: -0.1em;
                        margin-right: 0em;
                    }
                    rt:nth-of-type(2) {
                        margin-left: -0.5em;
                        margin-right: -0.2em;
                    }
                </style>"#,
                lol_html::html_content::ContentType::Html,
            );
            Ok(())
        }),
    ];

    element_content_handlers.push(text!("*", |text| {
        let content = text.as_str();
        if content
            .chars()
            .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
        {
            let mut new_content = String::new();
            let segments = JIEBA.cut(&content, false);
            for seg in segments {
                new_content.push_str(&insert_bopomofo(seg));
            }
            text.replace(&new_content, lol_html::html_content::ContentType::Html);
        };
        Ok(())
    }));

    rewrite_str(
        &body,
        RewriteStrSettings {
            element_content_handlers,
            ..RewriteStrSettings::default()
        },
    )
    .unwrap()
}

fn insert_bopomofo(text: &str) -> String {
    let tone_markers = ['ˊ', 'ˇ', 'ˋ', '˙'];
    if text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)) {
        if let Some(bopo) = BOPOMOFO_DICT.get(text) {
            let bopo_parts: Vec<&str> = bopo.split(' ').collect();
            let chars: Vec<char> = text.chars().collect();

            if bopo_parts.len() == chars.len() {
                let mut result = String::new();
                for (ch, bp) in chars.iter().zip(bopo_parts.iter()) {
                    if let Some(last_char) = bp.chars().last() {
                        if tone_markers.contains(&last_char) {
                            let bopomofo = &bp[..bp.len() - last_char.len_utf8()];
                            result.push_str(&format!(
                                "<ruby>{}<rt>{}</rt><rt>{}</rt></ruby>",
                                &ch, bopomofo, last_char
                            ));
                        } else {
                            result.push_str(&format!("<ruby>{}<rt>{}</rt></ruby>", &ch, &bp));
                        }
                    }
                }

                return result;
            }
        }
    }

    text.to_string()
}
