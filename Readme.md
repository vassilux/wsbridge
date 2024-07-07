# WebSocket to TCP Bridge

This project is a proof of concept (POC) and learning exercise aimed at bridging communication between a TCP server and WebSocket clients. The goal is to establish a reliable connection between a WebSocket client and a TCP server, allowing seamless data transfer between the two protocols.

## Project Overview

The project demonstrates how to:

- Establish a WebSocket server using Actix-Web.
- Connect to a TCP server using Tokio.
- Relay messages between WebSocket clients and a TCP server.
- Implement a basic heart-beating mechanism to maintain the WebSocket connection.

## Project Structure

The project is structured as follows:

- **`src/main.rs`**: Initializes the Actix-Web server and sets up the WebSocket route.
- **`src/ws_connection.rs`**: Handles WebSocket connections and communication with the TCP server.
- **`src/connection_manager.rs`**: Manages active WebSocket connections.
- **`src/tcp_server.rs`**: Simple TCP server to simulate an application server for testing purposes.

## Key Components

### WebSocket Connection (`WsConn`)

`WsConn` is an actor that handles WebSocket connections. It manages the lifecycle of WebSocket clients, processes incoming WebSocket messages, and relays them to the TCP server. It also handles messages from the TCP server and sends them to the WebSocket clients.

### TCP Stream Handler (`TcpStreamHandler`)

`TcpStreamHandler` is an actor that manages the TCP connection. It reads data from the TCP server and sends it to the `WsConn` actor. It also receives messages from `WsConn` and writes them to the TCP server.

### Connection Manager (`ConnectionManager`)

`ConnectionManager` is an actor that keeps track of active WebSocket connections. It helps manage the lifecycle of these connections and facilitates communication between different actors.

## Running the Project

### Step 1: Set Up the TCP Server

1. **Create a new project for the TCP server**: 
```sh
   git clone https://github.com/vassilux/tcp_server.git
   cd tcp_server
   cargo run 
```

### Step 2: Set Up the WebSocket to TCP Bridge
```sh
   git clone https://github.com/vassilux/wsbridge.git
   cd wsbridge
   cargo run 
```

### Step 3: Test the WebSocket Client

Open http://127.0.0.1:8080 , this url servs index.hml in a web browser to connect to the WebSocket server. 

Send messages from the WebSocket client and verify they are received by the TCP server.


### Learning Objectives

This project aims to:

Understand how to use Actix-Web to manage WebSocket connections.
Learn how to use Tokio for asynchronous TCP communication.
Explore how to bridge different protocols (WebSocket and TCP) using Rust.
Gain hands-on experience with Rust's actor model through Actix.


### Future Work

Potential improvements and extensions include:

Adding error handling and reconnection logic.
Implementing authentication and authorization mechanisms.

Enhancing the message protocol between WebSocket and TCP server.

Adding unit tests and integration tests for better reliability.

Contributing
Feel free to fork this repository and contribute by submitting pull requests. Any improvements, bug fixes, or suggestions are welcome!
