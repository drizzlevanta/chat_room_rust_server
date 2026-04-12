# Feature Enhancement Suggestions

## High Priority (Core Gaps)

### 1. Authentication & Authorization (JWT)
The server has zero auth — any client can create users, delete rooms, or send messages as anyone. Adding JWT middleware would let you enforce ownership (e.g., only message authors can delete their messages) and protect mutations. The idempotency infrastructure already in place fits naturally with signed tokens.

### 2. Message Editing
`MessageEvent::Edited` is already defined in the domain events but never used. The mutation, service method, cache invalidation, and subscription are all missing. This is the closest thing to a "free" feature — the event model is already laid out.

### 3. Rate Limiting
No throttling exists at any layer. A user could spam `sendMessage` in a loop. Tower middleware or a per-user token bucket (Moka is already a dependency) would fix this without much ceremony.

---

## Medium Priority (UX / Product Value)

### 4. Read Receipts
Track which users have seen which messages. Requires a new `message_reads` join table, a `markRead(messageId, userId)` mutation, and a `messageRead(roomId)` subscription. Clients get notified in real-time when others read.

### 5. Message Reactions
Emoji reactions per message. A `reactions` table (`message_id`, `user_id`, `emoji`), mutations `addReaction` / `removeReaction`, and a `reactionAdded(roomId)` subscription. Low complexity, high perceived value.

### 6. @Mentions & Notifications
Parse message content for `@username` patterns in `message_service.rs`, emit a `UserEvent::Mentioned` event, and expose a `userMentioned(userId)` subscription. The event bus architecture already supports this cleanly.

### 7. Room Updates Mutation
There is a `//TODO update room` comment in `libs/api/api-graphql/src/resolvers/room_resolver.rs`. This is a straightforward addition: an `updateRoom(id, name?, capacity?)` mutation wired to the service layer and cache invalidation.

---

## Lower Priority (Polish / Ops)

### 8. Message Search
Full-text search over message content within a room. SQLite supports `FTS5` — a virtual table can be added in a new migration and exposed as `searchMessages(roomId, query)`.

### 9. Unit & Integration Tests
No test coverage exists anywhere. Adding service-layer unit tests (mocking the DB) and at least one integration test (spin up server, run a GraphQL mutation) would significantly improve confidence in the cache invalidation and idempotency logic specifically.

### 10. REST API Completion
`api-rest` is a stub. If non-GraphQL clients (mobile, CLIs) are a target, a REST layer over the same `ServiceContainer` would be straightforward since the service layer is already decoupled from the transport.

### 11. Graceful Shutdown
The server does not drain connections on `SIGTERM`. A `tokio::signal::ctrl_c()` hook with a shutdown channel would prevent in-flight WebSocket subscriptions from being abruptly dropped.

### 12. Externalized Configuration
Cache TTLs, capacities, message length limits, and room capacity limits are hardcoded. Moving these into a config struct loaded from env vars or a TOML file would make the server tunable without recompiling.

---

## Quick Wins

| Feature | Where to Add |
|---|---|
| `updateRoom` mutation | `libs/api/api-graphql/src/resolvers/room_resolver.rs` + `room_service.rs` |
| `editMessage` mutation | `libs/api/api-graphql/src/resolvers/message_resolver.rs` + `message_service.rs` |
| User name length validation | `libs/service/src/user_service.rs` — `create_user` |
| Room name uniqueness | `libs/service/src/room_service.rs` + new migration |
| `last_active` timestamp on rooms | New migration + update on every message sent |
