# Rust_courses

## TP 1:  🏦 Bank Account System - Rust Learning Project

A simple bank account management system that demonstrates core Rust concepts.

## 🎯 Rust Concepts Demonstrated

### **Data Structures**
- **Structs** - Custom data types (`BankAccount`)
- **impl blocks** - Adding methods to structs
- **Vec<T>** - Dynamic arrays for storing accounts

### **Ownership & Borrowing**
- **`&self`** - Immutable borrowing (reading data)
- **`&mut self`** - Mutable borrowing (modifying data)
- **References** - `&Vec<BankAccount>` for function parameters

### **Pattern Matching**
- **`match`** - Powerful pattern matching for menu choices
- **`Option<T>`** - Handling nullable values safely
- **`Result<T, E>`** - Error handling with `parse()`

### **Control Flow**
- **`loop`** - Infinite loops with `break`
- **`if/else`** - Conditional logic
- **Iterator methods** - `.enumerate()`, `.iter()`

### **String Handling**
- **`String`** vs **`&str`** - Owned vs borrowed strings
- **`.to_string()`** - Converting string slices to owned strings
- **`.trim()`** - String manipulation

## TP 2: 🏦 Enhanced Bank Account System - Advanced Rust Features

Extends the bank account system with multiple accounts management and account operations.

### **Rust Concepts Demonstrated**
- **Vector operations** - Managing multiple accounts with `Vec<BankAccount>`
- **Index-based access** - Direct element access with bounds checking
- **Clone trait** - Data duplication with `.clone()`
- **Function parameters** - References vs owned values
- **User input validation** - Number parsing and error handling

## TP 3: 📁 File Manager System - Ownership & Advanced Patterns

A comprehensive file management system demonstrating advanced Rust concepts.

### **Rust Concepts Demonstrated**
- **Enums** - Custom types for operations (`FileOperation`, `OperationResult`)
- **Struct methods** - Implementation blocks with `impl`
- **Ownership & borrowing** - Memory management without garbage collection
- **Error handling** - `Result<T, E>` and `Option<T>` patterns
- **External crates** - `chrono` for date/time handling
- **Pattern matching** - Complex `match` expressions
- **Loops** - `loop`, `while`, and `for` iterations

## TP 4: 📝 Journalisation Server - Asynchronous TCP Server

An asynchronous logging server that accepts TCP connections and logs messages with timestamps.

### **Features**
- **TCP Server** - Listens on port 8080 for incoming connections
- **Multi-client support** - Multiple clients can connect simultaneously
- **Timestamped logging** - All messages are logged with precise timestamps
- **File logging** - Messages saved to `logs/server.log`
- **Interactive commands** - Built-in commands (stats, ping, help, quit)
- **Client management** - Tracks active connections with unique IDs

### **Rust Concepts Demonstrated**
- **Async/await** - Asynchronous programming with Tokio
- **TCP networking** - `TcpListener` and `TcpStream` for network communication
- **Concurrency** - `tokio::spawn` for handling multiple clients
- **Shared state** - `Arc<Mutex<T>>` for thread-safe data sharing
- **Error handling** - Comprehensive error management
- **External crates** - `tokio`, `chrono`, `uuid` for async operations
- **File I/O** - Asynchronous file operations with `tokio::fs`
- **String formatting** - Message formatting with timestamps

### **Usage**
```bash
# Start the server
cargo run --bin journalisation_server

# Test with netcat
nc 127.0.0.1 8080

# Test with the Rust client
cargo run --bin test_client

# Run multi-client test
./test/test_multiple_clients.sh
```

### **Testing**
- **Multi-client testing** - Bash script simulating 3 concurrent clients
- **Command testing** - Built-in commands (stats, ping, help)
- **Log verification** - Check `logs/server.log` for timestamped entries

## TP 7: 🌐 DNS Client/Server System - Network Programming & Protocol Implementation

A complete DNS (Domain Name System) client-server implementation demonstrating network programming and binary protocol handling.

### **Features**
- **DNS Server** - UDP server listening on port 5353 with RFC 1035 compliance
- **DNS Client** - Command-line client with timeout and comparison features
- **Protocol Implementation** - Complete DNS message parsing and serialization
- **Local Database** - Pre-configured domain records (localhost, google.com, etc.)
- **Error Handling** - NXDOMAIN, NOTIMP response codes
- **Performance Testing** - Concurrent client support and response time measurement
- **Public DNS Comparison** - Compare results with Google, Cloudflare, Quad9

### **Rust Concepts Demonstrated**
- **Network Programming** - UDP sockets with `tokio::net::UdpSocket`
- **Binary Protocol** - Byte-level DNS message format implementation
- **Serialization/Deserialization** - Custom encoding/decoding with `byteorder`
- **Async Programming** - Tokio async runtime for concurrent client handling
- **Error Handling** - Custom error types and comprehensive error management
- **Command-line Interface** - `clap` for argument parsing and user interaction
- **Bit Manipulation** - DNS header flags and compression pointer handling
- **Collections** - `HashMap` for DNS record storage and lookup
- **Pattern Matching** - Enum variants for DNS record types and classes
- **Memory Management** - Efficient buffer handling and zero-copy operations

### **Usage**
```bash
# Start the DNS server
cargo run --bin dns_server -- --port 5353 --verbose

# Query with DNS client
cargo run --bin dns_client -- google.com
cargo run --bin dns_client -- localhost --server 127.0.0.1:5353

# Compare with public DNS servers
cargo run --bin dns_client -- google.com --compare-with-public

# Test with different query types
cargo run --bin dns_client -- google.com --query-type A --timeout 3000

# Run comprehensive tests
./test/test_dns.sh
```

### **Testing**
- **Automated test suite** - Complete testing script with performance and concurrency tests
- **Domain resolution** - Tests for predefined domains and NXDOMAIN responses
- **Performance measurement** - Response time analysis and concurrent client testing
- **Protocol validation** - DNS message format compliance and error handling
- **Public DNS comparison** - Verification against real-world DNS servers