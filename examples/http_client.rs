//! HTTP client example for tower-circuitbreaker
//! Run with: cargo run --example http_client
//! With tracing: RUST_LOG=debug cargo run --example http_client --features tracing

use std::time::Duration;
use tokio::time::sleep;
use tower::{Service, ServiceBuilder };
use tower_circuitbreaker::circuit_breaker_builder;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};
use reqwest::{Client, Error};

// A service that wraps reqwest client to make HTTP requests
#[derive(Clone)]
struct HttpService {
    client: Client,
    base_url: String,
}

impl HttpService {
    fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
}

impl Service<bool> for HttpService {
    type Response = String;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, should_succeed: bool) -> Self::Future {
        let url = if should_succeed {
            format!("{}/success", self.base_url)
        } else {
            format!("{}/failure", self.base_url)
        };
        
        let client = self.client.clone();
        Box::pin(async move {
            let response = client.get(url).send().await?;
            let body = response.text().await?;
            Ok(body)
        })
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    // Start a mock server
    let mock_server = MockServer::start().await;
    
    // Configure the mock server to return success for /success endpoint
    Mock::given(method("GET"))
        .and(path("/success"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Success response"))
        .mount(&mock_server)
        .await;
    
    // Configure the mock server to return error for /failure endpoint
    Mock::given(method("GET"))
        .and(path("/failure"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Server error"))
        .mount(&mock_server)
        .await;
    
    println!("Mock server running at {}", mock_server.uri());
    
    // Create our HTTP service
    let http_service = HttpService::new(mock_server.uri());
    
    // Wrap it in a circuit breaker: opens if ≥50% of last 3 calls failed,
    // stays open 2s, then allows 1 trial in half-open.
    let breaker_layer = circuit_breaker_builder::<String, Error>()
        .failure_rate_threshold(0.5)
        .sliding_window_size(3) // of the past three calls fail
        .wait_duration_in_open(Duration::from_secs(2)) // open for 2 seconds
        .permitted_calls_in_half_open(1) // before allowing 1 call
        .name("http-circuit")
        .build();
    
    // Create a service with our HTTP service and the circuit breaker layer
    let mut svc = ServiceBuilder::new()
        .layer(breaker_layer)
        .service(http_service);
    
    // The circuit starts out closed
    println!("Circuit state (should be closed): {:?}", svc.state().await);
    
    // First successful call
    println!("Making a successful request...");
    match svc.call(true).await {
        Ok(response) => println!("Success response: {}", response),
        Err(e) => println!("Error: {}", e),
    }
    
    // Make three failing calls to trigger the circuit breaker
    println!("\nMaking three failing requests to trigger circuit breaker...");
    for i in 1..=3 {
        match svc.call(false).await {
            Ok(response) => println!("Call {} response: {}", i, response),
            Err(e) => println!("Call {} error: {}", i, e),
        }
    }
    
    // Check circuit state (should be open now)
    println!("\nCircuit state (should be open): {:?}", svc.state().await);
    
    // Try another call while circuit is open (should fail fast)
    println!("\nTrying a call while circuit is open (should fail fast)...");
    match svc.call(true).await {
        Ok(response) => println!("Response: {}", response),
        Err(e) => println!("Error (expected): {}", e),
    }
    
    // Wait for the circuit to transition to half-open
    println!("\nWaiting for circuit to transition to half-open...");
    sleep(Duration::from_secs(2)).await;
    
    // Make a successful call in half-open state
    println!("\nMaking a successful call in half-open state...");
    match svc.call(true).await {
        Ok(response) => println!("Success response: {}", response),
        Err(e) => println!("Error: {}", e),
    }
    
    // Check circuit state (should be closed again)
    println!("\nCircuit state (should be closed again): {:?}", svc.state().await);
    
    // Make more successful calls
    println!("\nMaking more successful calls...");
    for i in 1..=2 {
        match svc.call(true).await {
            Ok(response) => println!("Call {} response: {}", i, response),
            Err(e) => println!("Call {} error: {}", i, e),
        }
    }
    
    // Final circuit state
    println!("\nFinal circuit state: {:?}", svc.state().await);
}