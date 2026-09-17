// CAPI-09 对账机（web-check 链）：node 载 pkg-node 产物——值对表 + d.ts bigint 编译面。
// 常量单源=Rust（capi ffi pub const → js getter 引用）；此处验证"跨语言面不漂"。
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const root = new URL('..', import.meta.url).pathname; // scripts/.. = 仓根
const pkg = root + 'target/web/pkg-node/';

// 1) 产物存在性（web-build.sh 全件）
for (const f of ['visiaengine_wasm.js', 'visiaengine_wasm_bg.wasm', 'visiaengine_wasm.d.ts']) {
  assert.ok(existsSync(pkg + f), `缺产物 pkg-node/${f}——先跑 web-build.sh`);
}
const mod = require(pkg + 'visiaengine_wasm.js');
const { VisiaEngine } = mod;
const m = { ...mod, abiVersion: () => VisiaEngine.abiVersion() }; // static 挂类 [bindgen CJS 形状]

// 2) 值对表（12 项，C 侧镜像 ffi_spec 常量断言同谱）
assert.strictEqual(m.abiVersion(), 0x00010005);
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
assert.match(dts, /static\s+fromCanvas[^;]*Promise/, 'fromCanvas 必须 async 工厂(Promise) [FFI-R:v13-FEAS-2]');
// B1 数据带镜像方法在场（CAPI-13..16 双面单源）
for (const fn of ['setEntityVisible', 'entityVisible', 'addMesh', 'removeEntity', 'setEventCallback', 'addPoints', 'loadPclBytes']) {
  assert.match(dts, new RegExp(`\\b${fn}\\(`), `d.ts 缺 B1 镜像方法 ${fn}`);
}

console.log('MIRROR 3/3');
