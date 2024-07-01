
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
- **`src/messages.rs`**: Defines messages used for communication between actors.

## Key Components

### WebSocket Connection (`WsConn`)

`WsConn` is an actor that handles WebSocket connections. It manages the lifecycle of WebSocket clients, processes incoming WebSocket messages, and relays them to the TCP server. It also handles messages from the TCP server and sends them to the WebSocket clients.

### TCP Stream Handler (`TcpStreamHandler`)

`TcpStreamHandler` is an actor that manages the TCP connection. It reads data from the TCP server and sends it to the `WsConn` actor. It also receives messages from `WsConn` and writes them to the TCP server.

### Connection Manager (`ConnectionManager`)

`ConnectionManager` is an actor that keeps track of active WebSocket connections. It helps manage the lifecycle of these connections and facilitates communication between different actors.

## Usage

To run the project, ensure you have Rust installed and execute the following commands:

1. Clone the repository:
   ```sh
   git clone https://github.com/your-username/websocket-tcp-bridge.git
   cd websocket-tcp-bridge
