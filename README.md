# Tarantula

A blockchain based search engine, to combat the dead internet.

+ [tarantula-daemon] Collects the data and connects with other peers.
+ [tarantula-cli] A console command interface to add urls and search.
+ [tarantula-scrapper] Crawles websites, scrappes search information and seends them to the daemon.

## Tarantula Daemon Configuration

```toml
listen = "127.0.0.1:39093"
peer = "127.0.0.1:29092"
folder = "data/"
log = "debug"
connections = ["127.0.0.1:29093"]
```

## Development Requirements

Rust toolchain is needed and CMake and LLVM for compiling bundled C dependencies.

### Windows

```powershell
winget install LLVM.LLVM
winget install Kitware.CMake
winget install Rustlang.Rustup
```

### Linux / Mac OS

+ Rustup
+ CMake
+ LLVM
