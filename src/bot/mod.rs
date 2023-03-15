mod bind;
mod key;
mod ban;

use std::borrow::Cow;
use std::num::ParseIntError;
use actix_web::{HttpRequest, HttpResponse, patch, put, Responder};
use simple_log::{info, warn};
use url_encoded_data::UrlEncodedData;
use crate::bot::ban::{ban_user, ban_user_qq};
use crate::bot::bind::bind_qq;
use crate::bot::key::examine_key;

#[patch("/user")]
pub async fn user_patch(req: HttpRequest, _req_body: String) -> impl Responder {
    let uri = req.uri().to_string();
    let uri_encoded = UrlEncodedData::from(uri.as_str());
    let ip = req.peer_addr().unwrap().ip().to_string();

    if !uri_encoded.exists("uuid") || !uri_encoded.exists("qq") || !uri_encoded.exists("key") {
        warn!("400/user(patch)->{}: Missing argument(s).", ip.to_string());
        return HttpResponse::BadRequest().body("Missing argument(s).");
    }

    let uuid = uri_encoded.as_map_of_single_key_to_first_occurrence_value().get(&Cow::from("name")).unwrap().to_string();
    let qq = uri_encoded.as_map_of_single_key_to_first_occurrence_value().get(&Cow::from("qq")).unwrap().to_string();
    let qq = match qq.parse::<i64>() {
        Ok(a) => {a}
        Err(err) => {
            warn!("400/user(patch)->{}: Missing argument(s).", ip.to_string());
            return HttpResponse::BadRequest().body("Cannot parse qq: ".to_string() + err.to_string().as_str());
        }
    };
    let key = uri_encoded.as_map_of_single_key_to_first_occurrence_value().get(&Cow::from("key")).unwrap().to_string();
    if examine_key(key).await.is_err() {
        warn!("401/user(patch)->{}: Wrong key.", ip.to_string());
        return HttpResponse::Unauthorized().body("Wrong key.");
    }

    return match bind_qq(uuid, qq).await {
        Ok(_) => {
            info!("200/user(patch)->{}", ip.to_string());
            HttpResponse::Ok().body("")
        }
        Err(_) => {
            warn!("500/user(patch)->{}: User not found or already bound.", ip.to_string());
            HttpResponse::InternalServerError().body("User not found or already bound.")
        }
    };
}

#[put("/user")]
pub async fn user_put(req: HttpRequest, _req_body: String) -> impl Responder {

    let uri = req.uri().to_string();
    let uri_encoded = UrlEncodedData::from(uri.as_str());
    let ip = req.peer_addr().unwrap().ip().to_string();

    if !((uri_encoded.exists("uuid") || uri_encoded.exists("qq")) && uri_encoded.exists("key")) {
        warn!("400/user(patch)->{}: Missing argument(s).", ip.to_string());
        return HttpResponse::BadRequest().body("Missing argument(s).");
    }
    let key = uri_encoded.as_map_of_single_key_to_first_occurrence_value().get(&Cow::from("key")).unwrap().to_string();

    let reason = if uri_encoded.exists("reason") {
        uri_encoded.as_map_of_single_key_to_first_occurrence_value().get(&Cow::from("reason")).unwrap().to_string()
    } else {
        "".to_string()
    };

    if examine_key(key).await.is_err() {
        warn!("401/user(patch)->{}: Wrong key.", ip.to_string());
        return HttpResponse::Unauthorized().body("Wrong key.");
    }

    return if uri_encoded.exists("uuid") {
        let uuid = uri_encoded.as_map_of_single_key_to_first_occurrence_value().get(&Cow::from("uuid")).unwrap().to_string();
        match ban_user(uuid, reason).await {
            Ok(_) => {
                info!("200/user(patch)->{}", ip.to_string());
                HttpResponse::Ok().body("")
            }
            Err(_) => {
                warn!("500/user(patch)->{}: User not found or already disabled.", ip.to_string());
                HttpResponse::InternalServerError().body("User not found or already disabled.")
            }
        }
    } else {
        let qq = match uri_encoded.as_map_of_single_key_to_first_occurrence_value().get(&Cow::from("qq")).unwrap().to_string().parse::<i64>() {
            Ok(a) => a,
            Err(err) => {
                warn!("500/user(patch)->{}: Cannot parse qq: {}.", ip.to_string(), err.to_string());
                return HttpResponse::InternalServerError().body("cannot parse qq: ".to_string() + err.to_string().as_str() + ".");
            }
        };
        match ban_user_qq(qq, reason).await {
            Ok(_) => {
                info!("200/user(patch)->{}", ip.to_string());
                HttpResponse::Ok().body("")
            }
            Err(_) => {
                warn!("500/user(patch)->{}: User not found or already disabled.", ip.to_string());
                HttpResponse::InternalServerError().body("User not found or already disabled.")
            }
        }
    }
}