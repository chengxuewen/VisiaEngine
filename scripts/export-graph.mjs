#!/usr/bin/env node
// Export the codegraph index as a compact JSON graph for the docs site (S2).
//
// Why direct SQLite instead of the SDK: the SDK exposes per-seed queries
// (getCallGraph(nodeId), getCallers, getCallees) but no whole-graph export, and
// 2k+ round trips are not worth it for a static site. The schema is stable at the
// pinned codegraph version (see scripts/mcp-servers.json).
//
// Output shape (short keys keep the file ~3x smaller):
//   { generated_at, stats, nodes: [{i,k,n,f,l}], edges: [[srcIdx, dstIdx, kindIdx]] }

import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DB = path.join(ROOT, ".codegraph", "codegraph.db");
const OUT = process.argv[2] || path.join(ROOT, "build", "book-src", "graph.json");

// Directories that are noise for a project-architecture view.
const EXCLUDE = [/^\.agents\//, /^\.opencode\//, /^\.omo\//, /^PolyOrch\//, /^target\//, /^build\//, /^www\//, /^book\//];
const NODE_KINDS = ["function", "method", "struct", "trait", "class", "enum", "type_alias", "file", "constant"];
const EDGE_KINDS = ["calls", "imports", "contains", "instantiates"];

if (!existsSync(DB)) {
  console.error("[book-graph] no .codegraph/codegraph.db -- run: pixi run mcp index");
  process.exit(3);
}

const keep = (file) => typeof file === "string" && file.length > 0 && !EXCLUDE.some((re) => re.test(file));
const marks = (n) => NODE_KINDS.map(() => "?").join(",");

const db = new DatabaseSync(DB, { readOnly: true });
const rawNodes = db
  .prepare(`select id, kind, name, file_path, start_line from nodes where kind in (${marks(NODE_KINDS)})`)
  .all(...NODE_KINDS)
  .filter((n) => keep(n.file_path));

const byId = new Map(rawNodes.map((n, i) => [n.id, i]));
const rawEdges = db
  .prepare(`select source, target, kind from edges where kind in (${marks(EDGE_KINDS)})`)
  .all(...EDGE_KINDS)
  .filter((e) => byId.has(e.source) && byId.has(e.target));
db.close();

const payload = {
  generated_at: new Date().toISOString(),
  commit: process.env.BOOK_COMMIT || "",
  stats: { nodes: rawNodes.length, edges: rawEdges.length },
  nodes: rawNodes.map((n) => ({ i: byId.get(n.id), k: n.kind, n: n.name, f: n.file_path, l: n.start_line })),
  edges: rawEdges.map((e) => [byId.get(e.source), byId.get(e.target), EDGE_KINDS.indexOf(e.kind)]),
};

writeFileSync(OUT, JSON.stringify(payload));
const kb = Math.round(readFileSync(OUT).length / 1024);
console.log(`[book-graph] ${payload.stats.nodes} nodes / ${payload.stats.edges} edges -> ${path.relative(ROOT, OUT)} (${kb} KB)`);
