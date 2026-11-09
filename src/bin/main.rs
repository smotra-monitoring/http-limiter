#[tokio::main]
async fn main() -> http_limiter::Result<()> {
    let config = http_limiter::Config::from_env().validate()?;
    http_limiter::run(config).await
}
