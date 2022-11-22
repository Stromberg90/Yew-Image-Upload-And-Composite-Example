use axum::{extract::Multipart, http::StatusCode, routing::post, Router};
use futures::TryStreamExt;
use ril::{Image, Paste, Rgb, Rgba};
use simple_logger::SimpleLogger;
use std::{io, net::SocketAddr};
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let app = Router::new().route("/", post(accept_form)).layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
            .allow_credentials(false),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn accept_form(mut multipart: Multipart) -> Result<String, (StatusCode, String)> {
    dbg!(&multipart);
    if let Some(field) = multipart.next_field().await.unwrap() {
        let body_with_io_error = field.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        let mut buffer = Vec::new();
        body_reader.read_to_end(&mut buffer).await.unwrap();

        let mut user_image =
            Image::convert::<Rgba>(Image::<Rgb>::from_bytes_inferred(&buffer).unwrap());
        user_image.resize(1329, 1292, ril::ResizeAlgorithm::Lanczos3);

        let template = Image::<Rgba>::open("templates/mask.png").unwrap();

        let mut background =
            Image::new(template.width(), template.height(), Rgba::new(0, 0, 0, 255));
        background.paste(300, 300, user_image);

        background.draw(&Paste::new(template).with_overlay_mode(ril::OverlayMode::Merge));

        let mut encoded_image = Vec::new();
        background
            .encode(ril::ImageFormat::Png, &mut encoded_image)
            .unwrap();

        return Ok(base64::encode(encoded_image));
    }

    Err((
        axum::http::StatusCode::NOT_IMPLEMENTED,
        "Did not get a file".to_string(),
    ))
}
