use uuid::Uuid;

/// HTTP header name used to pass the idempotency key.
pub const IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";

/// Newtype wrapper so the idempotency key has its own distinct type in the
/// GraphQL context and cannot be confused with other `Option<Uuid>` types.
#[derive(Clone, Copy)]
pub struct IdempotencyHeader(pub Option<Uuid>);
