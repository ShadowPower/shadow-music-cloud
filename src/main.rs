use actix_web::{App, HttpServer, middleware};
use shadow_music_cloud::{service::*};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
        .wrap(middleware::Compress::default())
        .service(media::list)
        .service(media::list_diff)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}