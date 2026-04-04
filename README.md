# Chat Room

A real-time chat room application built with Rust, featuring a GraphQL API with subscriptions over WebSocket.

## 🏗️ Architecture

This project follows a monorepo structure using Cargo workspaces, organized into applications and libraries:

### Applications

- **main_server** - Entry point; delegates to `api-server`

### Libraries

#### API Layer

- **api-graphql** - GraphQL schema, resolvers, types, and request handler
- **api-rest** - REST API (stubbed, not yet implemented)
- **api-server** - Axum server wiring: DB connection, migrations, router composition

#### Core Business Logic

- **service** - Business services (`RoomService`, `UserService`, `MessageService`), `EventBus`, and in-memory `ChatCache`
- **domain** - Domain models (`Room`, `User`, `Message`), events, and pagination types

#### Data Layer

- **db/entity** - SeaORM entity definitions
- **db/migration** - Database migrations (auto-applied on startup)
- **db/seeder** - Database seeding utility

## 🚀 Features

- GraphQL API with queries, mutations, and real-time subscriptions
- WebSocket-based subscriptions for live events
- Idempotency key support for room, user, and message creation
- In-memory caching with [Moka](https://github.com/moka-rs/moka)
- Event-driven architecture via an internal `EventBus`
- Cursor-based pagination for message history
- Typing indicators per room
- User presence tracking (join / leave room)
- User status management (Online, Offline, Away)
- Unique username enforcement
- SQLite database via SeaORM
- Structured logging with `tracing` / `tracing-subscriber`

## 📋 Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)

## ⚙️ Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd chat_room_rust_server
```

2. Create a `.env` file in the root directory:

```env
DATABASE_URL=sqlite://./chat_room.db?mode=rwc
```

3. Build the project:

```bash
cargo build
```

## 🏃 Running the Application

```bash
cargo run --bin main_server
```

The server will connect to the database, run any pending migrations automatically, and start listening. Default address: `http://127.0.0.1:7000`.

### Environment Variables

| Variable       | Required | Default       | Description                                      |
|----------------|----------|---------------|--------------------------------------------------|
| `DATABASE_URL` | Yes      | —             | SQLite connection string, e.g. `sqlite://./chat_room.db?mode=rwc` |
| `HOST`         | No       | `127.0.0.1`   | Bind address                                     |
| `PORT`         | No       | `7000`        | Bind port                                        |
| `RUST_LOG`     | No       | `info`        | Tracing filter, e.g. `debug`, `api_graphql=trace` |

### Seeding the Database

```bash
cargo run --bin seeder
```

## 📚 GraphQL API

### Endpoints

| Path            | Description                         |
|-----------------|-------------------------------------|
| `GET /graphql`  | GraphQL Playground (browser UI)     |
| `POST /graphql` | GraphQL HTTP endpoint               |
| `WS /graphql/ws`| GraphQL WebSocket subscriptions     |

### Idempotency Key

The `createRoom`, `createUser`, and `sendMessage` mutations require an `Idempotency-Key` HTTP header containing a UUID v4. This prevents duplicate entries when a request is retried.

```http
Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000
```

### Queries

| Query | Arguments | Description |
|---|---|---|
| `rooms` | — | List all rooms |
| `room` | `id: UUID` | Get a room by ID |
| `user` | `id: UUID` | Get a user by ID |
| `usersInRoom` | `roomId: UUID` | List users currently in a room |
| `usersByStatus` | `status: UserStatus` | List users with a given status |
| `messages` | `roomId: UUID` | Fetch all messages in a room |
| `messagesPaginated` | `roomId: UUID`, `cursor?: UUID`, `limit: Int = 20` | Cursor-based pagination, newest first |

### Mutations

| Mutation | Arguments | Requires Idempotency-Key | Description |
|---|---|---|---|
| `createRoom` | `input: CreateRoomInput` | Yes | Create a new room |
| `deleteRoom` | `id: UUID` | No | Delete a room |
| `setTyping` | `roomId, userId, isTyping` | No | Broadcast a typing indicator |
| `createUser` | `input: CreateUserInput` | Yes | Create a new user |
| `updateUserStatus` | `input: UpdateUserStatusInput` | No | Change a user's status |
| `deleteUser` | `id: UUID` | No | Delete a user |
| `setPresence` | `roomId, userId, isPresent` | No | Join (`true`) or leave (`false`) a room |
| `sendMessage` | `input: SendMessageInput` | Yes | Send a message to a room |
| `deleteMessage` | `id: UUID` | No | Delete a message |

### Subscriptions

| Subscription | Arguments | Yields | Description |
|---|---|---|---|
| `roomAdded` | — | `Room` | Fires when a new room is created |
| `userTyping` | `roomId: UUID` | `TypingIndicator` | Typing events for a room |
| `userEntered` | `roomId: UUID` | `UserEnteredRoom` | User joins a room |
| `userLeft` | `roomId: UUID` | `UserLeftRoom` | User leaves a room |
| `userStatusChanged` | — | `UserStatusChanged` | Status changes across all users |
| `messageSent` | `roomId: UUID` | `MessageSentEvent` | New message in a room |
| `messageDeleted` | `roomId: UUID` | `UUID` | Deleted message ID in a room |

## 🏛️ Project Structure

```
chat_room_rust_server/
├── apps/
│   └── main_server/          # Binary entry point
├── libs/
│   ├── api/
│   │   ├── api-graphql/      # Schema, resolvers, types, handler
│   │   ├── api-rest/         # REST API (stubbed)
│   │   └── api-server/       # Axum server bootstrap
│   ├── db/
│   │   ├── entity/           # SeaORM entities
│   │   ├── migration/        # Database migrations
│   │   └── seeder/           # Seed data binary
│   ├── domain/               # Domain models and events
│   └── service/              # Business logic, EventBus, cache
└── target/                   # Build artifacts
```

## 🛠️ Technology Stack

- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **GraphQL**: [async-graphql](https://github.com/async-graphql/async-graphql)
- **ORM**: [SeaORM](https://www.sea-ql.org/SeaORM/)
- **Database**: SQLite
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **Caching**: [Moka](https://github.com/moka-rs/moka)
- **Logging**: [tracing](https://github.com/tokio-rs/tracing) + [tracing-subscriber](https://docs.rs/tracing-subscriber)
- **Environment**: [dotenvy](https://github.com/allan2/dotenvy)

## 🧪 Development

### Using Bacon (Optional)

```bash
bacon
```

### Running Tests

```bash
cargo test
```

### Building for Release

```bash
cargo build --release
```
