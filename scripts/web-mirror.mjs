// CAPI-09 对账机（web-check 链）：node 载 pkg-node 产物——值对表 + d.ts bigint 编译面。
// 常量单源=Rust（capi ffi pub const → js getter 引用）；此处验证"跨语言面不漂"。
import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const root = new URL('..', import.meta.url).pathname; // scripts/.. = 仓根
const pkg = root + 'target/web/pkg-node/';

// 1) 产物存在性（web-build.sh 全件）
for (const f of ['visiaengine_wasm.js', 'visiaengine_wasm_bg.wasm', 'visiaengine_wasm.d.ts']) {
  assert.ok(existsSync(pkg + f), `缺产物 pkg-node/${f}——先跑 web-build.sh`);
}

// 1b) Staleness guard (English-only for new content, C17).
// This script checks BUILD OUTPUT. Without a freshness check it happily passes
// against yesterday's artifact while today's source does not even compile —
// exactly how a wasm32-only breakage (`&mut Vec<f64>` bridge param: accepted by
// the host target, rejected by wasm-bindgen's wasm32 codegen) survived two
// bands, since `pixi run ci` carries no web segment at all. Fail loud instead.
const newestMtime = (dir) => {
  let max = 0;
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = dir + '/' + e.name;
    max = Math.max(max, e.isDirectory() ? newestMtime(p) : statSync(p).mtimeMs);
  }
  return max;
};
{
  const built = statSync(pkg + 'visiaengine_wasm.d.ts').mtimeMs;
  for (const d of ['bindings/js/rust/visiaengine-wasm/src', 'bindings/c/visiaengine-capi/src']) {
    const src = newestMtime(root + d);
    assert.ok(src <= built,
      `产物陈旧：${d} 的源比 pkg-node/visiaengine_wasm.d.ts 新——先跑 pixi run web-check 再跑本检`);
  }
}
const mod = require(pkg + 'visiaengine_wasm.js');
const { VisiaEngine } = mod;
const m = { ...mod, abiVersion: () => VisiaEngine.abiVersion() }; // static 挂类 [bindgen CJS 形状]

// 2) 值对表（12 项，C 侧镜像 ffi_spec 常量断言同谱）
assert.strictEqual(m.abiVersion(), 0x00010010);
assert.strictEqual(m.kind_ptr_move(), 1);
assert.strictEqual(m.kind_ptr_down(), 2);
assert.strictEqual(m.kind_ptr_up(), 3);
assert.strictEqual(m.kind_wheel(), 4);
assert.strictEqual(m.kind_key(), 5);
assert.strictEqual(m.kind_no_such(), 0xffffffff);
assert.strictEqual(m.ve_err_arg(), -1);
assert.strictEqual(m.ve_err_state(), -2);
assert.strictEqual(m.ve_err_io(), -3);
assert.strictEqual(m.ve_err_panic(), -4);
assert.strictEqual(m.ve_err_size(), -5);

// 3) d.ts 编译面：u64 面必须 bigint（number 静默截断=防线在此 [FFI-R:BS-6]）
const dts = readFileSync(pkg + 'visiaengine_wasm.d.ts', 'utf8');
assert.match(dts, /pick\([^)]*\):\s*bigint/, 'pick 返回须 bigint');
assert.match(dts, /entityAt\([^)]*\):\s*bigint/, 'entity_at 返回须 bigint');
assert.match(dts, /addPoints\([^)]*\):\s*bigint/, 'addPoints 位形返回须 bigint');
assert.match(dts, /addLabel\([^)]*\):\s*bigint/, 'addLabel 位形返回须 bigint');
assert.match(dts, /static\s+fromCanvas[^;]*Promise/, 'fromCanvas 必须 async 工厂(Promise) [FFI-R:v13-FEAS-2]');
// B1 数据带镜像方法在场（CAPI-13..16 双面单源）
for (const fn of ['setEntityVisible', 'entityVisible', 'addMesh', 'removeEntity', 'setEventCallback', 'addPoints', 'loadPclBytes', 'setClips', 'getClips', 'loadFont', 'addLabel', 'setMap', 'clearMap', 'navigateClick', 'getCameraPose', 'loadMvtDir', 'setTileSourceHttp', 'setTileView', 'loadRasterDir', 'setRasterView', 'loadEnvHdr', 'setNodeTransform', 'getNodeTransform', 'updateEntityAttrF64', 'updateEntityAttrStr', 'updateEntityAttrBool', 'createGroup', 'setParent', 'getParent', 'setGroupOffset', 'getGroupOffset']) {
  assert.match(dts, new RegExp(`\\b${fn}\\(`), `d.ts 缺 B1 镜像方法 ${fn}`);
}

console.log('MIRROR 3/3');
