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

You will need the Chrome above version 102. 

Download Chrome from [Dev channel](https://www.google.com/chrome/dev/?platform=linux&extra=devchannel).

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

## Models

Stanford Bunny - [Stanford University Computer Graphics Laboratory](https://graphics.stanford.edu/data/3Dscanrep/), [McGuire Computer Graphics Archive](https://casual-effects.com/data/)

Chinese Dragon - [Stanford University Computer Graphics Laboratory](https://graphics.stanford.edu/data/3Dscanrep/), [McGuire Computer Graphics Archive](https://casual-effects.com/data/)
