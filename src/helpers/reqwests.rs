use isahc::{
    config::{RedirectPolicy::Follow, VersionNegotiation},
    error::Error,
    prelude::Configurable,
    HttpClient, ReadResponseExt,
};
use serde_json::Value;

pub fn get_isahc(link: &str, user_agent: &str, referrer: &str) -> Result<Box<str>, Error> {
    let client = client(user_agent, referrer)?;
    get_isahc_client(&client, link)
}

// pub fn get_reqwest(
//     link: &str,
//     user_agent: &str,
//     referrer: &str,
// ) -> Result<Box<str>, reqwest::Error> {
//     // Create a client with custom configuration
//     let client = Client::builder()
//         .http2_prior_knowledge() // Force HTTP/2
//         .build()?;

//     // Build and send the request
//     let response = client
//         .get(link)
//         .header(header::USER_AGENT, user_agent)
//         .header(header::REFERER, referrer)
//         .send()?;

//     // Get the response text
//     let text = response.text()?.into();

//     Ok(text)
// }

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
