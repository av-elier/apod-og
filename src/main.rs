use actix_web::client::Client;
use actix_web::http::StatusCode;
use actix_web::{get, App, HttpResponse, HttpServer, Result};
use cached::proc_macro::cached;

mod parser;
use parser::get_full_url;

#[get("/")]
async fn apog() -> Result<HttpResponse> {
    let apog_url = get_apog_url().await;
    let body = format!(
        r#"<html><head>
<meta property="og:image" content="{0}" />
</head><body>
<style>
body {{
  background-image: url('{0}');
}}
</style>
</body></html>"#,
        apog_url
    );
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(body))
}

#[get("/307")]
async fn redirect() -> Result<HttpResponse> {
    let apog_url = get_apog_url().await;
    Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
        .set_header("Location", apog_url)
        .body(""))
}

#[cached(time = 3600)]
async fn get_apog_url() -> String {
    println!("calling get_apog_url");

    let client = Client::default();

    let html: actix_web::web::Bytes = client
        .get("https://apod.nasa.gov/apod/")
        .header("User-Agent", "Actix-web apog-og")
        .send()
        .await
        .unwrap()
        .body()
        .await
        .unwrap();

    let full_url = get_full_url(html);
    println!("got full_url {:?}", full_url);
    String::from(full_url)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(apog).service(redirect))
        .bind(format!(
            "0.0.0.0:{}",
            std::env::var("PORT").unwrap_or("8080".to_string())
        ))?
        .run()
        .await
}
