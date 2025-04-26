use rocket::get;
use serde::Serialize;
use crate::xml::Xml;

#[derive(Serialize)]
#[serde(rename(serialize = "device"))]
pub struct Device;

#[get("/v1/api/devices/@current/status")]
pub fn current_device_status() -> Xml<Device>{
    Xml(Device)
}

#[cfg(test)]
mod tests {
    use crate::nnid::devices::Device;

    #[test]
    fn test_device_data(){
        let text = crate::xml::serialize_with_version(&Device).unwrap();



        println!("{}", text);

    }
}