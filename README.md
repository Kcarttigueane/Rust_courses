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