use std::path::PathBuf;

use actix_files::NamedFile;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{
    get, post,
    web::{self},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use uuid::Uuid;

use crate::settings;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/defaults/detector")]
async fn defaults_detector() -> impl Responder {
    let settings = settings::DetectorSettings::default();
    match serde_json::to_string(&settings) {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(err) => HttpResponse::InternalServerError()
            .body(format!("Failed to serialize detector settings: {err}")),
    }
}

#[get("/defaults/ocr")]
async fn defaults_ocr() -> impl Responder {
    let settings = settings::OCRSettings::default();
    match serde_json::to_string(&settings) {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(err) => HttpResponse::InternalServerError()
            .body(format!("Failed to serialize OCR settings: {err}")),
    }
}

#[get("/defaults/inpainter")]
async fn defaults_inpainter() -> impl Responder {
    let settings = settings::InpainterSettings::default();
    match serde_json::to_string(&settings) {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(err) => HttpResponse::InternalServerError()
            .body(format!("Failed to serialize inpainter settings: {err}")),
    }
}

#[get("/defaults/mask_refinement")]
async fn defaults_mask_refinement() -> impl Responder {
    let settings = settings::MaskRefinementSettings::default();
    match serde_json::to_string(&settings) {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(err) => HttpResponse::InternalServerError().body(format!(
            "Failed to serialize mask refinement settings: {err}"
        )),
    }
}

#[get("/defaults/translator")]
async fn defaults_translator() -> impl Responder {
    let settings = settings::TranslatorSettings::default();
    match serde_json::to_string(&settings) {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(err) => HttpResponse::InternalServerError()
            .body(format!("Failed to serialize translator settings: {err}")),
    }
}

const UPLOAD_DIR: &str = "./uploads";

#[get("/image/{uuid}")]
async fn get_image(uuid: web::Path<String>, req: HttpRequest) -> impl Responder {
    let filename = uuid.into_inner();
    if let Err(_) = Uuid::parse_str(&filename) {
        return HttpResponse::BadRequest().body("Invalid UUID");
    }
    let path = PathBuf::from(UPLOAD_DIR).join(&filename);

    if !path.exists() {
        return HttpResponse::NotFound().body("Image not found");
    }

    match NamedFile::open(path) {
        Ok(file) => file.use_last_modified(true).into_response(&req),
        Err(_) => HttpResponse::InternalServerError().body("Failed to read image"),
    }
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    file: TempFile,
}

#[post("/image/upload")]
async fn upload_image(MultipartForm(form): MultipartForm<UploadForm>) -> impl Responder {
    std::fs::create_dir_all(UPLOAD_DIR).ok();
    let p = form.file.file.path();
    let uuid = Uuid::new_v4().to_string();
    let to = PathBuf::from(UPLOAD_DIR).join(&uuid);
    if let Err(err) = std::fs::rename(p, to) {
        return HttpResponse::InternalServerError().body(format!("Failed to rename file: {}", err));
    }
    HttpResponse::Ok().body(uuid)
}

pub async fn main(host: &str, port: u16) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(defaults_detector)
            .service(defaults_ocr)
            .service(defaults_mask_refinement)
            .service(defaults_translator)
            .service(defaults_inpainter)
            .service(upload_image)
            .service(get_image)
            .service(hello)
    })
    .bind((host, port))?
    .run()
    .await
}
