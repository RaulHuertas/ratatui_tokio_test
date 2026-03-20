use std::{error::Error};

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

    //let description = extract_description(&html).ok_or_else(|| {
    //    io::Error::other("Instagram metadata description was not found in the response")
    //})?;
    let likes = 32;
    //let likes = parse_like_count(&description)
    //    .ok_or_else(|| io::Error::other("Could not parse the Instagram like count"))?;

    Ok(("Hi!".to_string(), likes))
}
