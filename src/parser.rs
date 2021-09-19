use regex::Regex;
use scraper::{Html, Selector};

enum ContentLink {
    NotFound,
    Image(String),
    Video(String),
}

fn extract_src(document: &scraper::Html, selector: &str) -> Option<String> {
    let selector = Selector::parse(selector).unwrap();
    let mut element = document.select(&selector);
    if let Some(inner) = element.next() {
        let src = inner.value().attr("src").unwrap();
        return Some(src.to_owned());
    }
    None
}

fn extract_content_link(document: scraper::Html) -> ContentLink {
    if let Some(src) = extract_src(&document, "iframe") {
        return ContentLink::Video(src);
    }
    if let Some(src) = extract_src(&document, "a>img") {
        return ContentLink::Image(src);
    }
    ContentLink::NotFound
}

fn extract_video_id(link: &String) -> Option<String> {
    // shamelessly stolen from rustube crate that utilizes
    // experimental features and, therefore, cannot be used
    let watch_pattern: Regex = Regex::new(
        r"^(https?://)?(www\.)?youtube.\w\w\w?/watch\?v=(?P<id>[a-zA-Z0-9_-]{11})(&.*)?$",
    )
    .unwrap();
    let embed_pattern: Regex = Regex::new(
        r"^(https?://)?(www\.)?youtube.\w\w\w?/embed/(?P<id>[a-zA-Z0-9_-]{11})\\?(\?.*)?$",
    )
    .unwrap();
    let share_pattern: Regex =
        Regex::new(r"^(https?://)?youtu\.be/(?P<id>[a-zA-Z0-9_-]{11})$").unwrap();
    let id_pattern: Regex = Regex::new("^(?P<id>[a-zA-Z0-9_-]{11})$").unwrap();

    let id_patterns: [&Regex; 4] = [&watch_pattern, &embed_pattern, &share_pattern, &id_pattern];

    id_patterns.iter().find_map(|pattern| {
        pattern
            .captures(link.as_str())
            .map(|c| return c.name("id").unwrap().as_str().to_owned())
    })
}

pub fn get_full_url(bytes: actix_web::web::Bytes) -> String {
    let document = Html::parse_document(std::str::from_utf8(&bytes).unwrap());
    let not_found_url = "https://img.youtube.com/vi/7w8HlfC5Mb8/0.jpg".to_owned();

    return match extract_content_link(document) {
        ContentLink::NotFound => not_found_url,
        ContentLink::Image(link) => {
            format!("https://apod.nasa.gov/apod/{}", link)
        }
        ContentLink::Video(link) => {
            if let Some(id) = extract_video_id(&link) {
                return format!("https://img.youtube.com/vi/{}/0.jpg", id);
            }
            println!("failed to find id in {}", link);
            not_found_url
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::parser::get_full_url;
    use actix_web::web::Bytes;
    use std::fs;

    fn get_test_contents(file_name: &str) -> Bytes {
        Bytes::copy_from_slice(
            fs::read_to_string(format!("test-data/{}", file_name))
                .unwrap()
                .as_bytes(),
        )
    }

    #[test]
    fn test_get_full_url_image() {
        let url_expected =
            "https://apod.nasa.gov/apod/image/2109/saturn2004to2015_peach_960.jpg".to_owned();
        let url_actual = get_full_url(get_test_contents("test-image.htm"));

        assert_eq!(url_expected, url_actual);
    }

    #[test]
    fn test_get_full_url_video() {
        let url_expected = "https://img.youtube.com/vi/ImVl_TfTFEY/0.jpg".to_owned();
        let url_actual = get_full_url(get_test_contents("test-video.htm"));

        assert_eq!(url_expected, url_actual);
    }

    #[test]
    fn test_get_full_url_404() {
        let url_expected = "https://img.youtube.com/vi/7w8HlfC5Mb8/0.jpg".to_owned();
        let url_actual = get_full_url(get_test_contents("test-404.htm"));

        assert_eq!(url_expected, url_actual);
    }
}
