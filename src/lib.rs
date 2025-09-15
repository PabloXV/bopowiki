use lol_html::{element, rewrite_str, text, RewriteStrSettings};

mod bopomofo;

pub fn transform_page(body: String) -> String {
    let element_content_handlers = vec![
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
        text!("body *", |text| {
            text.replace(
                &bopomofo::insert_bopomofo(&text.as_str()),
                lol_html::html_content::ContentType::Html,
            );
            Ok(())
        }),
    ];

    rewrite_str(
        &body,
        RewriteStrSettings {
            element_content_handlers,
            ..RewriteStrSettings::default()
        },
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use std::{fs::read_to_string, time::Instant};

    use super::*;

    #[test]
    fn it_should_replace_relative_urls() {
        let input = "<a href=\"/a.html\"></a>";
        let output = "<a href=\"https://zh.wikipedia.org/a.html\"></a>";
        assert_eq!(transform_page(input.to_string()), output)
    }

    #[test]
    fn it_should_replace_wiki_with_lang() {
        let input = "<a href=\"/wiki/title\"></a>";
        let output = "<a href=\"/zh-tw/title\"></a>";
        assert_eq!(transform_page(input.to_string()), output)
    }

    #[test]
    fn it_should_add_bopomofo_to_body_text() {
        let input = "<body><p>你好</p></body>";
        let output =
            "<body><p><ruby>你<rt>ㄋㄧ</rt><rt>ˇ</rt></ruby><ruby>好<rt>ㄏㄠ</rt><rt>ˇ</rt></ruby></p></body>";
        assert_eq!(transform_page(input.to_string()), output)
    }

    #[test]
    fn it_should_not_add_bopomofo_to_nonbody_text() {
        let input = "<p>你好</p>";
        assert_eq!(transform_page(input.to_string()), input.to_string())
    }

    #[test]
    fn how_long_it_takes() {
        // Naive FS: 2.6-2.9s
        let input = read_to_string("test_input.html").expect("it should read file");
        let start = Instant::now();
        transform_page(input.to_string());
        let duration = start.elapsed();
        println!("Time: {:?}", duration);

        // 2nd run takes 90ms
        let input = read_to_string("test_input.html").expect("it should read file");
        let start = Instant::now();
        transform_page(input.to_string());
        let duration = start.elapsed();
        println!("Time: {:?}", duration)
    }
}
