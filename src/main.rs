use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use prometheus_client::encoding::text::encode;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use url::form_urlencoded;

use crate::prometheus::{init_prometheus_sate, Labels, PrometheusState};

mod prometheus;
mod shelly;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    eprintln!("Listening on {}", addr);

    let state = Arc::new(init_prometheus_sate());

    let mut shutdown_stream = signal(SignalKind::terminate()).unwrap();

    if let Err(e) = Server::bind(&addr)
        .serve(make_service_fn(move |_conn| {
            let state = state.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let state = state.clone();
                    async move { Ok::<_, Infallible>(handle(req, state).await) }
                }))
            }
        }))
        .with_graceful_shutdown(async move {
            shutdown_stream.recv().await;
        })
        .await
    {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn handle(req: Request<Body>, state: Arc<PrometheusState>) -> Response<Body> {
    match req.uri().path() {
        "/" => index(),
        "/probe" => probe_shelly_plug(&req, state).await,
        _ => not_found(),
    }
}

async fn probe_shelly_plug(req: &Request<Body>, state: Arc<PrometheusState>) -> Response<Body> {
    let query = if let Some(q) = req.uri().query() {
        q
    } else {
        return Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(Body::from("Missing query string"))
            .unwrap();
    };
    let params = form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let target = if let Some(t) = params.get("target") {
        t
    } else {
        return Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(Body::from("Missing target parameter"))
            .unwrap();
    };
    match shelly::scrape_shelly_plug(target.to_string()).await {
        Ok(s) => {
            let mut buf = String::new();
            let label = &Labels {
                target: target.to_string(),
            };
            state
                .power_metric
                .get_or_create(label)
                .set(s.meters[0].power);

            state
                .uptime_metric
                .get_or_create(label)
                .set(s.uptime.try_into().unwrap());

            state
                .relay_ison_metric
                .get_or_create(label)
                .set(s.relays[0].ison.try_into().unwrap());

            state
                .is_valid_metric
                .get_or_create(label)
                .set(s.meters[0].is_valid.try_into().unwrap());

            state
                .total_power_metric
                .get_or_create(label)
                .set(s.meters[0].total.try_into().unwrap());

            return encode(&mut buf, &state.clone().registry)
                .map(|_| {
                    Response::builder()
                        .header(
                            hyper::header::CONTENT_TYPE,
                            "application/openmetrics-text; version=1.0.0; charset=utf-8",
                        )
                        .body(Body::from(buf))
                        .unwrap()
                })
                .unwrap();
        }
        Err(e) => {
            eprintln!("Error: {:#}", e);
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal Server Error"))
                .unwrap();
        }
    };
}

fn index() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Shelly Exporter running!"))
        .unwrap()
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}
