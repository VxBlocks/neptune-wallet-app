# vxb neptune wallet

vxb neptune wallet is a cross-platform wallet for [neptunecash](https://github.com/Neptune-Crypto/neptune-core).

## development

### prerequisites

1. System Dependencies
    - Linux
    - macOS Catalina (10.15) and later
    - Windows 7 and later
2. Rust

3. Node.js

### dependencies

refer to [tauri](https://tauri.app/start/prerequisites)

### project structure

- `src` frontend
- `src-tauri` backend
  - `config`
  - `logger`
  - `os`
  - `rpc` rpc server for futher use on browser
  - `rpc_client` rpc_client to interact with rpc server (cli)
  - `wallet` wallet core
  - `service` state management
  - `session_store` session store for frontend
  - `cli` cli entrypoint
  - `gui` gui entrypoint
- `leveldb-sys` leveldb without compile c code since we dont use it

### build

Install [Go Task](https://taskfile.dev/)

refer to [taskfile](./taskfile.yml)

```bash
task build
```

NOTE: windows version can only be built on linux with cargo-xwin.

NOTE: android version can be compiled now, but the frontend is not ready, you can only use android app on tablet or landscape mode.
