use axum::{
    body::Body,
    extract::State,
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::{headers::Host, TypedHeader};
use clap::Parser;
use config::{read_config, Config};
use s3::{creds::Credentials, Bucket, Region};
use std::{collections::HashMap, net::SocketAddr, process::exit, sync::Arc};
use tracing::error;

mod config;

#[derive(Parser, Debug)]
#[command(version = env!("VERSION_STRING"))]
struct Args {
    #[arg(long, default_value = "config.json")]
    config: String,
    #[arg(long, default_value = "info")]
    log_level: tracing::Level,
    #[arg(short, long, default_value = "[::]:8000")]
    listen: SocketAddr,
}

type SharedState = Arc<AppState>;
struct AppState {
    buckets: HashMap<String, Bucket>,
}

fn make_state(config: Config) -> AppState {
    let credentials = Credentials::from_env().unwrap();

    let region = Region::Custom {
        region: config.s3_region,
        endpoint: config.s3_endpoint,
    };

    let mut buckets = HashMap::new();
    for (k, v) in config.sites {
        buckets.insert(
            k,
            *Bucket::new(&v.bucket, region.clone(), credentials.clone())
                .unwrap()
                .with_path_style(),
        );
    }

    AppState { buckets }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .init();

    let config = match read_config(args.config) {
        Ok(config) => config,
        Err(e) => {
            error!(error = ?e, "error reading state");
            exit(1)
        }
    };

    let state = make_state(config);

    let app = Router::new()
        .fallback(get(root))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(args.listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root(
    State(state): State<SharedState>,
    uri: Uri,
    TypedHeader(host): TypedHeader<Host>,
) -> Response {
    let host = host.hostname().to_lowercase();

    match state.buckets.get(&host) {
        Some(bucket) => {
            let mut path = uri.path().to_string();
            if path.ends_with('/') {
                path += "index.html";
            }

            match bucket.get_object_stream(path).await {
                Ok(data) => Body::from_stream(data.bytes).into_response(),
                Err(s3::error::S3Error::HttpFailWithBody(404, _)) => {
                    StatusCode::NOT_FOUND.into_response()
                }
                Err(e) => {
                    error!(error = ?e, uri = ?uri, host = host, "get object error");
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
