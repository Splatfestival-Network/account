use std::str::FromStr;
use bytemuck::{try_from_bytes, Pod, Zeroable};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use reqwest::Url;

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C, packed)]
struct FFLStoreData{
    mii_data: FFLiMiiDataOfficial
}

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C, packed)]
struct FFLiMiiDataOfficial{
    core_data: FFLiMiiDataCore
}

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C, packed)]
struct FFLiMiiDataCore{
    stuff: u32,
    author_id: u64,
    create_id: [u8; 10],
    unk_1: u16,
    unk_2: u16,
    pub name: [u16; 10],


}

pub struct MiiData{
    pub name: String
}

impl MiiData{
    pub fn read(data: &str) -> Option<Self>{
        let data = BASE64_STANDARD.decode(data).ok()?;

        let data: &FFLStoreData = try_from_bytes(data.get(0..size_of::<FFLStoreData>())?).ok()?;

        let name = data.mii_data.core_data.name;
        let idx = name.iter().position(|v| *v == 0x0).unwrap_or(10);

        let name = &name[0..idx];

        let name = String::from_utf16(&name).ok()?;

        Some(Self{
            name
        })
    }
}

pub async fn get_image_png(data: &str) -> Option<Vec<u8>>{
    let mut url = Url::from_str("https://mii-unsecure.ariankordi.net/miis/image.png\
    ").unwrap();

    url.set_query(Some(&format!("data={}", data)));

    reqwest::get(url).await.ok().map(|v| v.bytes())?.await.ok().map(|b| b.to_vec())
}

pub async fn get_image_tga(data: &str) -> Option<Vec<u8>>{
    let mut url = Url::from_str("https://mii-unsecure.ariankordi.net/miis/image.tga\
    ").unwrap();

    url.set_query(Some(&format!("data={}", data)));

    reqwest::get(url).await.ok().map(|v| v.bytes())?.await.ok().map(|b| b.to_vec())
}

#[cfg(test)]
mod test{
    use std::fs;
use crate::get_image_png;

#[tokio::test]
    async fn test_image_get(){
        let image = get_image_png("AAEAQDrPvmeBxJIQ3cL/BYp4iCWDvgAA8FVEAEoATQByAFQAVgAAAGgAZQByAAB/BAApBBpK4xiXEqQMAhgXbAoACClQQkhQTQBFAAAALQBTAHcAaQB0AGMAaAAAAMqP").await.unwrap();

        fs::write("heh.png", image).unwrap();


    }
}