# 💬 Terminal Chat App

A simple, real-time terminal-based chat application built with Rust. Features a server-client architecture with colorful output and emoji indicators.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)


## ✨ Features

- 🚀 **Real-time messaging** - Instant message delivery between clients
- 🎨 **Colorful interface** - Syntax-highlighted usernames and messages
- ⚡ **Fast and lightweight** - Built with async Rust (Tokio)
- 🔒 **Multiple clients** - Support for multiple simultaneous connections
- 📡 **Join/Leave notifications** - See when users connect or disconnect
- ⏰ **Timestamps** - Every message includes a formatted timestamp
- 🎯 **Simple commands** - Easy-to-use `/quit` command to exit

## 📋 Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

## 🛠️ Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/chat-app.git
cd chat-app
```

2. Build the project:
```bash
cargo build --release
```

## 🚀 Usage

### Starting the Server

Run the server on the default address (127.0.0.1:8080):
```bash
cargo run -- server
```

Or specify a custom address:
```bash
cargo run -- server --address 0.0.0.0:3000
```

### Connecting as a Client

Connect to the default server:
```bash
cargo run -- client
```

Or connect to a specific server:
```bash
cargo run -- client --address 192.168.1.100:3000
```

### Using the Chat

1. When you connect, you'll be prompted to enter a username
2. Type your message and press Enter to send
3. Type `/quit` to disconnect from the chat

## 📸 Screenshots

### Server View
```
🚀 Server running on 127.0.0.1:8080
📡 New connection from: 127.0.0.1:57472
✅ User 'alice' joined the chat
📨 Message from alice: Hello everyone!
📨 Message from bob: Hi Alice!
👋 User 'alice' is leaving
👋 Client disconnected: alice
```

### Client View
```
✅ Connected to server at 127.0.0.1:8080
Please enter your username: alice
> [15:45:04] ⚡ Welcome to the chat, alice! Type '/quit' to exit
> Hello everyone!
[15:45:10] ➡️ bob joined the chat!
[15:45:15] bob: Hi Alice!
> How are you?
[15:45:20] bob: I'm great, thanks!
> /quit
👋 Disconnected from server
```

## 🏗️ Architecture

### Project Structure
```
chat-app/
├── src/
│   ├── main.rs       # CLI entry point
│   ├── server.rs     # Server implementation
│   ├── client.rs     # Client implementation
│   └── message.rs    # Message data structures
├── Cargo.toml
└── README.md
```

### Components

- **Server** (`server.rs`)
  - Manages client connections
  - Broadcasts messages to all connected clients
  - Tracks active users
  - Uses `broadcast` channels for message distribution

- **Client** (`client.rs`)
  - Handles user input
  - Displays incoming messages with formatting
  - Manages connection to server
  - Provides colorful terminal output

- **Message** (`message.rs`)
  - Defines message structure (username, content, type, timestamp)
  - Handles JSON serialization/deserialization
  - Supports different message types (Text, Join, Leave, System)

## 📦 Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
clap = { version = "4", features = ["derive"] }
colored = "2.0"
chrono = "0.4"
```

## 🎨 Message Types

The application supports four types of messages:

- **Text** - Regular chat messages
- **Join** - User joined notification (➡️)
- **Leave** - User left notification (⬅️)
- **System** - System messages (⚡)

## 🔧 Configuration

### Server Options
- `-a, --address <ADDRESS>` - Server address to bind to (default: `127.0.0.1:8080`)

### Client Options
- `-a, --address <ADDRESS>` - Server address to connect to (default: `127.0.0.1:8080`)

## 🐛 Known Issues

- Username uniqueness is not enforced (multiple users can have the same name)
- No message history for new clients
- No authentication or encryption (suitable for local network only)

## 🚀 Future Improvements

- [ ] Private messaging between users
- [ ] Chat rooms/channels
- [ ] Message history persistence
- [ ] User authentication
- [ ] TLS/SSL encryption
- [ ] File transfer support
- [ ] Typing indicators
- [ ] User list command
- [ ] Message editing/deletion
- [ ] Rich text formatting (markdown)

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 👨‍💻 Author

**lipplopp**

## 🙏 Acknowledgments

- Built with [Tokio](https://tokio.rs/) - Async runtime for Rust
- Terminal colors by [colored](https://github.com/mackwic/colored)
- CLI parsing by [clap](https://github.com/clap-rs/clap)

## 📚 Learning Resources

If you're learning Rust and want to understand this project better:

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Book](https://rust-lang.github.io/async-book/)

---

Made with ❤️ and 🦀 Rust
