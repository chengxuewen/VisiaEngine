// CAPI-09 reconciliation machine (the web-check chain): node loads the pkg-node artifacts -- the value-pair table + the d.ts bigint compile surface.
// Constants have a single source = Rust (capi ffi pub const -> js getter reference); this verifies the "cross-language surface does not drift".
import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const root = new URL('..', import.meta.url).pathname; // scripts/.. = repo root
const pkg = root + 'target/web/pkg-node/';

// 1) Artifact presence (every web-build.sh output)
for (const f of ['visiaengine_wasm.js', 'visiaengine_wasm_bg.wasm', 'visiaengine_wasm.d.ts']) {
  assert.ok(existsSync(pkg + f), `missing artifact pkg-node/${f} -- run web-build.sh first`);
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
  for (const d of ['bindings/wasm/visiaengine-wasm/src', 'bindings/c/visiaengine-capi/src']) {
    const src = newestMtime(root + d);
    assert.ok(src <= built,
      `stale artifact: the sources of ${d} are newer than pkg-node/visiaengine_wasm.d.ts -- run pixi run web-check, then this check`);
  }
}
const mod = require(pkg + 'visiaengine_wasm.js');
const { VisiaEngine } = mod;
const m = { ...mod, abiVersion: () => VisiaEngine.abiVersion() }; // static hung on the class [the bindgen CJS shape]

// 2) Value-pair table (12 items, mirroring the C-side ffi_spec constant asserts, same schema)
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

// 3) d.ts compile surface: the u64 surface must be bigint (silent number truncation = the defense lives here [FFI-R:BS-6])
const dts = readFileSync(pkg + 'visiaengine_wasm.d.ts', 'utf8');
assert.match(dts, /pick\([^)]*\):\s*bigint/, 'pick must return bigint');
assert.match(dts, /entityAt\([^)]*\):\s*bigint/, 'entity_at must return bigint');
assert.match(dts, /addPoints\([^)]*\):\s*bigint/, 'addPoints must return bigint in its 64-bit form');
assert.match(dts, /addLabel\([^)]*\):\s*bigint/, 'addLabel must return bigint in its 64-bit form');
assert.match(dts, /static\s+fromCanvas[^;]*Promise/, 'fromCanvas must be an async factory (Promise) [FFI-R:v13-FEAS-2]');
// The B1 data-band mirror methods are present (the CAPI-13..16 dual-surface single source)
for (const fn of ['setEntityVisible', 'entityVisible', 'addMesh', 'removeEntity', 'setEventCallback', 'addPoints', 'loadPclBytes', 'setClips', 'getClips', 'loadFont', 'addLabel', 'setMap', 'clearMap', 'navigateClick', 'getCameraPose', 'loadMvtDir', 'setTileSourceHttp', 'setTileView', 'loadRasterDir', 'setRasterView', 'loadEnvHdr', 'setNodeTransform', 'getNodeTransform', 'updateEntityAttrF64', 'updateEntityAttrStr', 'updateEntityAttrBool', 'createGroup', 'setParent', 'getParent', 'setGroupOffset', 'getGroupOffset']) {
  assert.match(dts, new RegExp(`\\b${fn}\\(`), `d.ts is missing the B1 mirror method ${fn}`);
}

console.log('MIRROR 3/3');
