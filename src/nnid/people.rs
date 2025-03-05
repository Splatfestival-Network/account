use std::env;
use std::io::Cursor;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use gxhash::{gxhash32, gxhash64};
use minio::s3::args::PutObjectArgs;
use minio::s3::builders::{ObjectContent, SegmentedBytes};
use minio::s3::client::ClientBuilder;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::utils::crc32;
use once_cell::sync::Lazy;
use rocket::{get, post, put, State};
use rocket::serde::{Deserialize, Serialize};
use crate::account::account::{generate_password, Auth, User};
use crate::dsresponse::Ds;
use crate::error::Errors;
use crate::nnid::pid_distribution::next_pid;
use crate::nnid::timezones::{OFFSET_FROM_TIMEZONE, ZONE_TO_TIMEZONES};
use crate::Pool;
use crate::xml::{Xml, YesNoVal};

static S3_URL_STRING: Lazy<Box<str>> = Lazy::new(||
    env::var("S3_URL").expect("S3_URL not specified").into_boxed_str()
);


static S3_URL: Lazy<BaseUrl> = Lazy::new(||
    S3_URL_STRING.parse().unwrap()
);

static S3_USER: Lazy<Box<str>> = Lazy::new(||
    env::var("S3_USER").expect("S3_USER not specified").into_boxed_str()
);

static S3_PASSWD: Lazy<Box<str>> = Lazy::new(||
    env::var("S3_PASSWD").expect("S3_PASSWD not specified").into_boxed_str()
);

fn get_mii_img_url_path(pid: i32, format: &str) -> String{
    format!("mii/{}/main.{}", pid, format)
}

fn get_mii_img_url(pid: i32, format: &str) -> String{
    format!("{}/pn-boss/{}", &*S3_URL_STRING, get_mii_img_url_path(pid, format))
}

async fn generate_s3_images(pid: i32, mii_data: &str){


    let auth = StaticProvider::new(&S3_USER, &S3_PASSWD, None);

    let Ok(client) = ClientBuilder::new(S3_URL.clone())
        .provider(Some(Box::new(auth)))
        .build() else {
        return;
    };

    let Some(image) = mii::get_image_png(mii_data).await else {
        return;
    };
    let object_name = get_mii_img_url_path(pid, "png");
    let object_content = ObjectContent::from(image);
    client.put_object_content("pn-cdn", &object_name, object_content).send().await.ok();

    let Some(image) = mii::get_image_tga(mii_data).await else {
        return;
    };
    let object_name =  get_mii_img_url_path(pid, "tga");
    let object_content = ObjectContent::from(image);
    client.put_object_content("pn-cdn", &object_name, object_content).send().await.ok();
}

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

    generate_s3_images(pid, &data).await;

    Ok(
        Xml(AccountCreationResponseData{
            pid
        })
    )
}

#[derive(Serialize)]
struct DevAttr{

}

#[derive(Serialize)]
struct EmailInfoOwnProfileData{
    address: String,
    id: u32,
    parent: YesNoVal,
    primary: YesNoVal,
    reachable: YesNoVal,
    #[serde(rename = "type")]
    email_type: String,
    updated_by: String,
    validated: YesNoVal,
    validated_date: Option<NaiveDateTime>
}

#[derive(Serialize)]
struct MiiImage{
    cached_url: String,
    id: u32,
    url: String,
    #[serde(rename = "type")]
    image_type: String
}


#[derive(Serialize)]
struct MiiImages{
    mii_image: MiiImage
}

#[derive(Serialize)]
struct MiiDataOwnProfileData{
    status: String,
    data: String,
    id: u32,
    mii_hash: String,
    mii_images: MiiImages,
    name: String,
    primary: YesNoVal
}



#[derive(Serialize)]
#[serde(rename(serialize = "person"))]
struct GetOwnProfileData{
    active_flag: YesNoVal,
    birth_date: NaiveDate,
    country: String,
    create_date: NaiveDateTime,
    gender: String,
    language: String,
    updated: NaiveDateTime,
    marketing_flag: YesNoVal,
    off_device_flag: YesNoVal,
    pid: i32,
    email: EmailInfoOwnProfileData,
    mii: MiiDataOwnProfileData,
    region: i32,
    tz_name: String,
    user_id: String,
    utc_offset: String
}

#[get("/v1/api/people/@me/profile")]
pub fn get_own_profile(user: Auth<false>) -> Ds<Xml<GetOwnProfileData>>{
    let User{
        username,
        pid,
        account_level,
        mii_data,
        gender,
        birthdate,
        country,
        creation_date,
        timezone,
        language,
        email,
        email_verified_since,
        updated,
        marketing_allowed,
        off_device_allowed,
        region,
        ..
    } = user.into();

    let timezone_offset = (&*OFFSET_FROM_TIMEZONE).get(&timezone).unwrap().to_owned();

    // whenever we need an id or hash we just take the gxhash of the data cause i dont want data clutter
    // this both avoids the data we have to store as well as data clutter whilest keeping the ids
    // always the same

    let mii_data = mii_data
        .replace("\n", "")
        .replace("\t", "")
        .replace("\r", "")
        .replace(" ", "");

    Ds(Xml(
        GetOwnProfileData{
            active_flag: YesNoVal(true),
            pid,
            user_id: username,
            gender,
            birth_date: birthdate,
            country,
            create_date: creation_date,
            tz_name: timezone,
            language,
            updated,
            marketing_flag: YesNoVal(marketing_allowed),
            email: EmailInfoOwnProfileData{
                id: gxhash32(email.as_bytes(), 0),
                address: email,
                validated: YesNoVal(email_verified_since.is_some()),
                validated_date: email_verified_since,
                email_type: "DEFAULT".to_string(),
                updated_by: "USER".to_string(),
                reachable: YesNoVal(true),
                primary: YesNoVal(true),
                parent: YesNoVal(false),
            },
            mii: MiiDataOwnProfileData{
                id: gxhash32(mii_data.as_bytes(), 0),
                // the bitmask here is to avoid causing an too big number as we dont know if the
                // wii u uses a 64 bit int here
                mii_hash: hex::encode(bytemuck::bytes_of(
                    &(gxhash64(mii_data.as_bytes(), 1) & !(0x1000000000000000))
                )),
                name: mii::MiiData::read(&mii_data)
                        .map(|v| v.name).unwrap_or("INVALID".to_string()),
                primary: YesNoVal(true),
                data: mii_data,
                status: "COMPLETED".to_string(),
                mii_images: MiiImages{
                    mii_image: {
                        let image_url = get_mii_img_url(pid, "tga");
                        let url_hash = gxhash32(image_url.as_bytes(), 0);
                        MiiImage {
                            image_type: "standard".to_string(),
                            id: url_hash,
                            url: image_url.clone(),
                            cached_url: image_url,
                        }
                    }
                }
            },

            off_device_flag: YesNoVal(off_device_allowed),
            region,
            utc_offset: timezone_offset,
        }
    ))
}

#[put("/v1/api/people/@me/miis/@primary")]
pub fn change_mii() {
    // stubbed(tecnically requires auth but this doesnt do anything so theres no harm in not doing auth here rn)
}

