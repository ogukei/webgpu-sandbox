# WebGPU Sandbox

DEMO: https://ogukei.github.io/webgpu-sandbox/

You will need Chrome version above 102 such as Canary builds to run WebGPU at the moment.

## Setup

Install node via Volta if you don't have npm installed.

https://volta.sh/

```
# install Volta
curl https://get.volta.sh | bash

# install Node
volta install node
```

Execute npm install.
```
cd <repository-dir>/webgpu-sandbox-web
npm install
```

Install wasm-pack via https://rustwasm.github.io/wasm-pack/

## Build

```
npm run build
```

## Setup to Run

You will need the Chrome above version 102. Once installed, it is required to configure Unsafe WebGPU feature enabled via `about:flags`.

### macOS

```
brew tap homebrew/cask-versions
brew install --cask google-chrome-dev
```

### Linux

Download Chrome from [Dev channel](https://www.google.com/chrome/dev/?platform=linux&extra=devchannel).

```
sudo apt install <path-to>/google-chrome-unstable_current_amd64.deb
```

Make sure you have enabled the `Unsafe WebGPU` feature via `about:flags`.

## Run
```
npm run serve
```

## Misc
Rust Analyzer settings on VSCode settings.json
```
    "rust-analyzer.server.extraEnv": {
        "RUSTFLAGS": "--cfg=web_sys_unstable_apis"
    }
```
