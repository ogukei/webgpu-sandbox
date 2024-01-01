# WebGPU Sandbox

DEMO: https://ogukei.github.io/webgpu-sandbox/?2024

You will need the Chrome version above 102 on the Windows/macOS environment. 

Chrome on Linux is not currently supported since its WebGPU feature is disabled by default. In that case, you will need some experimental flags enabled such as `Unsafe WebGPU` feature via `about:flags`

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

## Models

Stanford Bunny - [Stanford University Computer Graphics Laboratory](https://graphics.stanford.edu/data/3Dscanrep/), [McGuire Computer Graphics Archive](https://casual-effects.com/data/)

Chinese Dragon - [Stanford University Computer Graphics Laboratory](https://graphics.stanford.edu/data/3Dscanrep/), [McGuire Computer Graphics Archive](https://casual-effects.com/data/)
