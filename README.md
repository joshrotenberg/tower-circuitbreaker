# tower-circuitbreaker

A circuit breaker middleware for Tower services that improves resilience in distributed systems.

[![Crates.io](https://img.shields.io/crates/v/tower-circuitbreaker.svg)](https://crates.io/crates/tower-circuitbreaker)
[![Docs.rs](https://docs.rs/tower-circuitbreaker/badge.svg)](https://docs.rs/tower-circuitbreaker)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/tower-circuitbreaker.svg)](./LICENSE-MIT)
[![GitHub](https://img.shields.io/badge/github-joshrotenberg/tower--circuitbreaker-8da0cb?logo=github)](https://github.com/joshrotenberg/tower-circuitbreaker)

## Table of Contents

- [What is a Circuit Breaker?](#what-is-a-circuit-breaker)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Error Handling](#error-handling)
- [Complete Example](#complete-example)
- [Customization](#customization)
  - [Failure Classification](#failure-classification)
  - [Named Circuit Breakers](#named-circuit-breakers)
  - [Metrics and Tracing](#metrics-and-tracing)
- [Manual Control](#manual-control)
- [When to Use Circuit Breakers](#when-to-use-circuit-breakers)
- [Tower Ecosystem Integration](#tower-ecosystem-integration)
- [License](#license)
- [Contributing](#contributing)

## What is a Circuit Breaker?

A circuit breaker is a design pattern that prevents cascading failures in distributed systems. Like an electrical circuit breaker, it "trips" when detecting problems, preventing further calls to a failing service:

- **Closed State**: Normal operation - requests pass through to the service
- **Open State**: After detecting failures, the circuit breaker fails fast without calling the service
- **Half-Open State**: After a cooldown period, allows limited test requests to check if the service has recovered

This pattern:
- Prevents overwhelming already-struggling services
- Avoids cascading failures across your system
- Enables faster recovery from failures
- Improves user experience by failing fast when services are unavailable

## Features

- **State Management**: Automatically transitions between Closed, Open, and Half-Open states
- **Configurable Thresholds**: Set failure rate threshold and sliding window size
- **Customizable Failure Detection**: Define what counts as a failure with custom classifiers
- **Observability**: Optional metrics and tracing support
- **Manual Control**: Force open/closed and reset operations
- **Minimal Dependencies**: Built on Tower and Tokio

## Installation

```toml
[dependencies]
tower-circuitbreaker = "0.1"

# Optional features
tower-circuitbreaker = { version = "0.1", features = ["metrics", "tracing"] }
```

## Quick Start

```rust
use std::time::Duration;
use tower::ServiceBuilder;
use tower_circuitbreaker::circuit_breaker_builder;

#[tokio::main]
async fn main() {
    let cb = circuit_breaker_builder::<String, ()>()
        .failure_rate_threshold(0.5)      // Open circuit when 50% of calls fail
        .sliding_window_size(20)          // Consider the last 20 calls
        .wait_duration_in_open(Duration::from_secs(5))  // Stay open for 5 seconds
        .permitted_calls_in_half_open(2)  // Allow 2 test calls when half-open
        .minimum_number_of_calls(5)  // Don't evaluate failure rate until at least 5 calls 
        .build();

    let mut svc = ServiceBuilder::new()
        .layer(cb)
        .service_fn(|req| async move { Ok::<_, ()>(req) });

    let resp = svc.call("hello".to_string()).await.unwrap();
    assert_eq!(resp, "hello");
}
```

## Error Handling

The circuit breaker wraps service errors in `CircuitBreakerError`, which provides helpful methods:

```rust
match service.call(request).await {
    Ok(response) => println!("Success: {}", response),
    Err(e) if e.is_circuit_open() => println!("Circuit is open, failing fast"),
    Err(e) => println!("Service error: {:?}", e.into_inner()),
}
```

## Complete Example

This example demonstrates the circuit breaker's state transitions:

```rust
use std::time::Duration;
use tokio::time::sleep;
use tower::{Service, ServiceBuilder, service_fn};
use tower_circuitbreaker::circuit_breaker_builder;

#[tokio::main]
async fn main() {
    // Service that succeeds with true and fails with false
    let boolean_service = service_fn(|req: bool| async move {
        if req {
            Ok::<_, ()>(req.to_string())
        } else {
            Err::<String, _>(())
        }
    });

    // Configure circuit breaker
    let breaker = circuit_breaker_builder::<String, ()>()
        .failure_rate_threshold(0.5)           // 50% failure threshold
        .sliding_window_size(2)                // Consider last 2 calls
        .wait_duration_in_open(Duration::from_secs(1))  // Open for 1 second
        .permitted_calls_in_half_open(1)       // Allow 1 test call
        .name("example-circuit")               // Name for logs/metrics
        .build();

    // Create service with circuit breaker
    let mut svc = ServiceBuilder::new()
        .layer(breaker)
        .service(boolean_service);

    // Initially closed
    println!("Circuit state: {:?}", svc.state().await);  // Closed

    // Two failures open the circuit
    svc.call(false).await.expect_err("Should fail");
    svc.call(false).await.expect_err("Should fail");
    println!("Circuit state: {:?}", svc.state().await);  // Open

    // Wait for cooldown
    sleep(Duration::from_secs(1)).await;

    // Test call in half-open state
    let result = svc.call(true).await;
    println!("Test call result: {:?}", result);
    println!("Circuit state: {:?}", svc.state().await);  // Closed again
}
```

## Example Applications

The repository includes runnable examples that demonstrate different use cases for the circuit breaker:

### Simple Example

A basic example showing circuit breaker state transitions with a simple boolean service:

```bash
# Run the simple example
cargo run --example simple

# With tracing enabled
RUST_LOG=debug cargo run --example simple --features tracing
```

### HTTP Client Example

A more realistic example showing how to use the circuit breaker with an HTTP client:

```bash
# Run the HTTP client example
cargo run --example http_client

# With tracing enabled
RUST_LOG=debug cargo run --example http_client --features tracing
```

This example demonstrates:
- Using the circuit breaker with a reqwest HTTP client
- Setting up a mock server for testing
- Handling HTTP errors with the circuit breaker
- Observing state transitions in a realistic scenario

## Customization

### Failure Classification

By default, any `Err` result is considered a failure. You can customize this:

```rust
// When configuring your circuit breaker
let circuit_breaker = circuit_breaker_builder::<Response, Error>()
    .failure_classifier(|result| {
        match result {
            // Consider HTTP 500 errors as failures
            Ok(resp) if resp.status().is_server_error() => true,
            // Consider timeouts as failures
            Err(e) if e.is_timeout() => true,
            // Other errors are failures
            Err(_) => true,
            // All other responses are successes
            _ => false,
        }
    })
    .build();
```

### Named Circuit Breakers

Name your circuit breakers for better observability:

```rust
// When configuring your circuit breaker
let circuit_breaker = circuit_breaker_builder::<Response, Error>()
    .name("user-service-circuit")
    .build();
```

### Metrics and Tracing

Enable the optional features for observability:

```rust
// In Cargo.toml
tower-circuitbreaker = { version = "0.1", features = ["metrics", "tracing"] }
```

This provides:
- Metrics: counts of calls, transitions, and current state
- Tracing: logs state transitions and call decisions

When the metrics feature is enabled, the following metrics are emitted:

| Metric Name | Type | Tags | Description |
|-------------|------|------|-------------|
| circuitbreaker_calls_total | Counter | outcome=success/failure/rejected | Total number of calls through the circuit breaker |
| circuitbreaker_transitions_total | Counter | from=Closed/Open/HalfOpen, to=Closed/Open/HalfOpen | State transitions |
| circuitbreaker_state | Gauge | state=Closed/Open/HalfOpen | Current state (1.0 = active) |

## Manual Control

You can manually control the circuit state:

```rust
// Force the circuit open during maintenance
service.force_open().await;

// Force the circuit closed
service.force_closed().await;

// Reset the circuit state and counters
service.reset().await;

// Check current state
let state = service.state().await;
```

## When to Use Circuit Breakers

Circuit breakers are particularly valuable in the following scenarios:

- **Microservice Architectures**: When your application depends on multiple services, circuit breakers prevent failures in one service from cascading to others.

- **External API Calls**: When integrating with third-party APIs that may experience downtime or rate limiting.

- **Database Operations**: To prevent overwhelming a struggling database with requests.

- **High-Traffic Systems**: In systems handling many concurrent requests, circuit breakers can help maintain overall system stability when parts of the system degrade.

- **Graceful Degradation**: When you want to provide fallback behavior or partial functionality when a dependency is unavailable.

Circuit breakers work best when combined with other resilience patterns like timeouts, retries, and bulkheads.

## Tower Ecosystem Integration

This library integrates seamlessly with the Tower ecosystem, allowing you to compose it with other Tower middleware:

```rust
let service = ServiceBuilder::new()
    .timeout(Duration::from_secs(1))         // Add timeout
    .rate_limit(100, Duration::from_secs(1)) // Add rate limiting
    .layer(circuit_breaker_layer)            // Add circuit breaking
    .retry(retry_policy)                     // Add retries
    .service(my_service);                    // Your service
```

Tower's middleware composition allows you to build a resilient service stack tailored to your specific needs. The circuit breaker can be positioned strategically in this stack:

- Place it **after** timeout middleware to ensure timeouts are counted as failures
- Place it **before** retry middleware if you want retries to happen only when the circuit is closed
- Place it **after** retry middleware if you want the circuit to open only when retries have been exhausted

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE2) or [MIT license](LICENSE-MIT) at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
