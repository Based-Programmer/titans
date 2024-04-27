use isahc::{
    config::{RedirectPolicy::Follow, VersionNegotiation},
    error::Error,
    prelude::Configurable,
    HttpClient, ReadResponseExt, Request, RequestExt,
};
use serde_json::Value;

pub fn get_isahc(link: &str, user_agent: &str, referrer: &str) -> Result<Box<str>, Error> {
    Ok(Request::get(link)
        .header("user-agent", user_agent)
        .header("referer", referrer)
        .version_negotiation(VersionNegotiation::http2())
        .redirect_policy(Follow)
        .body(())?
        .send()?
        .text()?
        .into())
}

pub fn get_isahc_client(client: &HttpClient, link: &str) -> Result<Box<str>, Error> {
    Ok(client.get(link)?.text()?.into())
}

pub fn get_isahc_json(
    client: &HttpClient,
    link: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(client.get(link)?.json()?)
}

pub fn client(user_agent: &str, referrer: &str) -> Result<HttpClient, Error> {
    HttpClient::builder()
        .version_negotiation(VersionNegotiation::http2())
        .redirect_policy(Follow)
        .default_headers(&[("user-agent", user_agent), ("referer", referrer)])
        .build()
}
