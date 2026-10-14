// VisiaEngine example wall -- shared bootstrap (hub + all E###_*.html).
// Flat sibling layout: everything lands in the served dir next to pkg/ and data.
// Engine face: bindings/wasm/visiaengine-wasm (loadGeoJsonBytes/loadGltfBytes/
// render/input/pick/entityAt/entityCount/viewport/setEntityVisible/
// entityVisible/setClips). Input kinds: 1 move, 2 down, 3 up, 4 wheel.

async function veBoot(canvas) {
  const mod = await import('./pkg/visiaengine_wasm.js');
  await mod.default();
  return await mod.VisiaEngine.fromCanvas(canvas);
}

async function veFile(url) {
  return new Uint8Array(await (await fetch(url)).arrayBuffer());
}

function veOrbit(canvas, eng) {
  let last = [0, 0];
  canvas.addEventListener('pointerdown', e => {
    last = [e.offsetX, e.offsetY]; eng.input(2, last[0], last[1], 0);
    canvas.setPointerCapture(e.pointerId);
  });
  canvas.addEventListener('pointermove', e => { if (e.buttons) eng.input(1, e.offsetX, e.offsetY, 0); });
  canvas.addEventListener('pointerup', e => eng.input(3, e.offsetX, e.offsetY, 0));
  canvas.addEventListener('wheel', e => {
    e.preventDefault(); eng.input(4, e.offsetX, e.offsetY, e.deltaY > 0 ? -1 : 1);
  }, { passive: false });
}

function veLoop(eng, onFrame) {
  (function r() { if (onFrame) onFrame(); eng.render(); requestAnimationFrame(r); })();
}

function vePage(title) {
  document.getElementById('ptitle').textContent = title;
}

function veFail(st) {
  st.textContent = '\u2717 WebGPU \u4e0d\u53ef\u7528\uff08\u9700 Chrome/Edge/Safari 26+ \u6216\u76f8\u5e94 flag\uff09';
}
