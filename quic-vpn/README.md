# QUIC VPN

A simple VPN implementation using QUIC as the transport layer, built with Cloudflare's quiche library and Tokio.

## Features

- QUIC-based transport for better performance over unreliable networks
- TUN device support for IP packet tunneling
- Self-signed certificate generation for demos
- Server and client CLI tools
- Built with Rust for safety and performance

## Requirements

- Rust 1.70 or later
- Linux (for TUN device support)
- Root privileges or `CAP_NET_ADMIN` capability (for TUN device creation)

## Building

```bash
cd quic-vpn
cargo build --release
```

The binaries will be available at:
- Server: `target/release/server`
- Client: `target/release/client`

## Usage

### Running the Server

The server needs to run with root privileges to create the TUN device:

```bash
sudo ./target/release/server --listen 0.0.0.0:4433
```

Options:
- `--listen`: Listen address (default: `0.0.0.0:4433`)
- `--cert`: Certificate file path (default: `certs/cert.pem`)
- `--key`: Private key file path (default: `certs/key.pem`)
- `--tun-name`: TUN device name (default: `tun0`)
- `--subnet`: VPN subnet (default: `10.8.0.0/24`)

The server will automatically generate self-signed certificates if they don't exist.

### Running the Client

The client also needs root privileges to create the TUN device:

```bash
sudo ./target/release/client --server localhost:4433
```

Options:
- `--server`: Server URL (required, format: `hostname:port`)
- `--tun-name`: TUN device name (default: `tun1`)
- `--client-ip`: Client IP on VPN (default: `10.8.0.2`)
- `--insecure`: Skip certificate verification for self-signed certs (default: `true`)

### Example: Testing the VPN

1. Start the server:
```bash
sudo ./target/release/server
```

2. In another terminal, start the client:
```bash
sudo ./target/release/client --server 127.0.0.1:4433
```

3. Verify the connection by pinging the server through the VPN:
```bash
ping 10.8.0.1
```

## Architecture

### Server
- Binds to a UDP socket and accepts QUIC connections
- Creates a TUN device (`tun0` by default) with IP `10.8.0.1`
- Forwards packets between QUIC clients and the TUN device
- Supports multiple concurrent clients

### Client
- Connects to the server via QUIC
- Creates a local TUN device (`tun1` by default) with IP `10.8.0.2`
- Routes packets between the local TUN and the QUIC connection
- Uses self-signed certificates in insecure mode for demos

### Protocol
- Uses bidirectional QUIC streams for data transfer
- Packet format:
  - Type byte (1 byte): `0x01` for data, `0x02` for ping, `0x03` for pong
  - Payload: IP packet data (for data packets) or empty (for ping/pong)

## Running Without Root (Advanced)

While a true VPN typically requires root access to create TUN devices, you can use Linux capabilities to grant specific permissions:

```bash
# Build the project
cargo build --release

# Grant CAP_NET_ADMIN capability
sudo setcap cap_net_admin+ep target/release/server
sudo setcap cap_net_admin+ep target/release/client

# Now you can run without sudo
./target/release/server
./target/release/client --server localhost:4433
```

Note: This still requires privileged operations for TUN device creation.

## Logging

Set the `RUST_LOG` environment variable to control logging:

```bash
RUST_LOG=debug sudo ./target/release/server
```

Log levels: `error`, `warn`, `info`, `debug`, `trace`

## Security Considerations

This is a demo implementation and should NOT be used in production without significant hardening:

- Uses self-signed certificates by default (vulnerable to MITM attacks)
- No authentication mechanism for clients
- Simplified packet handling
- No encryption beyond QUIC's built-in TLS 1.3

For production use, you would need:
- Proper certificate management (e.g., Let's Encrypt)
- Client authentication
- Access control and authorization
- Rate limiting
- Comprehensive error handling and recovery
- Security auditing

## Troubleshooting

### "Permission denied" when creating TUN device
Make sure you're running with root privileges or have the `CAP_NET_ADMIN` capability.

### "Address already in use"
Another process is using the port. Change the port with `--listen` or stop the conflicting process.

### Connection timeout
Check firewall rules and ensure the server is reachable from the client.

## License

MIT
