use chrono::{Datelike, NaiveDate};
use rocket::{post, State};
use serde::{Deserialize, Serialize};
use crate::account::account::{generate_password, User};
use crate::error::Errors;
use crate::nnid::pid_distribution::next_pid;
use crate::Pool;
use crate::xml::{Xml, YesNoVal};



#[derive(Deserialize)]
pub struct Email{
    address: Box<str>
}

#[derive(Deserialize, Serialize)]
pub struct Mii{
    name: Box<str>,
    primary: YesNoVal,
    data: Box<str>,
}

#[derive(Deserialize)]
#[serde(rename(serialize = "person"))]
pub struct AccountCreationData{
    birth_date: NaiveDate,
    user_id: Box<str>,
    password: Box<str>,
    country: Box<str>,
    language: Box<str>,
    tz_name: Box<str>,
    email: Email,
    mii: Mii,
    gender: Box<str>,
    marketing_flag: YesNoVal,
    off_device_flag: YesNoVal,
    region: i32
}

#[derive(Serialize)]
#[serde(rename(serialize = "person"))]
pub struct AccountCreationResponseData{
    pid: i32
}

#[post("/v1/api/people", data="<data>")]
pub async fn create_account(database: &State<Pool>, data: Xml<AccountCreationData>) -> Result<Xml<AccountCreationResponseData>, Option<Errors>>{
    let database = database.inner();

    // its fine to crash here if we cant get the next pid as that is in my opinion a dead state
    // anyways as noone can register anymore, EVER

    let pid = next_pid(database).await;

    let AccountCreationData {
        user_id,
        password,
        birth_date,
        tz_name,
        language,
        email: Email{
            address
        },
        mii: Mii{
            data,
            ..
        },
        marketing_flag,
        gender,
        region,
        country,
        off_device_flag,
        ..
    } = data.0;


    let password = generate_password(pid, &password).ok_or(None)?;

    sqlx::query!("
        INSERT INTO users (
                                 pid,
                                 username,
                                 password,
                                 birthdate,
                                 timezone,
                                 email,
                                 country,
                                 language,
                                 marketing_allowed,
                                 off_device_allowed,
                                 region,
                                 gender,
                                 mii_data
                                 ) VALUES (
                                            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13
                                 )
    ",
        pid,
        user_id.as_ref(),
        password,
        birth_date,
        tz_name.as_ref(),
        address.as_ref(),
        country.as_ref(),
        language.as_ref(),
        marketing_flag.0,
        off_device_flag.0,
        region,
        gender.as_ref(),
        data.as_ref()
    ).execute(database).await.unwrap();


    
    Ok(
        Xml(AccountCreationResponseData{
            pid
        })
    )
}

#[cfg(test)]
mod test{
    use chrono::NaiveDate;
    use crate::nnid::create_account::AccountCreationData;

    const TEST_XML: &str =
"<?xml version=\"1.0\" encoding=\"utf-8\"?>
<person>
  <birth_date>1991-02-03</birth_date>
  <user_id>testtest</user_id>
  <password>[PASSWORD]</password>
  <country>DE</country>
  <language>en</language>
  <tz_name>Europe/Berlin</tz_name>
  <agreement>
    <agreement_date>2025-02-24T19:42:45</agreement_date>
    <country>US</country>
    <location>https://account.spfn.cc/v1/api/content/agreements/Nintendo-Network-EULA/0300</location>
    <type>NINTENDO-NETWORK-EULA</type>
    <version>0300</version>
  </agreement>
  <email>
    <address>tvnebel@gmail.com</address>
    <owned>N</owned>
    <parent>N</parent>
    <primary>Y</primary>
    <validated>N</validated>
    <type>DEFAULT</type>
  </email>
  <mii>
    <name>y</name>
    <primary>Y</primary>
    <data>
      AwAAQDrPvmeBxJIQ3j+V8Ip4iCWDvgAAAEB5AAAAIABOAEEATQBFAAAAAAAAAEBAAAAhAQJoRBgm
      NEYUgRIXaA0AACkAUkhQAAAAAAAAAAAAAAAAAAAAAAAAAAAAANzO
    </data>
  </mii>
  <parental_consent>
    <scope>1</scope>
    <consent_date>2025-02-24T19:42:45</consent_date>
    <approval_id>0</approval_id>
  </parental_consent>
  <gender>M</gender>
  <region>1309343744</region>
  <marketing_flag>N</marketing_flag>
  <device_attributes>
    <device_attribute>
      <name>uuid_account</name>
      <value>55fdbad0-f2ab-11ef-b648-010144cdca06</value>
    </device_attribute>
    <device_attribute>
      <name>uuid_common</name>
      <value>898ed052-5e25-11ef-b648-010144cdca06</value>
    </device_attribute>
    <device_attribute>
      <name>persistent_id</name>
      <value>8000001d</value>
    </device_attribute>
    <device_attribute>
      <name>transferable_id_base</name>
      <value>0800000444cdca06</value>
    </device_attribute>
    <device_attribute>
      <name>transferable_id_base_common</name>
      <value>0640000444cdca06</value>
    </device_attribute>
  </device_attributes>
  <off_device_flag>N</off_device_flag>
</person>";
    #[test]
    fn test(){
        let data: AccountCreationData = quick_xml::de::from_str(TEST_XML).unwrap();

        assert_eq!(data.birth_date, NaiveDate::from_ymd_opt(1991,02,03).unwrap());
        assert_eq!(data.user_id.as_ref(), "testtest");
    }
}