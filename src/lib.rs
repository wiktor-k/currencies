use actix_web::{
    dev::Server,
    middleware::Logger,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use reqwest::Client;
use serde::Serialize;
use service_binding::Listener;

json_typegen::json_typegen!("Tables", "src/tables.json");

struct CurrencyIcon<'a>(&'a str, &'a str);

static CURRENCIES: &[CurrencyIcon] = &[
    CurrencyIcon("EUR", "i7792"),
    CurrencyIcon("USD", "i4935"),
    CurrencyIcon("CHF", "i469"),
];

#[derive(Debug, Serialize)]
struct Animation<'a> {
    frames: Vec<Frame<'a>>,
}

#[derive(Debug, Serialize)]
struct Frame<'a> {
    text: String,
    icon: &'a str,
}

fn render_to_frames(tables: Vec<Table>) -> Vec<Frame<'static>> {
    let rates = tables.into_iter().flat_map(|table| table.rates);
    let mut frames = vec![];
    for rate in rates {
        for currency in CURRENCIES {
            if rate.code == currency.0 {
                frames.push(Frame {
                    text: format!("{:.4}", rate.mid),
                    icon: currency.1,
                });
            }
        }
    }

    frames
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
}

impl actix_web::error::ResponseError for Error {}

async fn render(target: Data<String>, client: Data<Client>) -> Result<HttpResponse, Error> {
    let tables = client
        .get(target.get_ref())
        .header(
            "User-Agent",
            "currencies (+https://metacode.biz/@wiktor#currencies2)",
        )
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<Tables>()
        .await?;

    let frames = render_to_frames(tables);

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .append_header(("Cache-Control", "max-age=3600"))
        .append_header(("X-Collation-Id", uuid::Uuid::new_v4().to_string()))
        .body(serde_json::to_string(&Animation { frames })?))
}

async fn healthz() -> impl Responder {
    "OK"
}

pub fn start(target: String, listener: Listener) -> std::io::Result<Server> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Client::default()))
            .app_data(Data::new(target.clone()))
            .route("/healthz", web::get().to(healthz))
            .route("/currencies", web::get().to(render))
            .route("/", web::get().to(render))
            .wrap(Logger::default())
    });

    let server = match listener {
        Listener::Unix(listener) => server.listen_uds(listener),
        Listener::Tcp(listener) => server.listen(listener),
        _ => return Err(std::io::Error::other("Unsupported listener type")),
    }?
    .run();

    Ok(server)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fransform() -> testresult::TestResult {
        let tables = serde_json::from_reader(std::fs::File::open("src/tables.json")?)?;
        let frames = render_to_frames(tables);
        assert_eq!(frames.len(), 3);
        let frame = &frames[0];
        assert_eq!(frame.text, "3.9571");
        assert_eq!(frame.icon, "i4935");
        Ok(())
    }
}
