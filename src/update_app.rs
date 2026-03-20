use std::{error::Error, io};

use reqwest::Client;

pub async fn fetch_post_data(
    client: &Client,
) -> Result<(String, u32), Box<dyn Error + Send + Sync>> {
    let html = client
        .get("https://www.instagram.com/p/DV4eJZUDeJ5/")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let description = extract_description(&html).ok_or_else(|| {
        io::Error::other("Instagram metadata description was not found in the response")
    })?;
    let likes = parse_like_count(&description)
        .ok_or_else(|| io::Error::other("Could not parse the Instagram like count"))?;

    Ok((description, likes))
}

pub fn extract_description(html: &str) -> Option<String> {
    extract_meta_content(html, "property=\"og:description\"")
        .or_else(|| extract_meta_content(html, "name=\"description\""))
}

fn extract_meta_content(html: &str, marker: &str) -> Option<String> {
    html.split("<meta")
        .filter_map(|fragment| fragment.split('>').next())
        .find(|tag| tag.contains(marker))
        .and_then(extract_content_attribute)
        .map(decode_html_entities)
}

fn extract_content_attribute(tag: &str) -> Option<String> {
    if let Some(start) = tag.find("content=\"") {
        let rest = &tag[start + "content=\"".len()..];
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }

    if let Some(start) = tag.find("content='") {
        let rest = &tag[start + "content='".len()..];
        let end = rest.find('\'')?;
        return Some(rest[..end].to_string());
    }

    None
}

fn decode_html_entities(text: String) -> String {
    text.replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn parse_like_count(description: &str) -> Option<u32> {
    let likes_index = description.find(" likes")?;
    let raw_count = description[..likes_index].trim();
    let digits: String = raw_count
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect();

    if digits.is_empty() {
        return None;
    }

    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{extract_description, parse_like_count};

    #[test]
    fn extracts_og_description_content() {
        let html = r#"<html><head><meta property="og:description" content="13 likes, 0 comments - sample text"></head></html>"#;

        let description = extract_description(html);

        assert_eq!(
            description.as_deref(),
            Some("13 likes, 0 comments - sample text")
        );
    }

    #[test]
    fn parses_like_count_from_description() {
        let likes = parse_like_count("13 likes, 0 comments - sample text");

        assert_eq!(likes, Some(13));
    }

    #[test]
    fn parses_like_count_with_thousands_separator() {
        let likes = parse_like_count("1,234 likes, 5 comments - sample text");

        assert_eq!(likes, Some(1234));
    }
}
