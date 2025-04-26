use std::{env, io};
use rocket::fs::NamedFile;
use rocket::get;
use rocket::response::content::RawXml;
use tokio::fs::try_exists;
use crate::dsresponse::Ds;

#[get("/v1/api/content/agreements/Nintendo-Network-EULA/<lang>/@latest")]
pub async fn get_agreement(lang: &str) -> io::Result<Ds<RawXml<NamedFile>>>{
    let base_path = {
        // if this crashes then something is wrong with the server setup so crashing here is fine imo
        let mut path = env::current_dir().unwrap();

        path.push("res");
        path.push("agreement");

        path
    };

    let requested_file_path = {
        let mut path = base_path.clone();

        path.push(format!("{}.xml", lang));

        path
    };



    if try_exists(&requested_file_path).await.is_ok_and(|v| v == true){
        Ok(Ds(RawXml(NamedFile::open(&requested_file_path).await?)))
    } else {
        let fallback_path = {
            let mut path = base_path;

            path.push("DEFAULT.xml");

            path
        };

        Ok(Ds(RawXml(NamedFile::open(&fallback_path).await?)))
    }
}