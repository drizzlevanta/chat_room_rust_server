# Chat Room

A real-time chat room application built with Rust, featuring GraphQL and REST APIs, WebSocket support for real-time messaging.

## 🏗️ Architecture

This project follows a monorepo structure using Cargo workspaces, organized into applications and libraries:

### Applications

- **main_server** - The main server application entry point

### Libraries

#### API Layer

- **api-graphql** - GraphQL API implementation
- **api-rest** - REST API implementation

#### Core Business Logic

- **chat_room** - Core chat room functionality (chats, rooms, users, state management)
- **domain** - Domain models (Message, Room, User)
- **service** - Business services (RoomService, UserService)

#### Data Layer

- **db/entity** - Database entity definitions (using SeaORM)
- **db/migration** - Database migrations
- **db/seeder** - Database seeding utilities

#### Infrastructure

- **server** - Server implementation with async-graphql and Axum

## 🚀 Features

- Real-time messaging with WebSocket support
- GraphQL API with subscriptions
- REST API endpoints
- User management
- Chat room management
- Event-driven architecture with EventBus
- Database migrations and seeding
- SQLite database support

## 📋 Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)

## ⚙️ Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd chat_room
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

Run the main server:

```bash
cargo run --bin main_server
```

The server will:

1. Connect to the database
2. Run migrations automatically
3. Start the GraphQL and REST API servers

## 🧪 Development

### Using Bacon (Optional)

This project includes a `bacon.toml` configuration for continuous checking during development. If you have [bacon](https://github.com/Canop/bacon) installed:

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

## 📚 GraphQL Playground

Once the server is running, you can access the GraphQL playground at:

```
http://localhost:<port>/graphql
```

The playground allows you to:

- Explore the GraphQL schema
- Test queries and mutations
- Subscribe to real-time events

## 🏛️ Project Structure

```
chat_room/
├── apps/
│   └── main_server/          # Main application entry point
├── libs/
│   ├── api/
│   │   ├── api-graphql/      # GraphQL API layer
│   │   └── api-rest/         # REST API layer
│   ├── chat_room/            # Core chat functionality
│   │   ├── chats/
│   │   ├── rooms/
│   │   ├── services/
│   │   ├── shared/
│   │   └── users/
│   ├── db/
│   │   ├── entity/           # Database entities
│   │   ├── migration/        # Database migrations
│   │   └── seeder/           # Database seeders
│   ├── domain/               # Domain models
│   ├── server/               # Server infrastructure
│   └── service/              # Business services
└── target/                   # Build artifacts
```

## 🛠️ Technology Stack

- **Framework**: [Axum](https://github.com/tokio-rs/axum) - Web framework
- **GraphQL**: [async-graphql](https://github.com/async-graphql/async-graphql) - GraphQL server library
- **ORM**: [SeaORM](https://www.sea-ql.org/SeaORM/) - Database ORM
- **Database**: SQLite
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **Environment**: [dotenvy](https://github.com/allan2/dotenvy) - Environment variable management
