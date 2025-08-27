# Chatger-tui

TUI client implementation for the Penger Protocol

# Requirements

A server implementing the Penger Protocol, get the reference implementation from [Solarium.technology](https://solarium.technology/projects/chatger) or [my elixir implementation](https://github.com/blockdoth/chatger-elixir) 

# Supported architectures
| Architecture   | Build Support | Tested  | Cross compileable | Releases 
|----------------|---------------|---------|-------------------|----------|
| x86_64-linux   | X             | X       | X                 | X        |
| aarch64-linux  | X             |         | X                 | X        |
| x86_64-darwin  | X             |         |                   |          |
| aarch64-darwin | X             | X       |                   |          |
| windows        | X             |         | X                 | X        |

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
Or alternatively for repeated builds (faster because of caching)

```
git clone git@github.com:blockdoth/chatger-tui.git
nix develop
cargo build
```

#### Cross compiling
You can cross compile directly from the flake using the following commands 
```
nix build .#windows-cross
nix build .#x86_64-linux-cross
nix build .#aarch64-linux-cross
```
Be warned, windows packages are not cached in nixpkgs and the entire dependency tree must be build from scratch on first build

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
