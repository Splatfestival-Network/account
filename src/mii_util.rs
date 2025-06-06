use std::env;
use once_cell::sync::Lazy;

pub static MII_PROVIDER_SERVER_URL: Lazy<Box<str>> = Lazy::new(||
    env::var("MII_PROVIDER_SERVER_URL").expect("MII_PROVIDER_SERVER_URL not specified").into_boxed_str()
);

fn get_mii_img_url_path(pid: i32, format: &str) -> String{
    format!("{}/main.{}", pid, format)
}

pub fn get_mii_img_url(pid: i32, format: &str) -> String{
    format!("{}/{}", &*MII_PROVIDER_SERVER_URL, get_mii_img_url_path(pid, format))
}