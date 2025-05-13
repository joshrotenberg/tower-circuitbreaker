use crate::SharedFailureClassifier;
use std::time::Duration;

pub(crate) struct CircuitBreakerConfig<Res, Err> {
    pub failure_rate_threshold: f64,
    pub sliding_window_size: usize,
    pub wait_duration_in_open: Duration,
    pub permitted_calls_in_half_open: usize,
    pub minimum_number_of_calls: usize,
    pub failure_classifier: SharedFailureClassifier<Res, Err>,
    #[cfg(feature = "tracing")]
    pub name: Option<String>,
}
