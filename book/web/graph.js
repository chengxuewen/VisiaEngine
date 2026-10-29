// Interactive call/dependency graph for the docs site (S2).
// Data: ../graph.json, exported by scripts/export-graph.mjs (short keys: i,k,n,f,l).
// All labels are English so gate-i18n does not need a whitelist entry.

const EDGE_LABELS = ["calls", "imports", "contains", "instantiates"];
const KIND_COLOR = {
  function: "#2a6fdb", method: "#2a6fdb",
  struct: "#d9822b", class: "#d9822b", trait: "#d9822b", enum: "#d9822b", type_alias: "#d9822b",
  file: "#6b7280", constant: "#6b7280",
};
const REPO = "https://gitee.com/chengxuewen/VisiaEngine";

let graph = null;      // { nodes, edges, stats }
let cy = null;
let adj = null;        // adjacency over "calls" edges
let selected = -1;

const $ = (id) => document.getElementById(id);

function buildAdjacency() {
  adj = new Map(graph.nodes.map((n) => [n.i, { out: [], in: [] }]));
  for (const [s, t, kind] of graph.edges) {
    if (EDGE_LABELS[kind] !== "calls") continue;
    adj.get(s)?.out.push(t);
    adj.get(t)?.in.push(s);
  }
}

function neighbourhood(seed, depth) {
  const keep = new Set([seed]);
  let frontier = [seed];
  for (let d = 0; d < depth; d++) {
    const next = [];
    for (const id of frontier) {
      const entry = adj.get(id);
      if (!entry) continue;
      for (const other of [...entry.out, ...entry.in]) {
        if (!keep.has(other)) { keep.add(other); next.push(other); }
      }
    }
    frontier = next;
  }
  return keep;
}

function elements(seed, depth, withDeps) {
  const nodes = graph.nodes;
  const picked = seed >= 0 ? neighbourhood(seed, depth) : new Set();
  const els = [];
  for (const id of picked) {
    const n = nodes[id];
    els.push({ data: { id: String(id), label: n.n, kind: n.k } });
  }
  for (const [s, t, kind] of graph.edges) {
    const name = EDGE_LABELS[kind];
    if (name === "calls" && picked.has(s) && picked.has(t)) {
      els.push({ data: { id: `e${s}-${t}-${kind}`, source: String(s), target: String(t), kind: name } });
    } else if (withDeps && name !== "calls" && picked.has(s) && picked.has(t)) {
      els.push({ data: { id: `e${s}-${t}-${kind}`, source: String(s), target: String(t), kind: name } });
    }
  }
  return els;
}

function render() {
  if (!cy) {
    cy = cytoscape({
      container: $("cy"),
      elements: [],
      style: [
        { selector: "node", style: { label: "data(label)", "font-size": 10, color: "#e5e7eb",
          "background-color": (n) => KIND_COLOR[n.data("kind")] || "#6b7280", width: 14, height: 14 } },
        { selector: "node[?selected]", style: { "border-width": 2, "border-color": "#111" } },
        { selector: "edge", style: { width: 1, "line-color": "#9ca3af", "target-arrow-color": "#9ca3af",
          "target-arrow-shape": "triangle", "curve-style": "bezier", "arrow-scale": .6 } },
        { selector: 'edge[kind = "imports"]', style: { "line-style": "dashed" } },
        { selector: 'edge[kind = "contains"]', style: { "line-style": "dotted" } },
      ],
      layout: { name: "cose", animate: false },
    });
    cy.on("tap", "node", (evt) => { selected = Number(evt.target.id()); draw(); });
  }
  const depth = Number($("depth").value);
  const withDeps = $("showDeps").checked;
  cy.elements().remove();
  if (selected < 0) { return; }
  cy.add(elements(selected, depth, withDeps));
  cy.getElementById(String(selected)).addClass("selected");
  cy.layout({ name: "cose", animate: false, fit: true, padding: 40 }).run();
  showInfo(selected);
}

function showInfo(id) {
  const n = graph.nodes[id];
  if (!n) { $("info").textContent = ""; return; }
  const href = `${REPO}/blob/main/${n.f}#L${n.l}`;
  $("info").innerHTML = `<strong>${escapeHtml(n.n)}</strong><br>${escapeHtml(n.k)}<br>` +
    `<a href="${href}" target="_blank" rel="noopener">${escapeHtml(n.f)}:${n.l}</a>`;
}

function escapeHtml(s) {
  return String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
}

function search(term) {
  const q = term.trim().toLowerCase();
  const hits = [];
  if (q.length < 2) { $("hits").innerHTML = ""; return; }
  for (const n of graph.nodes) {
    if (n.n.toLowerCase().includes(q)) { hits.push(n); if (hits.length >= 20) break; }
  }
  $("hits").innerHTML = hits
    .map((n) => `<li data-i="${n.i}">${escapeHtml(n.n)} <span class="k">${escapeHtml(n.k)}</span></li>`)
    .join("");
  for (const li of $("hits").querySelectorAll("li")) {
    li.addEventListener("click", () => { selected = Number(li.dataset.i); draw(); });
  }
}

function draw() {
  render();
  document.querySelectorAll("#hits li").forEach((li) =>
    li.classList.toggle("active", Number(li.dataset.i) === selected));
}

fetch("../graph.json")
  .then((r) => { if (!r.ok) throw new Error(`graph.json: HTTP ${r.status}`); return r.json(); })
  .then((data) => {
    graph = data;
    buildAdjacency();
    $("stats").textContent = `indexed ${data.generated_at.replace("T", " ").slice(0, 19)} UTC · ` +
      `${data.stats.nodes} nodes / ${data.stats.edges} edges`;
    $("q").addEventListener("input", (e) => search(e.target.value));
    $("depth").addEventListener("input", (e) => { $("depthVal").textContent = e.target.value; if (selected >= 0) draw(); });
    $("showDeps").addEventListener("change", () => { if (selected >= 0) draw(); });
    // Default: the most connected function, so the page is never empty.
    let best = -1, bestDeg = -1;
    for (const [id, entry] of adj) {
      const deg = entry.out.length + entry.in.length;
      if (deg > bestDeg) { best = id; bestDeg = deg; }
    }
    if (best >= 0) { selected = best; draw(); }
  })
  .catch((err) => {
    $("side").insertAdjacentHTML("beforeend",
      `<p style="color:#b91c1c">Cannot load graph.json: ${escapeHtml(err.message)}.<br>` +
      `Run <code>pixi run book-build</code> (needs <code>pixi run mcp index</code> first).</p>`);
  });
