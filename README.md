# FerrisDB 🦀

![FerrisDB](https://img.shields.io/badge/FerrisDB-educational%20distributed%20database-orange)

Welcome to **FerrisDB**, an educational distributed database built in Rust. This project aims to help you learn about LSM-trees, MVCC, and distributed systems. Inspired by FoundationDB, FerrisDB is not intended for production use but serves as a valuable learning tool.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Architecture](#architecture)
- [Learning Resources](#learning-resources)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)
- [Releases](#releases)

## Introduction

FerrisDB is designed for educational purposes. It provides a hands-on experience with distributed database concepts. You will explore the underlying mechanics of LSM-trees, learn about Multi-Version Concurrency Control (MVCC), and understand how distributed systems operate. 

This project is built in Rust, a language known for its performance and safety. If you are interested in databases or distributed systems, FerrisDB is an excellent starting point.

## Features

- **Educational Focus**: Designed to help you learn about distributed databases.
- **LSM-trees**: Implement LSM-trees for efficient data storage and retrieval.
- **MVCC**: Understand and implement Multi-Version Concurrency Control.
- **Rust**: Written in Rust, ensuring safety and performance.
- **Distributed Systems**: Gain insights into the workings of distributed databases.

## Installation

To get started with FerrisDB, you need to clone the repository and build the project. Follow these steps:

1. Clone the repository:

   ```bash
   git clone https://github.com/tanyatarun18/ferrisdb.git
   cd ferrisdb
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

3. Run the project:

   ```bash
   cargo run
   ```

## Usage

After installation, you can use FerrisDB to experiment with distributed database concepts. The primary entry point is the `main.rs` file, where you can modify and extend the functionality.

To run a specific example or test, navigate to the corresponding directory and execute:

```bash
cargo run --example <example_name>
```

## Architecture

FerrisDB's architecture is based on several key components:

### LSM-trees

Log-Structured Merge-trees (LSM-trees) are used for efficient data storage. They optimize write operations and minimize disk I/O. 

### MVCC

Multi-Version Concurrency Control (MVCC) allows multiple versions of data to coexist. This ensures that read operations do not block write operations, enhancing performance.

### Distributed System Design

FerrisDB employs a distributed architecture, allowing data to be spread across multiple nodes. This design improves scalability and fault tolerance.

## Learning Resources

Here are some resources to deepen your understanding of the concepts used in FerrisDB:

- [LSM-trees Explained](https://example.com/lsm-trees)
- [Understanding MVCC](https://example.com/mvcc)
- [Distributed Systems Overview](https://example.com/distributed-systems)
- [Rust Programming Language](https://www.rust-lang.org/)

## Contributing

We welcome contributions to FerrisDB! If you want to help improve the project, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and commit them.
4. Push to your forked repository.
5. Create a pull request.

Please ensure your code adheres to the project's style guidelines and includes appropriate tests.

## License

FerrisDB is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

## Contact

For questions or feedback, feel free to reach out:

- **Email**: [your-email@example.com](mailto:your-email@example.com)
- **GitHub**: [tanyatarun18](https://github.com/tanyatarun18)

## Releases

You can find the latest releases of FerrisDB [here](https://github.com/tanyatarun18/ferrisdb/releases). Please download and execute the files as needed.

Explore the "Releases" section for updates and new features. 

---

Thank you for your interest in FerrisDB! We hope this project helps you learn and grow in the field of distributed databases.