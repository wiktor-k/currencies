use chrono::Utc;
use http::{Request, Response};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Server,
};
use hyper_tls::HttpsConnector;
use std::{convert::Infallible, env, net::SocketAddr, sync::Arc};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

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

struct Handler<T: hyper::client::connect::Connect + Clone + Send + Sync + 'static> {
    url: String,
    client: hyper::client::Client<T>,
}

impl<T: hyper::client::connect::Connect + Clone + Send + Sync + 'static> Handler<T> {
    fn create(url: String, connector: T) -> Self {
        Self {
            url,
            client: Client::builder().build::<_, hyper::Body>(connector),
        }
    }

    async fn handle(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let request_id = Uuid::new_v4();
        println!(
            "ts={} id={} req={:?}",
            Utc::now().to_rfc3339(),
            request_id,
            req
        );

        let req = Request::get(&self.url)
            .header(
                "User-Agent",
                "currencies (+https://metacode.biz/@wiktor#currencies)",
            )
            .header("Accept", "application/json")
            .body(hyper::body::Body::default())
            .unwrap();

        match self.client.request(req).await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if status == 200 {
                    let bytes = hyper::body::to_bytes(resp.into_body()).await?;
                    let tables: Vec<Table> = serde_json::from_slice(&bytes).unwrap();
                    let mut frames = Vec::new();
                    for table in tables {
                        for rate in table.rates {
                            for currency in CURRENCIES {
                                if rate.code == currency.0 {
                                    frames.push(Frame {
                                        text: format!("{:.4}", rate.mid),
                                        icon: currency.1,
                                    });
                                }
                            }
                        }
                    }
                    let mut resp = Response::new(Body::from(
                        serde_json::to_string_pretty(&Animation { frames }).unwrap(),
                    ));
                    let headers = resp.headers_mut();
                    headers.append("Cache-Control", "max-age=3600".parse().unwrap());
                    headers.append("Content-Type", "application/json".parse().unwrap());
                    Ok(resp)
                } else {
                    println!(
                        "ts={} id={} st={:?}",
                        Utc::now().to_rfc3339(),
                        request_id,
                        status
                    );

                    Ok(Response::new(Body::from("error")))
                }
            }
            Err(error) => {
                println!(
                    "ts={} id={} err={:?}",
                    Utc::now().to_rfc3339(),
                    request_id,
                    error
                );

                Ok(Response::new(Body::from("error")))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ts={} status=starting", Utc::now().to_rfc3339());
    let port = env::var("PORT")
        .unwrap_or(String::from("3000"))
        .parse()
        .expect("PORT must be a number");
    let url = env::var("URL").expect("Need target URL");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!(
        "ts={} status=listening addr={}",
        Utc::now().to_rfc3339(),
        addr
    );

    let handler = Arc::new(Handler::create(url, HttpsConnector::new()));

    let make_svc = make_service_fn(|_| {
        let handler = Arc::clone(&handler);
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let handler = Arc::clone(&handler);
                async move { handler.handle(req).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}
