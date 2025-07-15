# Chatger-tui

TUI client implementation for the Penger Protocol

# Requirements

A server implementing the Penger Protocol, I used the chatger server from [Link when Solarium Technology is not down]


# Build

### With nix

#### Run without cloning
```
nix run github:blockdoth/chatger-tui
```
#### With cloning

```
git clone git@github.com:blockdoth/chatger-tui.git
nix build
```
Or alternatively (faster because of caching)

```
git clone git@github.com:blockdoth/chatger-tui.git
nix develop
cargo build
```

### Without nix

```
git clone git@github.com:blockdoth/chatger-tui.git
cargo build
```

# Run

Cli options
```
Usage: chatgertui [OPTIONS]

Options:
      --address <ADDRESS>    Server address of chatger server to connect to [default: 0.0.0.0:4348]
      --username <USERNAME>  Username f [default: penger]
      --password <PASSWORD>  Password [default: password]
      --loglevel <LOGLEVEL>  Log level (error, warn, info, debug, trace) [default: DEBUG]
      --auto-login           Automatically login
  -h, --help                 Print help
  -V, --version              Print version
```
Example
```
cargo run -- --address 0.0.0.0:4348 --username penger --password password6 --auto-login

```