use actix_web::{
    dev::Server,
    middleware::Logger,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use awc::{
    error::{JsonPayloadError, SendRequestError},
    Client,
};

use serde::{Deserialize, Serialize};
use service_binding::Listener;

#[derive(Debug, Deserialize)]
struct Rate {
    code: String,
    mid: f64,
}

#[derive(Debug, Deserialize)]
struct Table {
    rates: Vec<Rate>,
}

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

#[derive(Debug)]
struct Error;

impl actix_web::error::ResponseError for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<SendRequestError> for Error {
    fn from(_: SendRequestError) -> Self {
        Self
    }
}

impl From<JsonPayloadError> for Error {
    fn from(_: JsonPayloadError) -> Self {
        Self
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self
    }
}

async fn render(target: Data<String>, client: Data<Client>) -> Result<HttpResponse, Error> {
    let tables = client
        .get(target.get_ref())
        .insert_header((
            "User-Agent",
            "currencies (+https://metacode.biz/@wiktor#currencies2)",
        ))
        .insert_header(("Accept", "application/json"))
        .send()
        .await?
        .json::<Vec<Table>>()
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
            .wrap(Logger::default())
    });

    let server = match listener {
        Listener::Unix(listener) => server.listen_uds(listener),
        Listener::Tcp(listener) => server.listen(listener),
    }?
    .run();

    Ok(server)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fransform() {
        let tables = vec![Table {
            rates: vec![Rate {
                code: "USD".into(),
                mid: 1.23,
            }],
        }];
        let frames = render_to_frames(tables);
        assert_eq!(frames.len(), 1);
        let frame = &frames[0];
        assert_eq!(frame.text, "1.2300");
        assert_eq!(frame.icon, "i4935");
    }
}
