use crate::config::CircuitBreakerConfig;
use crate::{CircuitBreaker, FallbackHandler, SharedFailureClassifier};
use std::sync::Arc;
use std::time::Duration;
use tower::Layer;

/// A Tower Layer that applies circuit breaker behavior to an inner service.
///
/// Wraps an inner service and manages its state according to circuit breaker logic.
#[derive(Clone)]
pub struct CircuitBreakerLayer<Res, Err> {
    config: Arc<CircuitBreakerConfig<Res, Err>>,
}

impl<Res, Err> CircuitBreakerLayer<Res, Err> {
    /// Creates a new `CircuitBreakerLayer` from the given configuration.
    pub(crate) fn new(config: impl Into<Arc<CircuitBreakerConfig<Res, Err>>>) -> Self {
        Self {
            config: config.into(),
        }
    }

    /// Wraps the given service with the circuit breaker middleware.
    pub fn layer<S>(&self, service: S) -> CircuitBreaker<S, Res, Err> {
        CircuitBreaker::new(service, self.config.clone())
    }
}

impl<S, Res, Err> Layer<S> for CircuitBreakerLayer<Res, Err> {
    type Service = CircuitBreaker<S, Res, Err>;

    fn layer(&self, inner: S) -> Self::Service {
        self.layer(inner)
    }
}

/// Builder for configuring and constructing a `CircuitBreakerLayer`.
pub struct CircuitBreakerLayerBuilder<Req, Res, Err> {
    failure_rate_threshold: f64,
    sliding_window_size: usize,
    wait_duration_in_open: Duration,
    permitted_calls_in_half_open: usize,
    failure_classifier: SharedFailureClassifier<Res, Err>,
    fallback_handler: FallbackHandler<Req, Err>,
    minimum_number_of_calls: Option<usize>,
    name: Option<String>,
}

impl<Req, Res, Err> Default for CircuitBreakerLayerBuilder<Req, Res, Err> {
    fn default() -> Self {
        Self {
            failure_rate_threshold: 0.5,
            sliding_window_size: 100,
            wait_duration_in_open: Duration::from_secs(30),
            permitted_calls_in_half_open: 1,
            failure_classifier: Arc::new(|res| res.is_err()),
            fallback_handler: Arc::new(|_| Ok(())),
            minimum_number_of_calls: None,
            name: None,
        }
    }
}

impl<Res, Err> CircuitBreakerLayerBuilder<Res, Err> {
    /// Sets the failure rate threshold at which the circuit will open.
    pub fn failure_rate_threshold(mut self, rate: f64) -> Self {
        self.failure_rate_threshold = rate;
        self
    }

    /// Sets the size of the sliding window for failure rate calculation.
    pub fn sliding_window_size(mut self, size: usize) -> Self {
        self.sliding_window_size = size;
        self
    }

    /// Sets the duration the circuit remains open before transitioning to half-open.
    pub fn wait_duration_in_open(mut self, duration: Duration) -> Self {
        self.wait_duration_in_open = duration;
        self
    }

    /// Sets the number of permitted calls in the half-open state.
    pub fn permitted_calls_in_half_open(mut self, n: usize) -> Self {
        self.permitted_calls_in_half_open = n;
        self
    }

    /// Sets a custom failure classifier function.
    pub fn failure_classifier<F>(mut self, classifier: F) -> Self
    where
        F: Fn(&Result<Res, Err>) -> bool + Send + Sync + 'static,
    {
        self.failure_classifier = Arc::new(classifier);
        self
    }

    /// Sets the minimum number of calls before failure rate is evaluated.
    pub fn minimum_number_of_calls(mut self, n: usize) -> Self {
        self.minimum_number_of_calls = Some(n);
        self
    }

    /// Give this breaker a human-readable name for logs/spans.
    pub fn name<N: Into<String>>(mut self, n: N) -> Self {
        self.name = Some(n.into());
        self
    }

    /// Builds the `CircuitBreakerLayer` with the configured parameters.
    pub fn build(self) -> CircuitBreakerLayer<Res, Err> {
        let config = CircuitBreakerConfig {
            failure_rate_threshold: self.failure_rate_threshold,
            sliding_window_size: self.sliding_window_size,
            wait_duration_in_open: self.wait_duration_in_open,
            permitted_calls_in_half_open: self.permitted_calls_in_half_open,
            failure_classifier: self.failure_classifier,
            minimum_number_of_calls: self
                .minimum_number_of_calls
                .unwrap_or(self.sliding_window_size),
            #[cfg(feature = "tracing")]
            name: self.name,
        };

        CircuitBreakerLayer::new(config)
    }
}
