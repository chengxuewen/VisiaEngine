#!/usr/bin/env node
// mcp-serve.mjs — pinned npm-MCP launcher for the agent single-source (pixi tasks call this).
//
// Why: review #16 forbids `npx -y <pkg>` (floating version, unreproducible). This launcher
// installs <pkg>@<version> ONCE into ./node_modules (gitignored — same dir the legacy
// .opencode/init-mcp-*.mjs wrappers used) and then spawns the bin with the given args.
// Verified package sources (2026-10-28): codegraph = npm `@colbymchenry/codegraph`
// (NOT pypi, contrary to the first plan sketch — corrected per review #17).
//
// Usage:
//   node scripts/mcp-serve.mjs --pkg <npm-pkg> --version <ver> --bin <bin> -- <server-args...>
//   node scripts/mcp-serve.mjs --from scripts/mcp-servers.json --name <server-name>
// The --from/--name form is the SINGLE SOURCE for launcher specs (shared by the pixi
// tasks AND scripts/gate-mcp.py / mcp-bootstrap.py), so the pinned version lives in one file.
//   --pkg     npm package name (e.g. @colbymchenry/codegraph)
//   --version pinned semver (e.g. 1.6.0) — MUST be confirmed via `npm view <pkg> version`
//   --bin     executable name inside node_modules/.bin (e.g. codegraph)
//   --from    path to scripts/mcp-servers.json
//   --name    server key inside that json
//   --        separator; everything after is passed verbatim to the server bin
import { spawn, spawnSync } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = join(__dirname, '..')

const argv = process.argv.slice(2)
const sep = argv.indexOf('--')
const pre = sep === -1 ? argv : argv.slice(0, sep)
const serverArgs = sep === -1 ? [] : argv.slice(sep + 1)

function need(name, val) {
  if (!val) { console.error(`[mcp-serve] missing required --${name}`); process.exit(2) }
  return val
}

let pkg, version, bin, from, name
for (let i = 0; i < pre.length; i++) {
  if (pre[i] === '--pkg') pkg = need('pkg', pre[++i])
  else if (pre[i] === '--version') version = need('version', pre[++i])
  else if (pre[i] === '--bin') bin = need('bin', pre[++i])
  else if (pre[i] === '--from') from = pre[++i]
  else if (pre[i] === '--name') name = pre[++i]
  else { console.error(`[mcp-serve] unknown arg: ${pre[i]}`); process.exit(2) }
}

// single-source mode: load launcher spec from scripts/mcp-servers.json
if (from) {
  const spec = JSON.parse(readFileSync(join(root, from), 'utf8'))[need('name', name)]
  if (!spec) { console.error(`[mcp-serve] no server "${name}" in ${from}`); process.exit(2) }
  pkg = spec.pkg
  version = spec.version
  bin = spec.bin
  if (Array.isArray(spec.args)) serverArgs.push(...spec.args)
}
need('pkg', pkg); need('version', version); need('bin', bin)

const binName = process.platform === 'win32' ? `${bin}.cmd` : bin
const binPath = join(root, 'node_modules', '.bin', binName)

// read the version of <pkg> currently installed in ./node_modules (scoped-aware path)
function installedVersion() {
  const rel = pkg.startsWith('@')
    ? join('node_modules', pkg.split('/')[0], pkg.split('/')[1])
    : join('node_modules', pkg)
  const pp = join(root, rel, 'package.json')
  if (!existsSync(pp)) return null
  try { return JSON.parse(readFileSync(pp, 'utf8')).version } catch { return null }
}

// review #4 (version pinning): never reuse a pre-existing binary at a different version.
// Missing binary -> install (first run); installed-but-different -> fail (pin drift).
const have = installedVersion()
if (!existsSync(binPath) || have !== version) {
  // Pin drift is a real state (someone ran `npm install`, or the pin was bumped
  // without setup): fail loudly instead of reinstalling silently on every start.
  if (have && have !== version) {
    console.error(`[mcp-serve] pin drift: ${pkg} installed @${have}, pin wants @${version}`)
    console.error('[mcp-serve] run: pixi run mcp setup')
    process.exit(1)
  }
  // Missing binary is the first-run path (GUI host first click, clean clone, CI
  // probe) and must keep working. npm writes its summary to stdout, which IS the
  // MCP JSON-RPC channel here, so capture it and re-emit on stderr.
  console.error(`[mcp-serve] installing ${pkg}@${version} (into ./node_modules, gitignored) ...`)
  // Arguments are passed as an array (no shell), so a package name or version
  // from the manifest can never turn into shell syntax. Windows resolves npm
  // through npm.cmd, hence the explicit command name.
  const npm = process.platform === 'win32' ? 'npm.cmd' : 'npm'
  const res = spawnSync(npm, ['install', '-D', '--save-exact', `${pkg}@${version}`], {
    cwd: root,
    stdio: ['ignore', 'pipe', 'inherit'],
  })
  if (res.stdout && res.stdout.length) process.stderr.write(res.stdout)
  if (res.error) {
    console.error('[mcp-serve] install failed:', res.error.message)
    process.exit(1)
  }
  if (res.status !== 0) process.exit(res.status ?? 1)
}

const child = spawn(binPath, serverArgs, { stdio: 'inherit' })
child.on('error', (err) => { console.error(`[mcp-serve] failed to start ${bin}:`, err.message); process.exit(1) })
child.on('exit', (code) => process.exit(code ?? 1))
