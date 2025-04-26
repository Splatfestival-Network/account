use std::collections::HashMap;
use std::{env, fs};
use once_cell::sync::Lazy;
use rocket::get;
use serde::{Deserialize, Serialize};
use crate::xml::{Xml};

#[derive(Serialize, Deserialize)]
#[serde(rename(serialize = "timezone"))]
pub struct Timezone{
    area: String,
    language: String,
    name: String,
    utc_offset: String,
    order: String
}

#[derive(Serialize)]
#[serde(rename(serialize = "timezones"))]
pub struct Timezones<'a>{
    pub timezone: &'a [Timezone],
}

pub static ZONE_TO_TIMEZONES: Lazy<HashMap<String, HashMap<String, Vec<Timezone>>>> = Lazy::new(||{
    let path = {
        // if this crashes then something is wrong with the server setup so crashing here is fine imo
        let mut path = env::current_dir().unwrap();

        path.push("res");
        path.push("timezones.json");

        path
    };
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
});

pub static OFFSET_FROM_TIMEZONE: Lazy<HashMap<String, String>> = Lazy::new(||{
    let mut map = HashMap::new();

    for val in ZONE_TO_TIMEZONES.values(){
        for val in val.values(){
            for tz in val{
                map.insert(tz.area.clone(), tz.utc_offset.clone());
            }
        }
    }

    map
});


#[get("/v1/api/content/time_zones/<zone>/<lang>")]
pub fn get_timezone(zone: &str, lang: &str) -> Option<Xml<Timezones<'static>>>{
    let timezone = (&*ZONE_TO_TIMEZONES).get(zone)?.get(lang)?;
    let timezones = Timezones{ timezone };
    Some(Xml(timezones))
}

#[cfg(test)]
mod test{
    use crate::nnid::timezones::{Timezones, ZONE_TO_TIMEZONES};
    use crate::xml::serialize_with_version;

    #[test]
    fn test(){
        let timezone = (&*ZONE_TO_TIMEZONES).get("DE").unwrap().get("en").unwrap();
        let timezones = Timezones{ timezone };
        let ser = serialize_with_version(&timezones).unwrap();

        println!("{}", ser);

        assert_eq!(
            ser.as_ref(),
            "<?xml version=\"1.0\"?><timezones><timezone><area>Europe/Berlin</area><language>en</language><name>Amsterdam, Berlin, Rome</name><utc_offset>3600</utc_offset><order>0</order></timezone></timezones>"
        )
    }
}