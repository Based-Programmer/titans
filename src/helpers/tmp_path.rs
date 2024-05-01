use std::{
    env::{consts::OS, var},
    error::Error,
};

pub fn tmp_path(twatter: bool) -> Result<String, Box<dyn Error>> {
    let file_name = if twatter { "twatter_guest_token" } else { "" };

    let tmp_path = match OS {
        "windows" => var("TEMP").unwrap_or(var("TMP")?) + "\\" + file_name,
        "android" => var("TMPDIR")? + "/" + file_name,
        _ => format!("/tmp/{}", file_name),
    };

    Ok(tmp_path)
}
