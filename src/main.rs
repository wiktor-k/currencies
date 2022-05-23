use clap::Parser;
use service_binding::Binding;

#[derive(Debug, Parser)]
struct Args {
    #[clap(
        short = 'H',
        long,
        env = "HOST",
        default_value = "tcp://127.0.0.1:8080"
    )]
    host: Binding,

    #[clap(
        short,
        long,
        env = "TARGET",
        default_value = "https://api.nbp.pl/api/exchangerates/tables/A/"
    )]
    target: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = Args::parse();

    currencies::start(args.target, args.host.try_into()?)?.await
}
