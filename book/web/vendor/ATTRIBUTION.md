# Vendored assets

| File | Package | Version | Source | License | Modified |
|---|---|---|---|---|---|
| `cytoscape.min.js` | [cytoscape](https://www.npmjs.com/package/cytoscape) | 3.34.4 | npm tarball (`npm pack cytoscape`), `dist/cytoscape.min.js` | MIT (`CYTOSCAPE-LICENSE.txt`) | no |
| `mermaid.min.js` | [mermaid](https://www.npmjs.com/package/mermaid) | 11.17.2 | npm tarball (`npm pack mermaid@11`), `dist/mermaid.min.js` (UMD single file) | MIT (`MERMAID-LICENSE.txt`) | no |

Vendored single-file libraries keep the docs site offline-capable: the repository
ships no CDN references (same convention as `examples/wasm/common.js`).

Size notes: `mermaid.min.js` is ~3.5 MB and is **not** loaded on every page — the
1 KB `../mermaid-init.js` loader injects it only when the page contains
```` ```mermaid ```` fences. mermaid 12 ships a 5.3 MB bundle plus hash-named
chunks (~62 MB with maps), which is why the 11.x UMD build is pinned.

Upgrade procedure: `npm pack <pkg>@<version>`, copy the `dist/<pkg>.min.js` and
`LICENSE` here, bump the row above, rebuild (`pixi run book-build`).
