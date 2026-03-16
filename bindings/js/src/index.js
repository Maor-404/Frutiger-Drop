let modPromise = null;

async function loadWasm() {
  if (!modPromise) {
    modPromise = import("../wasm/pkg/frutiger_drop_wasm.js").then(async (m) => {
      if (typeof m.default === "function") {
        await m.default();
      }
      return m;
    });
  }
  return modPromise;
}

export async function applyBlur(input, width, height) {
  const m = await loadWasm();
  return m.apply_blur(input, width, height);
}

export async function applyTint(rgba, tintRgba) {
  const m = await loadWasm();
  return m.apply_tint(rgba, tintRgba);
}

export async function compositeLayers(bottomRgba, topRgba) {
  const m = await loadWasm();
  return m.composite_layers(bottomRgba, topRgba);
}

export async function init() {
  await loadWasm();
}

