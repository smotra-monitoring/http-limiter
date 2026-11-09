use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let result = match http_limiter::Config::from_env().validate() {
        Ok(config) => http_limiter::run(config).await,
        Err(e) => Err(e.into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
