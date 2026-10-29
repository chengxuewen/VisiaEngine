# Vendored assets

| File | Package | Version | Source | License | Modified |
|---|---|---|---|---|---|
| `cytoscape.min.js` | [cytoscape](https://www.npmjs.com/package/cytoscape) | 3.34.4 | npm tarball (`npm pack cytoscape`), `dist/cytoscape.min.js` | MIT (`CYTOSCAPE-LICENSE.txt`) | no |

Vendored single-file libraries keep the docs site offline-capable: the repository
ships no CDN references (same convention as `examples/wasm/common.js`).

Upgrade procedure: `npm pack cytoscape@<version>`, copy `dist/cytoscape.min.js` and
`LICENSE` here, bump the row above.
