# tower-circuitbreaker

> **⚠️ DEPRECATED: This crate has moved to [tower-resilience](https://github.com/joshrotenberg/tower-resilience)**
>
> This standalone `tower-circuitbreaker` crate is no longer maintained. All functionality has been integrated into the comprehensive `tower-resilience` project, which provides:
>
> - **Circuit Breaker** (what you're looking for!)
> - **Bulkhead** (resource isolation)
> - **Retry** (with advanced backoff strategies)
> - **Time Limiter** (advanced timeouts)
> - **Cache** (response memoization)
> - **Rate Limiter** (request throttling)
>
> All patterns share a unified event system, consistent APIs, and comprehensive observability support.
>
> ## Migration
>
> **Old (tower-circuitbreaker):**
> ```toml
> [dependencies]
> tower-circuitbreaker = "0.1"
> ```
>
> **New (tower-resilience):**
> ```toml
> [dependencies]
> tower-resilience = "0.2"
> # Or use the individual crate:
> tower-resilience-circuitbreaker = "0.3"
> ```
>
> **API Changes:**
> ```rust
> // Old API
> use tower_circuitbreaker::circuit_breaker_builder;
> let cb = circuit_breaker_builder::<String, ()>()
>     .failure_rate_threshold(0.5)
>     .build();
>
> // New API
> use tower_resilience::circuitbreaker::CircuitBreakerLayer;
> let cb = CircuitBreakerLayer::<String, ()>::builder()
>     .failure_rate_threshold(0.5)
>     .build();
> ```
>
> ## Resources
>
> - **New Repository**: https://github.com/joshrotenberg/tower-resilience
> - **Documentation**: https://docs.rs/tower-resilience
> - **Crates.io**: https://crates.io/crates/tower-resilience
> - **Migration Guide**: See [CHANGELOG](https://github.com/joshrotenberg/tower-resilience/blob/main/CHANGELOG.md)

---

## Original README

For the original documentation, see the [v0.1 documentation](https://docs.rs/tower-circuitbreaker/0.1).
