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