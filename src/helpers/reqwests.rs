use isahc::{
    config::{RedirectPolicy::Follow, VersionNegotiation},
    error::Error,
    prelude::Configurable,
    AsyncReadResponseExt, HttpClient, Request, RequestExt,
};

pub async fn get_isahc(link: &str, user_agent: &str, referrer: &str) -> Result<Box<str>, Error> {
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

pub async fn get_isahc_client(client: &HttpClient, link: &str) -> Result<Box<str>, Error> {
    Ok(client.get_async(link).await?.text().await?.into())
}

pub fn client(user_agent: &str, referrer: &str) -> Result<HttpClient, Error> {
    HttpClient::builder()
        .version_negotiation(VersionNegotiation::http2())
        .redirect_policy(Follow)
        .default_headers(&[("user-agent", user_agent), ("referer", referrer)])
        .build()
}
