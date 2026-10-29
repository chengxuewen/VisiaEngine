# Call & Dependency Graph

Every edge below comes from the codegraph index (`.codegraph/`), rebuilt with
`pixi run mcp index`. Search for a symbol to see who calls it and what it calls;
click a node for the exact `file:line` and a link into the repository.

<iframe src="graph-view.html" title="call and dependency graph" style="width:100%;height:80vh;border:1px solid #8884;border-radius:6px"></iframe>

If the frame stays empty, the page was built without an index: run
`pixi run mcp index` and then `pixi run book-build` again.
