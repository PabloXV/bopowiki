use jieba_rs::Jieba;
use once_cell::sync::Lazy;
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
            let word = parts[0].to_string();
            if dict.contains_key(&word) {
                continue;
            }
            dict.insert(word, parts[2].to_string());
        }
    }
    dict
}

static BOPOMOFO_DICT: Lazy<HashMap<String, String>> = Lazy::new(load_dictionary);
static JIEBA: Lazy<Jieba> = Lazy::new(|| {
    let mut jieba = Jieba::new();
    for (word, _bopomofo) in BOPOMOFO_DICT.iter() {
        if word.chars().count() > 1
            && !word.chars().all(|c| {
                ('\u{3105}'..='\u{312F}').contains(&c)
                    || c == 'ˊ'
                    || c == 'ˋ'
                    || c == 'ˇ'
                    || c == '˙'
            })
        {
            jieba.add_word(word, None, None);
        }
    }
    jieba
});

fn transform_segment(segment: &str) -> String {
    if segment
        .chars()
        .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
    {
        if let Some(bopo) = BOPOMOFO_DICT.get(segment) {
            let bopo_parts: Vec<&str> = bopo.split(' ').collect();
            let chars: Vec<char> = segment.chars().collect();

            if bopo_parts.len() == chars.len() {
                return synthesize_char_and_bopo(chars, bopo_parts);
            }
        }
    }

    segment.to_string()
}

fn synthesize_char_and_bopo(chars: Vec<char>, bopo_parts: Vec<&str>) -> String {
    let tone_markers = ['ˊ', 'ˇ', 'ˋ', '˙'];
    let mut result = String::new();
    for (ch, bp) in chars.iter().zip(bopo_parts.iter()) {
        if let Some(last_char) = bp.chars().last() {
            if tone_markers.contains(&last_char) {
                let bopomofo = &bp[..bp.len() - last_char.len_utf8()];
                let last_char = format!("{}", last_char);
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

pub fn insert_bopomofo(text: &str) -> String {
    if text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)) {
        let mut new_content = String::new();
        let segments = JIEBA.cut(&text, false);
        for seg in segments {
            new_content.push_str(&transform_segment(seg));
        }
        return new_content;
    };

    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_replace_relative_urls() {
        let input = "你好";
        let output = "<ruby>你<rt>ㄋㄧ</rt><rt>ˇ</rt></ruby><ruby>好<rt>ㄏㄠ</rt><rt>ˇ</rt></ruby>";
        assert_eq!(insert_bopomofo(input), output)
    }
}
