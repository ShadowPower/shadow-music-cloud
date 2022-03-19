use actix_web::{get, post, HttpResponse, Responder, web, Result};

use crate::{infra::{file_utils}};

#[get("/media/list")]
pub async fn list() -> Result<impl Responder> {
    let audio_files = file_utils::list_audio_file();
    // TODO
    Ok(web::Json(audio_files))
}

#[post("/media/list-diff")]
pub async fn list_diff(file_info_hashs: web::Json<Vec<String>>) -> impl Responder {
    for file_info_hash in file_info_hashs.into_inner() {
        println!("{}", file_info_hash);
    }
    HttpResponse::Ok().body("not implemented")
}