pub fn unescape_html_chars(title: &str) -> Box<str> {
    title
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#x27;", "'")
        .replace("&#40;", "(")
        .replace("&#41;", ")")
        .into()
}
