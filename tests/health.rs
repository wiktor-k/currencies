use currencies::start;
use std::net::TcpListener;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[actix_rt::test]
async fn healthz_test() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = format!("http://127.0.0.1:{}/healthz", listener.local_addr()?.port());
    let _ = tokio::spawn(start("".into(), listener.into())?);
    let client = awc::Client::default();
    let response = client.get(address).send().await.unwrap();

    assert_eq!(200, response.status().as_u16());

    Ok(())
}

#[actix_rt::test]
async fn frames_test() -> std::io::Result<()> {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/source"))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
  {
    "table": "A",
    "no": "097/A/NBP/2022",
    "effectiveDate": "2022-05-20",
    "rates": [
      {
        "currency": "",
        "code": "EUR",
        "mid": 4.1279
      },
      {
        "currency": "dolar amerykański",
        "code": "USD",
        "mid": 4.3832
      },
      {
        "currency": "",
        "code": "CHF",
        "mid": 4.0986
      }]}]"#,
            "application/json",
        ))
        .expect(1)
        .mount(&mock)
        .await;

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = format!("http://127.0.0.1:{}/", listener.local_addr()?.port());
    let _ = tokio::spawn(start(format!("{}/source", mock.uri()), listener.into())?);
    let client = awc::Client::default();
    let mut response = client.get(address).send().await.unwrap();

    assert_eq!(200, response.status().as_u16());
    assert_eq!(
        r#"{"frames":[{"text":"4.1279","icon":"i7792"},{"text":"4.3832","icon":"i4935"},{"text":"4.0986","icon":"i469"}]}"#,
        response.body().await.unwrap()
    );

    Ok(())
}
