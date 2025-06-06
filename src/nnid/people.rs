use std::env;
use std::io::Write;
use chrono::{NaiveDate, NaiveDateTime};
use gxhash::{gxhash32, gxhash64};
use once_cell::sync::Lazy;
use rocket::{get, post, put, State};
use rocket::serde::{Deserialize, Serialize};
use crate::account::account::{generate_password, Auth, User};
use crate::dsresponse::Ds;
use crate::error::{Error, Errors};
use crate::nnid::pid_distribution::next_pid;
use crate::nnid::timezones::{OFFSET_FROM_TIMEZONE};
use crate::Pool;
use crate::xml::{Xml, YesNoVal};
use crate::email::send_verification_email;
use rand::Rng;
use mii::{get_image_png, get_image_tga};
use std::sync::Arc;
use crate::mii_util::get_mii_img_url;

const DATABASE_ERROR: Errors = Errors{
    error: &[
        Error{
            code: "9999",
            message: "Internal server error"
        }
    ]
};


/*
pub async fn generate_s3_images(pid: i32, mii_data: &str) {
    let auth = StaticProvider::new(&S3_USER, &S3_PASSWD, None);

    let Ok(client) = ClientBuilder::new(S3_URL.clone())
        .provider(Some(Box::new(auth)))
        .build()
    else {
        println!("Failed to build S3 client for PID {}", pid);
        return;
    };

    let Some(image) = mii::get_image_png(mii_data).await else {
        println!("Failed to fetch PNG image for PID {}", pid);
        return;
    };

    let object_name = get_mii_img_url_path(pid, "png");
    let object_content = ObjectContent::from(image);

    if let Err(e) = client.put_object_content(&**S3_BUCKET, &object_name, object_content).send().await {
        println!("Failed to upload PNG for PID {}: {:?}", pid, e);
    } else {
        println!("Successfully uploaded PNG for PID {}", pid);
    }

    let Some(image) = mii::get_image_tga(mii_data).await else {
        println!("Failed to fetch TGA image for PID {}", pid);
        return;
    };

    let object_name = get_mii_img_url_path(pid, "tga");
    let object_content = ObjectContent::from(image);

    if let Err(e) = client.put_object_content(&**S3_BUCKET, &object_name, object_content).send().await {
        println!("Failed to upload TGA for PID {}: {:?}", pid, e);
    } else {
        println!("Successfully uploaded TGA for PID {}", pid);
    }
}*/

#[derive(Deserialize)]
pub struct Email{
    address: Box<str>
}

#[derive(Deserialize)]
pub struct UpdateMiiData {
    name: Box<str>,
    primary: crate::xml::YesNoVal,
    data: Box<str>,
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

    let verification_code: i32 = rand::thread_rng().gen_range(100_000..1_000_000);

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
            name,
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
                                 mii_data,
                                 verification_code
                                 ) VALUES (
                                            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14
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
        data.as_ref(),
        verification_code,
    ).execute(database).await.unwrap();

    //generate_s3_images(pid, &data).await;

    if let Err(e) = send_verification_email(address.as_ref(), verification_code, user_id.as_ref()).await {
        println!("Failed to send verification email: {e}");
    }

    Ok(
        Xml(AccountCreationResponseData{
            pid
        })
    )
}

// #[derive(Serialize)]
// struct DevAttr{
//
// }

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
pub struct GetOwnProfileData{
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
    Ds(Xml(build_profile(user.into())))
}

#[get("/v1/api/people/@me/devices/owner")]
pub fn get_device_owner(user: Auth<false>) -> Ds<Xml<GetOwnProfileData>>{
    Ds(Xml(build_profile(user.into())))
}

#[post("/v1/api/people/@me/devices")]
pub fn get_own_device(user: Auth<false>) -> Ds<Xml<GetOwnProfileData>>{
    Ds(Xml(build_profile(user.into())))
}

pub fn build_profile(user: User) -> GetOwnProfileData {
    let User {
        username,
        pid,
        // account_level,
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
        // verification_code,
        ..
    } = user.into();

    let timezone_offset = (&*OFFSET_FROM_TIMEZONE).get(&timezone).unwrap().to_owned();

    let mii_data = mii_data
        .replace("\n", "")
        .replace("\t", "")
        .replace("\r", "")
        .replace(" ", "");


        GetOwnProfileData {
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
            email: EmailInfoOwnProfileData {
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
            mii: MiiDataOwnProfileData {
                id: gxhash32(mii_data.as_bytes(), 0),
                mii_hash: hex::encode(bytemuck::bytes_of(
                    &(gxhash64(mii_data.as_bytes(), 1) & !(0x1000000000000000))
                )),
                name: mii::MiiData::read(&mii_data)
                .map(|v| v.name)
                .unwrap_or_else(|| "INVALID".to_string()),
                primary: YesNoVal(true),
                data: mii_data,
                status: "COMPLETED".to_string(),
                mii_images: MiiImages {
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
}

#[put("/v1/api/people/@me/miis/@primary", data = "<data>")]
pub async fn change_mii(
    database: &State<Pool>,
    auth: Auth<false>,
    data: Xml<UpdateMiiData>,
) -> Result<(), Option<Errors<'static>>> {
    let db = database.inner();
    let pid = auth.pid;
    let mii_data = data.data.as_ref();

    println!("Received new Mii data update for PID {}", pid);

    let result = sqlx::query!(
        "UPDATE users SET mii_data = $1 WHERE pid = $2",
        mii_data,
        pid
    )
        .execute(db)
        .await;

    if let Err(e) = result {
        println!("Failed to update Mii data for PID {}: {:?}", pid, e);
        return Err(Some(DATABASE_ERROR));
    }

    println!("Successfully updated Mii data for PID {}", pid);

    Ok(())
}