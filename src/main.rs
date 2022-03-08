use actix_web::client::Client;
use actix_web::http::StatusCode;
use actix_web::{get, App, HttpResponse, HttpServer, Result};
use cached::proc_macro::cached;
use chrono;
use chrono::Datelike;

mod parser;
use parser::get_full_url;

#[get("/")]
async fn apog() -> Result<HttpResponse> {
    let apod_url = get_apog_url().await;
    return og_response(apod_url);
}

#[get("/cats")]
async fn cpod() -> Result<HttpResponse> {
    let cats_url = get_cat_pod_url().await;
    return og_response(cats_url);
}

fn og_response(full_url: String) -> Result<HttpResponse> {
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
        full_url
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
        .header("User-Agent", "Actix-web apod-og")
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

#[cached(time = 3600)]
async fn get_cat_pod_url() -> String {
    println!("calling get_cat_pod_url");

    let gifday = chrono::Local::now().weekday() == chrono::Weekday::Mon;
    let mime = if !gifday { "gif" } else { "jpg" };

    let api_key = std::env::var("CAT_API_KEY").unwrap_or("DEMO-API-KEY".to_string());
    let client = Client::new();
    let body = client
        .get(format!("https://api.thecatapi.com/v1/images/search?mime_types={}&limit=1", mime))
        .header("User-Agent", "Actix-web apod-og")
        .header("X-Api-Key", api_key)
        .send()
        .await
        .unwrap()
        .body()
        .await
        .unwrap();
    let parsed = json::parse(std::str::from_utf8(&body).unwrap()).unwrap();
    let full_url = &parsed[0]["url"];

    println!("got full_url {:?}", full_url);
    return format!("{}", full_url)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(apog).service(redirect).service(cpod))
        .bind(format!(
            "0.0.0.0:{}",
            std::env::var("PORT").unwrap_or("8080".to_string())
        ))?
        .run()
        .await
}
