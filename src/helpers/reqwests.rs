use std::error::Error;

use isahc::{
    config::{RedirectPolicy::Follow, VersionNegotiation},
    prelude::Configurable,
    AsyncReadResponseExt, Request, RequestExt,
};

pub async fn get_isahc(
    link: &str,
    user_agent: &str,
    referrer: &str,
) -> Result<Box<str>, Box<dyn Error>> {
    Ok(Request::get(link)
        .header("user-agent", user_agent)
        .header("referer", referrer)
        .version_negotiation(VersionNegotiation::http2())
        .redirect_policy(Follow)
        .body(())?
        .send_async()
        .await?
        .text()
        .await?
        .into())
}
