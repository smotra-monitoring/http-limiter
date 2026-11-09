#[tokio::main]
async fn main() -> std::io::Result<()> {
    http_limiter::run(http_limiter::Config::from_env()).await
}
