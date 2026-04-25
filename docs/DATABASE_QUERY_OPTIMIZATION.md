# Database Query Optimization for the Certificate Verification API

> Resolves issue **#464 – Performance: Implement Database Query Optimization**

## Why this repo does not add SQL

The certificate verification API in this repository does not use an active SQL ORM or relational database for live reads. Its backend query path is:

- Soroban read-only `simulateTransaction` contract calls
- optional off-chain indexer / monitoring pipelines that mirror contract state for analytics and dashboards

For issue #464, "database query optimization" is implemented honestly at that query backend layer instead of introducing a fake SQL dependency.

## What was implemented

### Query analysis

`api/src/utils/queryOptimizer.ts` now records bounded per-query performance samples and aggregates for:

- average query time
- p95 / p99 latency
- slow query counts using `QUERY_SLOW_THRESHOLD_MS`
- last observed duration
- backend call counts
- cache hits / misses / stale hits / evictions
- estimated backend load reduction from caching plus request coalescing

The report is exposed at:

- `GET /api/v1/performance/query-optimization`

## Result caching

Read-only API queries now use a TTL-based in-memory cache with bounded size and LRU-style eviction:

- `get_certificate`
- `get_revocation_record`
- `get_student_certificates`
- `get_analytics`

Important behaviors:

- cache keys include the query name plus deterministic serialized contract arguments
- null / not-found results are cached briefly to protect the backend without making stale misses linger
- failed backend calls are never written into cache
- concurrent identical misses are coalesced so only one backend simulation runs

## Soroban RPC connection pooling

The API now maintains a pool of `SorobanRpc.Server` instances rather than a single shared client.

Configuration:

- `STELLAR_RPC_URLS` for a comma-separated pool of RPC endpoints
- `QUERY_POOL_SIZE` for the logical pool size

If multiple URLs are not supplied, the configured RPC URL is repeated to the target pool size. This acts as the connection-pooling analogue for the Soroban RPC backend while preserving the existing contract-read semantics.

## Automated optimization

The optimizer performs conservative best-effort automation driven by observed read traffic:

- adaptive TTL growth for hot, stable queries
- background pre-warming for the analytics singleton query when it is hot and close to expiry

This is intentionally deterministic and bounded. It does not guess at correctness-sensitive contract state, and it only applies to read paths already considered cacheable.

## Index optimization / advisory metadata

Because the repo does not contain a live SQL schema, index optimization is represented as advisory metadata for the real access patterns the API serves.

Current recommended lookup keys and index specs:

| Query | Access pattern | Recommended lookup / index |
|---|---|---|
| `get_certificate` | certificate by certificate ID | direct key lookup on `certificate_id`; off-chain unique index on `certificate_id` |
| `get_revocation_record` | revocation by certificate ID | direct key lookup on `certificate_id`; off-chain index on `revocation.certificate_id` |
| `get_student_certificates` | student certificate list by student address | address-keyed lookup; off-chain composite index on `(student_address, issued_at DESC)` with `certificate_id` as covering field |
| `get_analytics` | aggregate singleton | materialized singleton snapshot such as `analytics/current` |

The performance report includes these advisory specs and whether the current API path is already covered by direct lookup and/or cache.

## Monitoring and dashboards

New Prometheus metrics are exported automatically through the existing `/health/metrics` endpoint, including:

- `cert_api_query_duration_seconds`
- `cert_api_query_cache_events_total`
- `cert_api_query_in_flight_backend`
- `cert_api_query_pool_active_size`
- `cert_api_query_pool_available_size`
- `cert_api_query_cache_entries`
- `cert_api_query_estimated_load_reduction_percent`
- `cert_api_query_average_time_ms`
- `cert_api_query_cache_hit_ratio`

Grafana and alerting were extended to track:

- average query latency against the `<100ms` target
- cache hit ratio and estimated load reduction against the `>=40%` target
- backend query volume split by `cache` vs `backend`
- backend pool utilization and cache entry count

## How success criteria are measured

### Average query time target: `<100ms`

Measured from `cert_api_query_average_time_ms`, which is derived from observed API query durations across cache hits and backend reads.

### Load reduction target: `>=40%`

Measured from `cert_api_query_estimated_load_reduction_percent`.

Formula:

- `(total_requests - backend_request_calls) / total_requests * 100`

This captures both:

- cache hits that avoided backend work
- in-flight request coalescing where many callers share a single backend read

Background pre-warm refreshes are tracked separately so the reduction estimate stays focused on request-triggered load.

## Operational controls

### Query optimization report

`GET /api/v1/performance/query-optimization`

Returns:

- current averages and target compliance
- load reduction and target compliance
- cache totals and hit ratio
- pool state
- slow queries
- top queries
- adaptive optimization recommendations
- index advisory metadata

### Cache invalidation

`POST /api/v1/performance/query-cache/invalidate`

Authenticated endpoint for clearing:

- the full query cache
- a query family
- a key prefix subset

## Environment variables

- `STELLAR_RPC_URLS`
- `QUERY_POOL_SIZE`
- `QUERY_CACHE_MAX_ENTRIES`
- `QUERY_CACHE_DEFAULT_TTL_MS`
- `QUERY_CACHE_ANALYTICS_TTL_MS`
- `QUERY_CACHE_STUDENT_CERTS_TTL_MS`
- `QUERY_CACHE_CERTIFICATE_TTL_MS`
- `QUERY_CACHE_REVOCATION_TTL_MS`
- `QUERY_SLOW_THRESHOLD_MS`
- `QUERY_TARGET_AVG_MS`
- `QUERY_TARGET_LOAD_REDUCTION_PERCENT`

## Scope boundaries

This implementation intentionally does not add:

- a new relational database
- fake SQL migrations
- caching of mutating operations

It keeps the existing contract read behavior intact while adding optimization, monitoring, and operator controls around the real backend query path used by this repository.
