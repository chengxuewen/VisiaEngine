// Provenance footer (S8): commit, doc date, graph index time.
//
// Values come from /<lang>/meta.json, written by book-build on every run. Kept in a
// separate tiny file (injected via additional-js) so a stale graph is visible on the
// page instead of being something a reader has to guess.

(() => {
  const self = document.currentScript ||
    Array.from(document.scripts).find((s) => s.src.includes("footer"));
  if (!self) return;

  const langRoot = new URL("../", self.src);   // /<lang>/

  fetch(new URL("meta.json", langRoot), { cache: "no-store" })
    .then((r) => (r.ok ? r.json() : null))
    .then((meta) => {
      if (!meta) return;
      const footer = document.createElement("footer");
      footer.style.cssText =
        "margin:3rem auto 2rem;max-width:46rem;padding:1rem;border-top:1px solid #8884;" +
        "font:12px/1.7 system-ui,sans-serif;opacity:.72;text-align:center";
      const parts = [];
      if (meta.commit) parts.push("commit " + meta.commit);
      if (meta.updated) parts.push("docs " + meta.updated);
      if (meta.indexed) parts.push("graph " + meta.indexed);
      footer.textContent = parts.join("  ·  ");
      document.querySelector("main")?.appendChild(footer) || document.body.appendChild(footer);
    })
    .catch(() => { /* a partial build may have no meta.json; stay silent */ });
})();
