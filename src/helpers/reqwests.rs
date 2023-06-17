use isahc::{AsyncReadResponseExt, Request, RequestExt};
use reqwest::{
    header::{REFERER, USER_AGENT},
    Client,
};

pub async fn get_html_isahc(link: &str, user_agent: &str, referrer: &str) -> String {
    Request::get(link)
        .header("user-agent", user_agent)
        .header("referrer", referrer)
        .body(())
        .unwrap()
        .send_async()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}

pub async fn get_html_reqwest(link: &str, user_agent: &str, referrer: &str) -> String {
    Client::new()
        .get(link)
        .header(USER_AGENT, user_agent)
        .header(REFERER, referrer)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}
