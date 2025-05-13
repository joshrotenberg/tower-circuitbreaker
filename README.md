# tower-circuitbreaker

Circuit breaker middleware for Tower services.

[![Crates.io](https://img.shields.io/crates/v/tower-circuitbreaker.svg)](https://crates.io/crates/tower-circuitbreaker)
[![Docs.rs](https://docs.rs/tower-circuitbreaker/badge.svg)](https://docs.rs/tower-circuitbreaker)

## Installation

```toml
[dependencies]
tower-circuitbreaker = "0.1"

# Optional features
tower-circuitbreaker = { version = "0.1", features = ["metrics","tracing"] }
```

## Quick Start

```rust
use std::time::Duration;
use tower::ServiceBuilder;
use tower_circuitbreaker::builder;

#[tokio::main]
async fn main() {
    let cb = builder::<String, ()>()
        .failure_rate_threshold(0.5)
        .sliding_window_size(20)
        .wait_duration_in_open(Duration::from_secs(5))
        .permitted_calls_in_half_open(2)
        .build();

    let mut svc = ServiceBuilder::new()
        .layer(cb)
        .service_fn(|req| async move { Ok::<_, ()>(req) });

    let resp = svc.call("hello".to_string()).await.unwrap();
    assert_eq!(resp, "hello");
}
```

## Customization

- **Failure classifier:** `.failure_classifier(|res| …)`
- **Named breakers:** `.name("my_cb")`
- **Metrics:** enable `metrics` feature
- **Tracing:** enable `tracing` feature

## Docs & Crates

- [API docs](https://docs.rs/tower-circuitbreaker)
- [Crate](https://crates.io/crates/tower-circuitbreaker)
