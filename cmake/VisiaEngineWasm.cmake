# PolyOrch wasm lane for VisiaEngine (T-G4): the js binding's build engine,
# expressed as PolyOrch public-function calls only (the user-ruled
# no-invented-features constraint). The ci chain (scripts/web-build.sh:
# WEB-SIZE print, node mirror test, wasm-opt accounting) stays the
# measurement lane -- this file adds the IDE/graph face next to it.
#
# Facts pinned by the T-G2 probe round (2026-10-12):
#  * triple injection rides POLYORCH_RUST_CARGO_TARGET (the setup-consumed
#    knob; polyorch_rust_build has no triple keyword);
#  * the D29 naming table normalizes dashes -> underscores for the wasm
#    module (visiaengine_wasm.wasm);
#  * polyorch_rust_test on a wasm triple routes to the HOST (the supplier's
#    probe carve-out): the #[ignore] web_mirror leg stays ignored here and
#    runs in the ci chain where the glue exists;
#  * the glue depends on the wasm-bindgen CLI matching the crate pin
#    (0.2.127, J1) -- the version lock-compare in tool_bootstrap enforces
#    it; a mismatched local install is DEMOTED, not used.
#
# No wasm->capi edge: the wasm32 build compiles the capi path-dep itself,
# for wasm32, inside the dedicated .cargo-target-wasm dir; the host capi
# handles are different-triple artifacts and an edge would force pointless
# host builds (adjudicated in the plan, Momus-reviewed).
#
# FOLDER map (position-mirror): every user-facing button lives under the
# crate's on-disk directory; the face aggregate is FOLDER-less (hidden,
# rlibs precedent).

option(VISIAENGINE_POLYORCH_WASM "Register PolyOrch-driven targets for the wasm/js lane" ON)

function(visiaengine_wasm)
  if(NOT VISIAENGINE_POLYORCH_WASM)
    return()
  endif()
  if(NOT COMMAND polyorch_rust_build OR NOT POLYORCH_RUST_FOUND)
    return()
  endif()

  set(_src "${CMAKE_SOURCE_DIR}/bindings/wasm/visiaengine-wasm")
  set(_manifest "${_src}/Cargo.toml")
  if(NOT EXISTS "${_manifest}")
    message(STATUS "visiaengine wasm lane: no manifest, off")
    return()
  endif()
  if(NOT POLYORCH_RUST_CARGO_TARGET)
    # this function is only reached from the host graph; the wasm lane needs
    # the knob set for its build -- but the knob is global per-directory
    # scope, so the lane registers inside its own directory scope by
    # convention (the root calls us AFTER the crates face, where the knob
    # is still empty). Set + restore around the calls that consume it.
    set(POLYORCH_RUST_CARGO_TARGET wasm32-unknown-unknown)
  endif()

  set(_wasm_cache "${CMAKE_BINARY_DIR}/.cargo-target-wasm")
  set(_fold "bindings/wasm/visiaengine-wasm")

  # one dedicated cache dir for the wasm32 lane (clean is rm -rf BASE_DIR --
  # mixing into a host cache would make any clean wipe host artifacts)
  polyorch_rust_build(TARGET visiaengine-wasm
      PACKAGE visiaengine-wasm CRATE visiaengine-wasm SHARED
      MANIFEST "${_manifest}"
      BASE_DIR "${_wasm_cache}"
      FOLDER "${_fold}")
  polyorch_rust_clean(
      NAME visiaengine-wasm-clean
      BASE_DIR "${_wasm_cache}"
      FOLDER "${_fold}")

  # host-side test button (wasm triples test on the host; the ignored
  # mirror leg stays ignored -- it is the ci chain's job, where glue exists)
  polyorch_rust_test(
      PACKAGE visiaengine-wasm
      NAME    visiaengine-wasm-test
      MANIFEST "${_manifest}"
      BASE_DIR "${_wasm_cache}"
      FOLDER "${_fold}")

  # js glue: web (the demo page) + nodejs (the mirror harness's flavor),
  # typescript decls, the version-locked tool, wasm-opt when present.
  # OUT_DIR lands under the build tree's web/ face for easy serving.
  # Tool resolution: the version lock-compare demands CLI == crate pin
  # (0.2.127). The pixi wasm-spike env carries the pinned pair -- pass it
  # as PREFIX when present (the caller-parameter form of the pixi-env
  # probe order; discovery stays prefix-scoped when PREFIX is given).
  set(_bindgen_prefix "")
  file(GLOB _pixi_wbg "$ENV{HOME}/.pixi/envs/*/bin/wasm-bindgen"
                      "${CMAKE_SOURCE_DIR}/.pixi/envs/*/bin/wasm-bindgen")
  foreach(_wbg ${_pixi_wbg})
    execute_process(COMMAND "${_wbg}" --version
        OUTPUT_VARIABLE _wbg_v ERROR_QUIET OUTPUT_STRIP_TRAILING_WHITESPACE)
    # banner shape "wasm-bindgen 0.2.127": first-triple extraction (the
    # bootstrap's own lock-compare convention), not an exact-string match
    set(_wbg_tri "")
    if(_wbg_v MATCHES "([0-9]+\\.[0-9]+\\.[0-9]+)")
      set(_wbg_tri "${CMAKE_MATCH_1}")
    endif()
    if(_wbg_tri STREQUAL "0.2.127")
      get_filename_component(_bindgen_prefix "${_wbg}" DIRECTORY)
      get_filename_component(_bindgen_prefix "${_bindgen_prefix}" DIRECTORY)
      break()
    endif()
  endforeach()
  set(_bgpkw "")
  if(_bindgen_prefix)
    set(_bgpkw PREFIX "${_bindgen_prefix}")
  endif()
  polyorch_rust_wasm_bindgen(TARGET visiaengine-wasm
      NAME_PREFIX visiaengine-wasm
      FLAVOR web nodejs
      TYPESCRIPT
      OUT_DIR "${CMAKE_BINARY_DIR}/web"
      WASM_BINDGEN_VERSION 0.2.127
      ${_bgpkw}
      WASM_OPT AUTO
      FOLDER "${_fold}")

  # the demo/serve button: assemble web-demo/ (page + park data + the
  # web flavor as pkg/) and serve :8017 -- the old standalone serve lane
  # retired to this graph-internal form (T-H5.4, button visiaengine-wasm-serve): stale glue cannot be
  # served, the DEPENDS edge replays it first.
  # the wall (T-H11): hub + live pages + manifest + bootstrap are globbed.
  # NOTE: the SOURCES list is a CONFIGURE-TIME snapshot (it bakes into
  # fixed copy commands) -- a new E###.html joins the served dir only
  # after a reconfigure; the wall dev loop is file + wall.json entry +
  # reconfigure (CONFIGURE_DEPENDS does not apply to baked commands).
  file(GLOB _wall_pages "${CMAKE_SOURCE_DIR}/examples/wasm/*.html"
                        "${CMAKE_SOURCE_DIR}/examples/wasm/wall.json"
                        "${CMAKE_SOURCE_DIR}/examples/wasm/common.js")
  # the demo/serve button -- HOST-REGISTERED (owner request 2026-10-19): the
  # supplier's polyorch_rust_wasm_serve bakes its Ninja DESC at add_custom_target
  # time and is frozen (C23: no supplier commits while the push backlog is
  # unconfirmed), so the host registers the button itself with the SAME assembly
  # recipe (mirror of the supplier function) -- the DESC carries BOTH urls, and
  # the entry script is dual http+https whenever .certs/ exists (pixi run
  # serve-https-gen once); no certs = the supplier's http-only behavior. ONE
  # code path, no overlay hacks. Retire to the supplier function when it grows
  # TLS_PORT/TLS_CERT/TLS_KEY args.
  set(_serve_dir "${CMAKE_BINARY_DIR}/web-demo")
  file(MAKE_DIRECTORY "${_serve_dir}")
  get_target_property(_pkgroot visiaengine-wasm POLYORCH_WASM_PKG_ROOT)
  if(NOT _pkgroot)
    message(FATAL_ERROR "visiaengine-wasm-serve: no POLYORCH_WASM_PKG_ROOT on visiaengine-wasm")
  endif()
  get_target_property(_jsouts visiaengine-wasm POLYORCH_WASM_JS_OUTPUTS)
  if(NOT _jsouts OR _jsouts STREQUAL "NOTFOUND")
    message(FATAL_ERROR "visiaengine-wasm-serve: no POLYORCH_WASM_JS_OUTPUTS on visiaengine-wasm")
  endif()
  set(_copy_cmds)
  foreach(_f ${_wall_pages}
      "${CMAKE_SOURCE_DIR}/resources/data/park.geojson"
      "${CMAKE_SOURCE_DIR}/resources/data/hierarchy.glb"
      "${CMAKE_SOURCE_DIR}/resources/data/twoprim.glb")
    get_filename_component(_fn "${_f}" NAME)
    list(APPEND _copy_cmds
      COMMAND "${CMAKE_COMMAND}" -E copy_if_different "${_f}" "${_serve_dir}/${_fn}")
  endforeach()
  list(APPEND _copy_cmds
    COMMAND "${CMAKE_COMMAND}" -E chdir "${_serve_dir}"
            "${CMAKE_COMMAND}" -E copy_directory "${_pkgroot}/pkg-web" pkg)

  find_package(Python3 REQUIRED COMPONENTS Interpreter)
  set(_serve_script "${CMAKE_BINARY_DIR}/polyorch-wasm-serve-8017.py")
  if(EXISTS "${CMAKE_SOURCE_DIR}/.certs/server.pem"
      AND EXISTS "${CMAKE_SOURCE_DIR}/.certs/server-key.pem")
    set(_desc "visiaengine-wasm: assemble + serve http://127.0.0.1:8017 + https://<LAN-IP>:8443 (Ctrl-C stops)")
    # string(JOIN): a multi-line quoted arg inside file(WRITE) breaks the
    # CMake parser (measured: Parse error); ';' inside lines is ALSO a CMake
    # list splitter (C21) -- keep both out of the line strings.
    set(_py_lines
      "import http.server, threading, ssl"
      "h = http.server.SimpleHTTPRequestHandler"
      "h.extensions_map['wasm'] = 'application/wasm'"
      "httpd = http.server.ThreadingHTTPServer(('0.0.0.0', 8017), h)"
      "threading.Thread(target=httpd.serve_forever, daemon=True).start()"
      "print('serving: http://localhost:8017  (dev / gate replay)')"
      "ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)"
      "ctx.load_cert_chain('${CMAKE_SOURCE_DIR}/.certs/server.pem', '${CMAKE_SOURCE_DIR}/.certs/server-key.pem')"
      "httpsd = http.server.ThreadingHTTPServer(('0.0.0.0', 8443), h)"
      "httpsd.socket = ctx.wrap_socket(httpsd.socket, server_side=True)"
      "print('serving: https://0.0.0.0:8443  (LAN secure-context WebGPU -- trust the self-signed cert once)')"
      "try:"
      "    httpsd.serve_forever()"
      "except KeyboardInterrupt:"
      "    pass")
    message(STATUS "wasm-serve: dual http+https (certs found -- LAN WebGPU live on :8443)")
  else()
    set(_desc "visiaengine-wasm: assemble + serve http://127.0.0.1:8017 (Ctrl-C stops)")
    set(_py_lines
      "import http.server"
      "h = http.server.SimpleHTTPRequestHandler"
      "h.extensions_map['wasm'] = 'application/wasm'"
      "http.server.test(HandlerClass=h, port=8017)")
    message(STATUS "wasm-serve: http-only (no .certs/ -- pixi run serve-https-gen to enable LAN WebGPU)")
  endif()
  string(JOIN "\n" _py_body ${_py_lines})
  file(WRITE "${_serve_script}" "${_py_body}\n")
  add_custom_target(visiaengine-wasm-serve
    COMMAND "${CMAKE_COMMAND}" -E make_directory "${_serve_dir}"
    ${_copy_cmds}
    COMMAND "${CMAKE_COMMAND}" -E chdir "${_serve_dir}" "${Python3_EXECUTABLE}" "${_serve_script}"
    DEPENDS ${_jsouts}
    USES_TERMINAL
    VERBATIM
    COMMENT "${_desc}")
  if(TARGET visiaengine-wasm-build)
    add_dependencies(visiaengine-wasm-serve visiaengine-wasm-build)
  endif()
  # the button shows its own stage list (T-H11 user catch -- custom-target
  # SOURCES are IDE display only, nothing compiles)
  set_target_properties(visiaengine-wasm-serve PROPERTIES
    SOURCES "${_wall_pages};${CMAKE_SOURCE_DIR}/resources/data/park.geojson;${CMAKE_SOURCE_DIR}/resources/data/hierarchy.glb;${CMAKE_SOURCE_DIR}/resources/data/twoprim.glb"
    FOLDER  "examples/wasm")

  # IDE sources: the WP6 sub-handle is an IMPORTED target (no SOURCES
  # property -- measured), so the mount rides the MEDIATOR (the crates
  # face mounts its custom build nodes; the mediator carries the build
  # inputs and IS the IDE node for this lane)
  polyorch_rust_ide_mount(TARGET visiaengine-wasm-build
      MANIFEST "${_manifest}"
      CRATE visiaengine-wasm
      FOLDER "${_fold}")

endfunction()
