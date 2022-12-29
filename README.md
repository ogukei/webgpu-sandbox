# WebGPU Sandbox

DEMO: https://ogukei.github.io/webgpu-sandbox/

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

Since WebGPU Origin Trial: 94 to 109, you will need the specific version of Chrome.

https://chromestatus.com/feature/6213121689518080

1. Install Chrome 108 from the link below.

https://dl.google.com/linux/chrome/deb/pool/main/g/google-chrome-stable/google-chrome-stable_108.0.5359.124-1_amd64.deb

([Note](https://stackoverflow.com/a/59469945))

2. Launch Chrome by running the following command.
```
google-chrome --enable-unsafe-webgpu --enable-features=Vulkan
```
https://stackoverflow.com/a/72495310

3. Launch dev server
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
