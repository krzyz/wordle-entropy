async function init() {
    const wasm = await import(
    './wasm.js'
    );
    await wasm.default();
    await wasm.initThreadPool(navigator.hardwareConcurrency);
};

init();