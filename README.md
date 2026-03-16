# Frutiger Drop

**Frutiger Drop** is a production-ready, multi-language project with a Rust core library and official bindings for:

- JavaScript (WASM via `wasm-pack`)
- Python (pyo3 + `maturin`)
- C# (.NET wrapper via `DllImport`)

## Core (Rust)

The core library (`core/`) implements real image math on RGBA8 buffers:

- `apply_blur(input: &[u8], width: u32, height: u32) -> Vec<u8>` (separable box blur)
- `apply_tint(rgba: &[u8], tint: (u8,u8,u8,u8)) -> Vec<u8>`
- `composite_layers(bottom_rgba: &[u8], top_rgba: &[u8]) -> Vec<u8>` (source-over alpha compositing)

## JavaScript (WASM)

Build the WASM package:

```bash
cd bindings/js/wasm
wasm-pack build --release --mode no-install
```

Use the wrapper:

```js
import { init, applyBlur } from "@frutiger-drop/js";
await init();
const out = await applyBlur(inputRgbaBytes, width, height);
```

## Python (maturin)

```bash
cd bindings/python
pip install maturin
maturin build --release
```

## .NET

Build the wrapper:

```bash
cd bindings/dotnet
dotnet build -c Release
```

At runtime, ensure the Rust native library built from `core/` is discoverable by the OS dynamic loader.

