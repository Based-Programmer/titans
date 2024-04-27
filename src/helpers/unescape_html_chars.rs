pub fn unescape_html_chars(title: &str) -> Box<str> {
    title
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&apos;", "'")
        .replace("&#x27;", "'")
        .replace("&#x39;", "'")
        .replace("&#40;", "(")
        .replace("&#41;", ")")
        .replace('\u{200b}', "")
        .replace("\\u2013", "-")
        .into()
}
