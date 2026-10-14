// VisiaEngine example wall -- shared bootstrap (hub + all E###_*.html).
// Flat sibling layout: everything lands in the served dir next to pkg/ and data.
// Engine face: bindings/wasm/visiaengine-wasm (loadGeoJsonBytes/loadGltfBytes/
// render/input/pick/entityAt/entityCount/viewport/setEntityVisible/
// entityVisible/setClips). Input kinds: 1 move, 2 down, 3 up, 4 wheel.

// 三级分诊: 上下文/接口/适配器 —— 把「黑屏」变成一句可行动的诊断
async function veDiag() {
  if (!window.isSecureContext)
    return '非安全上下文: 当前地址不是 localhost/HTTPS, WebGPU 被浏览器锁掉'
         + ' (别用 file:// 或 LAN IP 直开; 走端口转发用 http://localhost:8017)';
  if (!navigator.gpu)
    return '浏览器无 WebGPU 接口: 升级到 Edge/Chrome 113+; 无 GPU 机器可加'
         + '启动参数 --enable-unsafe-swiftshader 用软件适配器';
  const a = await navigator.gpu.requestAdapter();
  if (!a)
    return '无可用 GPU 适配器: 常见于老核显无 D3D12(Intel 7代及更早)/虚拟机/远程'
         + '桌面会话; edge://gpu 页查 WebGPU 行可定案; 软件兜底同上 flag';
  let i = '';
  try { const x = await a.requestAdapterInfo();
        i = ` (适配器: ${x.vendor||'?'} / ${x.architecture||'?'} / ${x.device||'?'})`; } catch {}
  return '引擎初始化失败' + i;
}
async function veBoot(canvas) {
  const mod = await import('./pkg/visiaengine_wasm.js');
  await mod.default();
  const eng = await mod.VisiaEngine.fromCanvas(canvas);
  if (!eng) throw new Error(await veDiag());
  return eng;
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

function veFail(st, msg) {
  st.textContent = '\u2717 ' + (msg || 'WebGPU unavailable');
}
