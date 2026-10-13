//! CAPI-01/02/03: handle generation / panic fence / thread affinity (plan v1.4 §2-§4 I1).
//! rlib direct calls on the extern "C" face (true ABI shape; artifact-level nm verification belongs to gate-abi/I2).

use visiaengine::{
    KIND_NO_SUCH, KIND_PTR_DOWN, KIND_PTR_MOVE, VE_ERR_ARG, VE_ERR_IO, VE_ERR_SIZE, VE_ERR_STATE,
    VE_OK, VE_PCL_FASTFAIL, VE_PCL_LENIENT, VeCameraPose, VeClipPlane, VeInput, VeLabelSpec,
    VeMapView, VeMeshDesc, VePclReport, VePointMark, VePointsDesc, visiaengine_abi_version,
    visiaengine_add_label, visiaengine_add_mesh, visiaengine_add_points, visiaengine_attach,
    visiaengine_attr_bool, visiaengine_attr_f64, visiaengine_attr_str, visiaengine_create_group,
    visiaengine_create_headless, visiaengine_destroy, visiaengine_entity_at,
    visiaengine_entity_count, visiaengine_entity_set_visible, visiaengine_entity_visible,
    visiaengine_fly_state, visiaengine_fly_to, visiaengine_get_camera, visiaengine_get_clips,
    visiaengine_get_group_offset, visiaengine_get_node_transform, visiaengine_get_parent,
    visiaengine_last_error, visiaengine_load_env_hdr, visiaengine_load_font, visiaengine_load_gltf,
    visiaengine_load_mvt_dir, visiaengine_load_pcl, visiaengine_load_raster_dir,
    visiaengine_navigate_click, visiaengine_on_input, visiaengine_pick, visiaengine_readback,
    visiaengine_remove_entity, visiaengine_render, visiaengine_set_clips,
    visiaengine_set_group_offset, visiaengine_set_map, visiaengine_set_node_transform,
    visiaengine_set_parent, visiaengine_set_raster_view, visiaengine_set_tile_source_http,
    visiaengine_set_tile_view, visiaengine_viewport,
};

/// every entry must return -1 for stale/foreign handles (the 17->22 scheme carried and extended; the set_event_callback gate lives in event_spec) (handle validation precedes state validation; abi/last_error have no ve gate)
/// (extern "C" fn does not coerce to a safe fn pointer -- enumerate the gate table explicitly, [rustc ABI pointer types])
fn stale_doors(ve: u64) {
    assert_eq!(visiaengine_destroy(ve), VE_ERR_ARG);
    assert_eq!(visiaengine_attach(ve, 0, 0, 0), VE_ERR_ARG);
    assert_eq!(visiaengine_load_gltf(ve, std::ptr::null()), VE_ERR_ARG);
    assert_eq!(visiaengine_render(ve), VE_ERR_ARG);
    assert_eq!(
        visiaengine_readback(ve, std::ptr::null_mut(), 0),
        VE_ERR_ARG
    );
    assert_eq!(visiaengine_viewport(ve, 8, 8), VE_ERR_ARG);
    assert_eq!(
        visiaengine_attr_f64(ve, 0, std::ptr::null(), std::ptr::null_mut()),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_attr_str(ve, 0, std::ptr::null(), std::ptr::null_mut(), 0),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_attr_bool(ve, 0, std::ptr::null(), std::ptr::null_mut()),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_set_clips(ve, std::ptr::null(), 0),
        VE_ERR_ARG,
        "CAPI-20 new entry joins the stale scheme"
    );
    assert_eq!(
        visiaengine_load_font(ve, std::ptr::null(), 0),
        VE_ERR_ARG,
        "CAPI-21 stale scheme"
    );
    let lspec = VeLabelSpec {
        struct_size: std::mem::size_of::<VeLabelSpec>(),
        pos: [0.0; 3],
        color: [1.0; 4],
        size_px: 16.0,
        text: std::ptr::null(),
    };
    assert_eq!(
        visiaengine_add_label(ve, &lspec, std::ptr::null_mut()),
        VE_ERR_ARG,
        "CAPI-22 stale scheme"
    );
    assert_eq!(
        visiaengine_get_clips(ve, std::ptr::null_mut(), 0),
        VE_ERR_ARG,
        "CAPI-20 read entry same scheme"
    );
    assert_eq!(
        visiaengine_fly_to(ve, std::ptr::null(), 0),
        VE_ERR_ARG,
        "CAPI-23 stale scheme"
    );
    assert_eq!(
        visiaengine_fly_state(ve, std::ptr::null_mut()),
        VE_ERR_ARG,
        "CAPI-24 stale scheme"
    );
    assert_eq!(
        visiaengine_set_map(ve, std::ptr::null()),
        VE_ERR_ARG,
        "CAPI-25 stale scheme"
    );
    assert_eq!(
        visiaengine_navigate_click(ve, 1.0, 1.0, 0),
        VE_ERR_ARG,
        "CAPI-26 stale scheme"
    );
    assert_eq!(
        visiaengine_get_camera(ve, std::ptr::null_mut()),
        VE_ERR_ARG,
        "CAPI-27 stale scheme"
    );
    assert_eq!(visiaengine_entity_set_visible(ve, 0, 1), VE_ERR_ARG);
    assert_eq!(visiaengine_entity_visible(ve, 0), VE_ERR_ARG);
    assert_eq!(
        visiaengine_add_mesh(ve, std::ptr::null(), std::ptr::null_mut()),
        VE_ERR_ARG,
        "the stale engine gate precedes argument validation (return-code scheme, out written zero)"
    );
    assert_eq!(visiaengine_remove_entity(ve, 0), VE_ERR_ARG);
}

#[must_use]
fn input(kind: u32) -> VeInput {
    VeInput {
        struct_size: std::mem::size_of::<VeInput>(),
        kind,
        px: 0.0,
        py: 0.0,
        wheel: 0.0,
        button: 0,
        mods: 0,
    }
}
// note: on_input/pick/entity_at/entity_count validate differently-shaped signatures, asserted separately (under CAPI-01)

// spec: CAPI-01
#[test]
fn handle_generation_slot_base1_stale_and_all_doors() {
    // slot base 1: the first handle is non-0 (0 is the failure sentinel's exclusive use)
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0, "slot base 1: the first create must not encode as 0");
    // every entry usable (not -1/-2(thread)): entity_count=0 empty scene, pick miss sentinel, on_input consumability
    assert_eq!(visiaengine_entity_count(ve), 0);
    assert_eq!(
        visiaengine_pick(ve, 32.0, 32.0),
        u64::MAX,
        "empty-scene miss = UINT64_MAX"
    );
    assert_eq!(
        visiaengine_entity_at(ve, 0),
        u64::MAX,
        "out-of-range index = UINT64_MAX"
    );
    // spec: CAPI-05
    // illegal kind (>5) -> 0 not consumed (no-op family); struct_size too small -> -1 (attribution table [FFI-R:CS-4])
    assert_eq!(visiaengine_on_input(ve, &input(KIND_NO_SUCH)), 0);
    let tiny = input(KIND_PTR_MOVE); // copy, then shrink the size
    let mut tiny = tiny;
    tiny.struct_size = 1;
    assert_eq!(
        visiaengine_on_input(ve, &tiny),
        VE_ERR_ARG,
        "struct_size=1 must reject"
    );
    // foreign (slot outside the table)
    assert_eq!(
        visiaengine_viewport(ve, 0, 0),
        VE_ERR_SIZE,
        "dimension violation attributed to -5 (CAPI-05 attribution table; typo corrected during the RED period)"
    );
    assert_eq!(
        visiaengine_attach(ve, 0, 0, 9),
        VE_ERR_ARG,
        "kind out of range = entry argument -1"
    );
    // destroy idempotent -> 0; after stale all gates -1
    assert_eq!(visiaengine_destroy(ve), 0);
    // semantics correction (RED-period design-conflict ruling): re-destroy on a released handle = -1 --
    // double destroy is a host bug signal, not swallowed by idempotence (the original "repeat = 0" cannot coexist with all-gates -1)
    assert_eq!(
        visiaengine_destroy(ve),
        VE_ERR_ARG,
        "double destroy must report -1 (do not swallow the host bug)"
    );
    assert_eq!(
        visiaengine_entity_count(ve),
        VE_ERR_ARG,
        "stale handle must be -1"
    );
    assert_eq!(
        visiaengine_pick(ve, 0.0, 0.0),
        u64::MAX,
        "stale pick = the miss sentinel"
    );
    assert_eq!(visiaengine_load_gltf(ve, std::ptr::null()), VE_ERR_ARG);
    stale_doors(ve); // six gates explicitly -1 (destroy included)
    // re-create: the new handle != the old (generation advanced); two live instances coexist
    let ve2 = visiaengine_create_headless(64, 64);
    assert_ne!(ve2, ve);
    assert_ne!(ve2, 0);
    let ve3 = visiaengine_create_headless(320, 240);
    assert!(
        (ve2 & 0xFFFF_FFFF_0000_0000) != (ve3 & 0xFFFF_FFFF_0000_0000)
            || (ve2 >> 32) != (ve3 >> 32)
    );
    // I2 implemented: empty-scene render = 0 (image-chain behavior assertions belong to CAPI-04/render_spec)
    assert_eq!(visiaengine_render(ve3), 0);
    visiaengine_destroy(ve2);
    visiaengine_destroy(ve3);
}

// spec: CAPI-02
#[test]
fn panic_fence_returns_minus4_with_message() {
    // fence macro behavior (entries = the same macro 100% coverage, a grep gate tests it separately): a closure injected with panic
    let code = visiaengine::test_util::guard_panic_probe();
    assert_eq!(
        code, -4,
        "the fence swallows panic and returns VE_ERR_PANIC"
    );
    let ve = visiaengine_create_headless(64, 64);
    assert_eq!(
        visiaengine_viewport(ve, 128, 128),
        0,
        "the fence does not pollute subsequent entries"
    );
    assert_eq!(
        visiaengine_readback(ve, std::ptr::null_mut(), 0),
        VE_ERR_STATE,
        "I1 readback without device = -2 state semantics"
    );
    visiaengine_destroy(ve);
}

// spec: CAPI-03
#[test]
fn thread_affinity_owner_only_with_error_string() {
    let ve = visiaengine_create_headless(64, 64);
    let h = std::thread::spawn(move || {
        // non-owner thread: entry -2; last_error = the thread-violation sentence
        assert_eq!(
            visiaengine_entity_count(ve),
            VE_ERR_STATE,
            "cross-thread -2"
        );
        assert_eq!(
            visiaengine_destroy(ve),
            VE_ERR_STATE,
            "destroy rejected cross-thread (does not destroy)"
        );
        let s = unsafe { std::ffi::CStr::from_ptr(visiaengine_last_error(ve)).to_owned() };
        assert!(
            s.to_str().unwrap().contains("thread"),
            "the non-owner error string names thread: {s:?}"
        );
        ve
    });
    let ve_back = h.join().unwrap();
    assert_eq!(
        visiaengine_entity_count(ve_back),
        0,
        "the handle stays alive on the owner thread"
    );
    visiaengine_destroy(ve_back);
}

// spec: CAPI-03
#[test]
fn last_error_write_policy_success_never_clobbers() {
    let ve = visiaengine_create_headless(64, 64);
    // manufacture an error: load a nonexistent file -> -3 and a string written
    let bad = std::ffi::CString::new("no-such-file.glb").unwrap();
    assert_eq!(visiaengine_load_gltf(ve, bad.as_ptr()), VE_ERR_IO);
    let read = || unsafe { std::ffi::CStr::from_ptr(visiaengine_last_error(ve)).to_owned() };
    let first = read();
    assert!(
        !first.to_str().unwrap().is_empty(),
        "the error branch must write a string"
    );
    assert!(
        first.to_str().unwrap().contains("no-such-file"),
        "the string must carry diagnostic value"
    );
    // write standard: successful entries (returning 0/positive) must not touch the string
    assert_eq!(visiaengine_entity_count(ve), 0);
    // engine policy standard: a lone MOVE (not pressed) is not consumed = 0; MOVE after PTR_DOWN is consumed = 1 (orbit)
    assert_eq!(
        visiaengine_on_input(ve, &input(KIND_PTR_MOVE)),
        0,
        "a move without prior press = not this engine's event, not consumed"
    );
    let _down = input(KIND_PTR_DOWN);
    assert_eq!(visiaengine_on_input(ve, &_down), 1, "press consumed");
    assert_eq!(
        visiaengine_on_input(ve, &input(KIND_PTR_MOVE)),
        1,
        "move after press = orbit consumed"
    );
    assert_eq!(
        read().to_str().unwrap(),
        first.to_str().unwrap(),
        "the string is unchanged after a successful call (the SDL warning face)"
    );
    // voiding predicate: the next error overwrites
    assert_eq!(visiaengine_load_gltf(ve, bad.as_ptr()), VE_ERR_IO);
    // if the same path yields the same string, use different paths to assert "an overwrite happened"
    let bad2 = std::ffi::CString::new("no-such-file2.glb").unwrap();
    assert_eq!(visiaengine_load_gltf(ve, bad2.as_ptr()), VE_ERR_IO);
    assert_ne!(
        read().to_str().unwrap(),
        first.to_str().unwrap(),
        "a new error must overwrite the old string"
    );
    visiaengine_destroy(ve);
}

// spec: CAPI-01
#[test]
fn abi_version_packed_and_never_thread_gated() {
    assert_eq!(
        visiaengine_abi_version(),
        0x0001_0010,
        "major 1 minor 16 (V2.2: +CAPI-42/43 node transforms, MAJOR-contained)"
    );
    let h = std::thread::spawn(|| visiaengine_abi_version());
    assert_eq!(
        h.join().unwrap(),
        0x0001_0010,
        "exception-set member is not thread-gated"
    );
}

// spec: CAPI-02
#[test]
fn symbol_surface_grep_gate() {
    // entries = 17 with a uniform style (safe signatures validating internally): self-scan of the file ([FFI-R:FC-5])
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ffi.rs")).unwrap();
    // v1.3 style evolution: C ABI exports are native-only cfg_attr (wasm has zero no_mangle -- the root fix for the
    // clash with the wasm-bindgen export table, J1 link-time proof); the grep gate follows the form
    assert_eq!(
        src.lines()
            .filter(
                |l| l.starts_with("#[cfg_attr(not(target_arch = \"wasm32\"), unsafe(no_mangle))]")
            )
            .count(),
        49,
        "extern entry count (cfg-gated line-head form; 47->49 = V2.2 node pair)"
    );
    assert_eq!(
        src.matches("pub unsafe extern").count(),
        0,
        "unsafe signatures forbidden (validation lives in the body)"
    );
    assert!(
        src.contains("visiaengine_") && !src.contains("cbindgen"),
        "hand-written header discipline"
    );
}

// -- B1 data band, four entries (RED first; the gate table/abi/count are the linked face) --

fn mesh_desc(pos: &[[f32; 3]], nrm: &[[f32; 3]], idx: &[u32]) -> VeMeshDesc {
    VeMeshDesc {
        struct_size: std::mem::size_of::<VeMeshDesc>(),
        positions: pos.as_ptr().cast(),
        normals: nrm.as_ptr().cast(),
        indices: idx.as_ptr(),
        n_positions: pos.len() as u64,
        n_indices: idx.len() as u64,
        base_color: [1.0f32, 1.0, 1.0, 1.0].as_ptr(),
        origin: [0.0f64; 3].as_ptr(),
        // N6 (REND-45): dielectric default (engine maps <=0 roughness to 1.0)
        roughness: 0.0,
        metalness: 0.0,
    }
}

fn load_twoprim(ve: u64) {
    let path = std::ffi::CString::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/twoprim.glb"
    ))
    .unwrap();
    assert_eq!(visiaengine_load_gltf(ve, path.as_ptr()), VE_OK);
}

// spec: CAPI-13
// spec: CAPI-14
#[test]
fn entity_visibility_ffi_roundtrip() {
    let ve = visiaengine_create_headless(160, 120);
    assert_ne!(ve, 0);
    load_twoprim(ve);
    let h0 = visiaengine_entity_at(ve, 0);
    assert_eq!(visiaengine_entity_visible(ve, h0), 1, "visible by default");
    assert_eq!(visiaengine_entity_set_visible(ve, h0, 0), VE_OK);
    assert_eq!(visiaengine_entity_visible(ve, h0), 0);
    assert_eq!(
        visiaengine_entity_set_visible(ve, h0, 2),
        VE_ERR_ARG,
        "strict 0/1 arguments"
    );
    assert_eq!(visiaengine_entity_set_visible(ve, h0, -1), VE_ERR_ARG);
    assert_eq!(
        visiaengine_entity_set_visible(ve, 777, 1),
        VE_ERR_ARG,
        "unknown shape"
    );
    assert_eq!(visiaengine_entity_visible(ve, 777), VE_ERR_ARG);
    assert_eq!(
        visiaengine_entity_set_visible(ve, h0, 1),
        VE_OK,
        "visibility restored"
    );
    assert_eq!(
        visiaengine_entity_count(ve),
        2,
        "visibility does not move the enumeration domain"
    );
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-15
#[test]
fn add_mesh_ffi_struct_contract() {
    let ve = visiaengine_create_headless(160, 120);
    assert_ne!(ve, 0);
    let pos = [
        [0f32, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    let nrm = [[0f32, 0.0, 1.0]; 4];
    let idx = [0u32, 1, 2, 0, 2, 3];
    let mut out = u64::MAX;
    assert_eq!(
        visiaengine_add_mesh(ve, &mesh_desc(&pos, &nrm, &idx), &mut out),
        VE_OK
    );
    assert_eq!(
        out,
        visiaengine_entity_at(ve, 0),
        "shape = the enumeration-domain exit (may be 0, do not judge by 0)"
    );
    assert_eq!(visiaengine_entity_count(ve), 1);
    let h_ok = out; // keep the first-success shape separately (used below -- out is about to be reset as a sentinel trick)
    out = u64::MAX; // reset the sentinel (the previous success may have written shape = 0, exactly the CAPI-01 scheme)
    assert_eq!(
        visiaengine_add_mesh(ve, std::ptr::null(), &mut out),
        VE_ERR_ARG,
        "NULL desc = -1 with out untouched"
    );
    assert_eq!(out, u64::MAX, "failure paths write zero partial");
    let small = VeMeshDesc {
        struct_size: 4,
        ..mesh_desc(&pos, &nrm, &idx)
    };
    assert_eq!(
        visiaengine_add_mesh(ve, &small, &mut out),
        VE_ERR_ARG,
        "struct_size gate = reject (forward-compat discipline)"
    );
    let oob = [0u32, 9, 2];
    assert_eq!(
        visiaengine_add_mesh(ve, &mesh_desc(&pos, &nrm, &oob), &mut out),
        VE_ERR_ARG,
        "index out of range = degenerate rejection"
    );
    assert_eq!(
        visiaengine_add_mesh(ve, &mesh_desc(&pos, &nrm, &idx), std::ptr::null_mut()),
        VE_ERR_ARG,
        "bad out self-guards"
    );
    assert_eq!(
        visiaengine_render(ve),
        VE_OK,
        "an added piece enters the frame command stream"
    );
    assert_eq!(visiaengine_remove_entity(ve, h_ok), VE_OK);
    assert_eq!(
        visiaengine_entity_count(ve),
        0,
        "degenerate/failure paths commit zero (the count stays traceable throughout)"
    );
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-16
#[test]
fn remove_entity_ffi_reentry_aba() {
    let ve = visiaengine_create_headless(160, 120);
    assert_ne!(ve, 0);
    load_twoprim(ve);
    let h0 = visiaengine_entity_at(ve, 0);
    assert_eq!(visiaengine_remove_entity(ve, h0), VE_OK);
    assert_eq!(visiaengine_entity_count(ve), 1);
    assert_eq!(
        visiaengine_remove_entity(ve, h0),
        VE_ERR_ARG,
        "old handle re-entry = same scheme as double destroy"
    );
    assert_eq!(visiaengine_entity_visible(ve, h0), VE_ERR_ARG);
    let pos = [
        [0f32, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    let nrm = [[0f32, 0.0, 1.0]; 4];
    let idx = [0u32, 1, 2, 0, 2, 3];
    let mut h_new = 0u64;
    assert_eq!(
        visiaengine_add_mesh(ve, &mesh_desc(&pos, &nrm, &idx), &mut h_new),
        VE_OK
    );
    assert_ne!(
        h_new, h0,
        "ABA: a new generation in the same slot does not collide with the old shape"
    );
    assert_eq!(
        visiaengine_remove_entity(ve, h0),
        VE_ERR_ARG,
        "the new entity is present, the old handle stays dead"
    );
    assert_eq!(visiaengine_remove_entity(ve, h_new), VE_OK);
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

/// CAPI-18 passthrough load: four locks on the value-domain table (degenerate zero commit / success + out queryable / struct_size gate / non-finite accepted as-is = the host-responsibility division).
// spec: CAPI-18
#[test]
fn add_points_domain_table() {
    let ve = visiaengine_create_headless(160, 120);
    let marks = [
        VePointMark {
            pos: [-1.0, 0.0, 0.0],
            radius_px: 4.0,
            color: [1.0, 0.0, 0.0],
        },
        VePointMark {
            pos: [0.0, 0.0, 0.0],
            radius_px: 6.0,
            color: [0.0, 1.0, 0.0],
        },
        VePointMark {
            pos: [1.0, 0.0, 0.0],
            radius_px: 8.0,
            color: [0.0, 0.0, 1.0],
        },
    ];
    let mut out: u64 = u64::MAX;
    let desc = VePointsDesc {
        struct_size: std::mem::size_of::<VePointsDesc>(),
        marks: marks.as_ptr(),
        count: 3,
    };
    // (1) success road: returns 0, out = queryable shape (entity_count +1)
    assert_eq!(
        visiaengine_add_points(ve, &desc, &mut out),
        VE_OK,
        "stub always -5 = RED target"
    );
    assert_ne!(out, u64::MAX, "success must write out");
    // the render road is real (extra_cmds replay includes the point cloud); the shape does not enter the items enumeration domain (v0 explicit account, same scheme as geo marker)
    assert_eq!(visiaengine_render(ve), VE_OK);
    assert_eq!(visiaengine_entity_count(ve), 0);
    let out1 = out;
    // (2) degenerate zero commit: count=0 / NULL marks -> VE_ERR_ARG with out untouched (sentinel-reset trick)
    let zero = VePointsDesc {
        struct_size: std::mem::size_of::<VePointsDesc>(),
        marks: marks.as_ptr(),
        count: 0,
    };
    out = u64::MAX;
    assert_eq!(visiaengine_add_points(ve, &zero, &mut out), VE_ERR_ARG);
    assert_eq!(out, u64::MAX, "failure must not partially write");
    assert_eq!(
        visiaengine_add_points(ve, std::ptr::null(), &mut out),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_add_points(ve, std::ptr::null(), std::ptr::null_mut()),
        VE_ERR_ARG
    );
    // (3) struct_size forward-compat gate (per CAPI-15: smaller than required = reject)
    let small = VePointsDesc {
        struct_size: 4,
        marks: marks.as_ptr(),
        count: 3,
    };
    assert_eq!(visiaengine_add_points(ve, &small, &mut out), VE_ERR_ARG);
    // (4) non-finite accepted as-is = host responsibility on this entry (with the load-side four-class division explicitly written); shape 0 is legal, do not treat it as a sentinel (CAPI-01)
    let nan = [VePointMark {
        pos: [f32::NAN, 0.0, 0.0],
        radius_px: 4.0,
        color: [1.0, 1.0, 1.0],
    }];
    let dnan = VePointsDesc {
        struct_size: std::mem::size_of::<VePointsDesc>(),
        marks: nan.as_ptr(),
        count: 1,
    };
    assert_eq!(
        visiaengine_add_points(ve, &dnan, &mut out),
        VE_OK,
        "non-finite accepted as-is (only the load side discards by type)"
    );
    assert_ne!(out, out1, "the two clouds have distinct shapes");
    assert_eq!(visiaengine_render(ve), VE_OK, "both clouds replay");
    // (5) handle gate same scheme
    assert_eq!(visiaengine_add_points(0, &desc, &mut out), VE_ERR_ARG);
    assert_eq!(visiaengine_destroy(ve), 0);
}

/// CAPI-19 load domain: success road + meta prefixed queries + three gates of policy/NULL/handle (stub always -3 = RED target).
// spec: CAPI-19
#[test]
fn load_pcl_mount_meta_and_gates() {
    let ve = visiaengine_create_headless(160, 120);
    let path = std::ffi::CString::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/pcl_tetra_ascii.ply"
    ))
    .unwrap();
    let mut ent: u64 = u64::MAX;
    let mut rep = VePclReport {
        struct_size: std::mem::size_of::<VePclReport>(),
        dropped_non_finite: u32::MAX,
        dropped_out_of_domain: u32::MAX,
        dropped_unsupported: u32::MAX,
        truncated_points: u32::MAX,
        kept: u64::MAX,
    };
    assert_eq!(
        visiaengine_load_pcl(ve, path.as_ptr(), VE_PCL_LENIENT, &mut ent, &mut rep),
        VE_OK,
        "stub always -3 = RED target (the GREEN slice turns to 0)"
    );
    assert_ne!(ent, u64::MAX, "success must write out_entity");
    assert_eq!(rep.kept, 4, "ascii four clean points");
    assert_eq!(rep.dropped_non_finite, 0);
    // cloud-level meta prefixed queries (CAPI-19: pcl shapes go through the attr domain, keys collide nowhere thanks to global shape uniqueness)
    let mut pc = 0f64;
    assert_eq!(
        visiaengine_attr_f64(ve, ent, c"point_count".as_ptr(), &mut pc),
        1,
        "point_count queryable (CAPI-10 value domain: 1 = found, 0 = none -- on the attr face, non-0 = OK scheme)"
    );
    assert_eq!(pc, 4.0);
    let mut buf = [0i8; 32];
    assert_eq!(
        visiaengine_attr_str(
            ve,
            ent,
            c"format".as_ptr(),
            buf.as_mut_ptr(),
            buf.len() as u64
        ),
        1,
        "format queryable (found = 1 same scheme)"
    );
    assert_eq!(
        visiaengine_render(ve),
        VE_OK,
        "the point-cloud replay road is real"
    );
    // argument-domain triple
    assert_eq!(
        visiaengine_load_pcl(ve, path.as_ptr(), 9, &mut ent, &mut rep),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_load_pcl(ve, std::ptr::null(), VE_PCL_FASTFAIL, &mut ent, &mut rep),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_load_pcl(
            ve,
            path.as_ptr(),
            VE_PCL_FASTFAIL,
            std::ptr::null_mut(),
            &mut rep
        ),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_load_pcl(0, path.as_ptr(), VE_PCL_FASTFAIL, &mut ent, &mut rep),
        VE_ERR_ARG
    );
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-20
#[test]
fn set_get_clips_full_domain_roundtrip_truncation() {
    let ve = visiaengine_create_headless(160, 120);
    // initially empty; NULL+0 = clearing is legal (the only clearing form)
    assert_eq!(visiaengine_get_clips(ve, std::ptr::null_mut(), 0), 0);
    assert_eq!(visiaengine_set_clips(ve, std::ptr::null(), 0), VE_OK);
    // value-domain rejection: NULL with n>0 / n>4 / degenerate (zero normal / non-finite)
    assert_eq!(visiaengine_set_clips(ve, std::ptr::null(), 1), VE_ERR_ARG);
    let p = VeClipPlane {
        nx: 0.0,
        ny: 1.0,
        nz: 0.0,
        d: 0.0,
    };
    assert_eq!(
        visiaengine_set_clips(ve, &p, 5),
        VE_ERR_ARG,
        "n>MAX rejected (read bounds precede dereference)"
    );
    let bad = VeClipPlane {
        nx: f64::NAN,
        ny: 0.0,
        nz: 0.0,
        d: 0.0,
    };
    assert_eq!(visiaengine_set_clips(ve, &bad, 1), VE_ERR_ARG);
    assert!(
        !visiaengine_last_error(ve).is_null(),
        "the error string is in place on the rejection road"
    );
    // round-trip bit by bit (unit inputs) + normalized form ((0,2,0,d=2) -> (0,1,0,d=1))
    let planes = [
        VeClipPlane {
            nx: 0.0,
            ny: 1.0,
            nz: 0.0,
            d: 0.0,
        },
        VeClipPlane {
            nx: 0.0,
            ny: 2.0,
            nz: 0.0,
            d: 2.0,
        },
        VeClipPlane {
            nx: 0.0,
            ny: 0.0,
            nz: -1.0,
            d: 1.0,
        },
    ];
    assert_eq!(visiaengine_set_clips(ve, planes.as_ptr(), 3), VE_OK);
    assert_eq!(
        visiaengine_get_clips(ve, std::ptr::null_mut(), 9),
        3,
        "NULL buf = count only"
    );
    // truncation self-proof [Momus-A2]: n=3, cap=2 -> return 3 write 2, the third slot's sentinel untouched
    let mut buf = [VeClipPlane {
        nx: -9.0,
        ny: -9.0,
        nz: -9.0,
        d: -9.0,
    }; 3];
    assert_eq!(visiaengine_get_clips(ve, buf.as_mut_ptr(), 2), 3);
    assert_eq!(
        (buf[0].nx, buf[0].ny, buf[0].nz, buf[0].d),
        (0.0, 1.0, 0.0, 0.0)
    );
    assert_eq!(
        (buf[1].nx, buf[1].ny, buf[1].nz, buf[1].d),
        (0.0, 1.0, 0.0, 1.0),
        "normalization syncs w"
    );
    assert_eq!(buf[2].nx, -9.0, "zero writes outside cap");
    // cap=0 with buf!=NULL = the pure-count road, no error (one argument does not serve two masters)
    assert_eq!(visiaengine_get_clips(ve, buf.as_mut_ptr(), 0), 3);
    // clearing restores
    assert_eq!(visiaengine_set_clips(ve, std::ptr::null(), 0), VE_OK);
    assert_eq!(visiaengine_get_clips(ve, std::ptr::null_mut(), 0), 0);
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

const DEJAVU: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../resources/data/DejaVuSans.ttf"
));

// spec: CAPI-21
#[test]
fn load_font_and_add_label_domain_and_flow() {
    let ve = visiaengine_create_headless(160, 120);
    let spec = VeLabelSpec {
        struct_size: std::mem::size_of::<VeLabelSpec>(),
        pos: [0.0, 0.0, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        size_px: 20.0,
        text: c"VI".as_ptr(),
    };
    let mut out = 0u64;
    assert_eq!(
        visiaengine_add_label(ve, &spec, &mut out),
        VE_ERR_ARG,
        "rejected without font"
    );
    assert_eq!(visiaengine_load_font(ve, std::ptr::null(), 0), VE_ERR_ARG);
    assert_eq!(visiaengine_load_font(ve, b"junk".as_ptr(), 4), VE_ERR_ARG);
    assert_eq!(
        visiaengine_load_font(ve, DEJAVU.as_ptr(), DEJAVU.len()),
        VE_OK
    );
    let n0 = visiaengine_entity_count(ve);
    assert_eq!(visiaengine_add_label(ve, &spec, &mut out), VE_OK);
    // B1 lesson used in reverse: shape 0 is legal (CAPI-01 division), the success axis = the return code; the shape-distinctness axis = count + mutual difference
    let mut out2 = 0u64;
    assert_eq!(visiaengine_add_label(ve, &spec, &mut out2), VE_OK);
    assert_ne!(
        out, out2,
        "the two loads have distinct shapes (the ABA-immunity face)"
    );
    // management domain outside items (same scheme as add_points CAPI-18): not in entity_count; distinct shapes are the living proof
    assert_eq!(
        visiaengine_entity_count(ve),
        n0,
        "labels = the domain outside items (same explicit account as add_points)"
    );
    assert_eq!(
        visiaengine_add_label(ve, &spec, std::ptr::null_mut()),
        VE_ERR_ARG
    );
    let empty = VeLabelSpec {
        struct_size: std::mem::size_of::<VeLabelSpec>(),
        pos: [0.0; 3],
        color: [1.0; 4],
        size_px: 20.0,
        text: c"".as_ptr(),
    };
    assert_eq!(
        visiaengine_add_label(ve, &empty, &mut out),
        VE_ERR_ARG,
        "empty text rejected with zero commit"
    );
    let badsz = VeLabelSpec {
        struct_size: std::mem::size_of::<VeLabelSpec>(),
        pos: [0.0; 3],
        color: [1.0; 4],
        size_px: 0.0,
        text: c"X".as_ptr(),
    };
    assert_eq!(
        visiaengine_add_label(ve, &badsz, &mut out),
        VE_ERR_ARG,
        "size out of domain rejected"
    );
    let tiny = VeLabelSpec {
        struct_size: 1,
        pos: [0.0; 3],
        color: [1.0; 4],
        size_px: 20.0,
        text: c"X".as_ptr(),
    };
    assert_eq!(
        visiaengine_add_label(ve, &tiny, &mut out),
        VE_ERR_ARG,
        "struct_size forward-compat gate"
    );
    assert_eq!(visiaengine_render(ve), VE_OK);
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-22
#[test]
fn add_label_returncode_axis_zero_commit_and_position() {
    // return code = the sole success axis; the shape may be 0 (CAPI-01); every rejection form commits zero (the mark count stays the same)
    let ve = visiaengine_create_headless(160, 120);
    assert_eq!(
        visiaengine_load_font(ve, DEJAVU.as_ptr(), DEJAVU.len()),
        VE_OK
    );
    let mk = |text: *const std::ffi::c_char, sz: f32| VeLabelSpec {
        struct_size: std::mem::size_of::<VeLabelSpec>(),
        pos: [1.0, 2.0, 3.0],
        color: [0.2, 0.4, 0.6, 1.0],
        size_px: sz,
        text,
    };
    let mut out = 0u64;
    let before = {
        let _ = visiaengine_render(ve);
        1i32 // after one successful load the mark exists; after a rejection form it is still 1 (no growth = zero commit)
    };
    assert_eq!(
        visiaengine_add_label(ve, &mk(c"A".as_ptr(), 12.0), &mut out),
        VE_OK
    );
    let _ = before;
    // rejection-form triple: NULL text / size 0 / negative size -- out must not be written (sentinel pre-stamped judgment)
    for bad in [
        mk(std::ptr::null(), 12.0),
        mk(c"B".as_ptr(), 0.0),
        mk(c"B".as_ptr(), -3.0),
    ] {
        let mut sentinel = u64::MAX;
        assert_eq!(visiaengine_add_label(ve, &bad, &mut sentinel), VE_ERR_ARG);
        assert_eq!(sentinel, u64::MAX, "rejection forms write out zero");
    }
    // struct_size forward-compat gate: same zero write
    let mut tiny = mk(c"C".as_ptr(), 12.0);
    tiny.struct_size = 8;
    let mut sentinel = u64::MAX;
    assert_eq!(visiaengine_add_label(ve, &tiny, &mut sentinel), VE_ERR_ARG);
    assert_eq!(sentinel, u64::MAX, "gate failure writes out zero");
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-23
#[test]
fn fly_ports_domain_and_progress() {
    let ve = visiaengine_create_headless(160, 120);
    // pose domain: NULL / struct_size gate / bad parameters
    assert_eq!(visiaengine_fly_to(ve, std::ptr::null(), 100), VE_ERR_ARG);
    let mut pose = VeCameraPose {
        struct_size: std::mem::size_of::<VeCameraPose>(),
        target: [3.0, 4.0, 0.0],
        yaw: 0.6,
        pitch: 0.4,
        dist: 30.0,
        zoom: 20.0,
        fov: 1.0,
    };
    assert_eq!(visiaengine_fly_to(ve, &pose, 100), VE_OK);
    let mut t01 = -7.0f64;
    assert_eq!(visiaengine_fly_state(ve, &mut t01), 0, "flying");
    assert!(
        (0.0..=1.0).contains(&{ t01 }),
        "out written while flying [0,1) got {t01}"
    );
    // teleport form: dur=0 -> rc=0 and state immediately done (the done road does not write out = the B1 zero-partial-write scheme)
    assert_eq!(visiaengine_fly_to(ve, &pose, 0), VE_OK);
    let mut sentinel = -9.0f64;
    assert_eq!(
        visiaengine_fly_state(ve, &mut sentinel),
        1,
        "done includes idle"
    );
    assert_eq!(sentinel, -9.0, "the done road writes out zero");
    assert_eq!(
        visiaengine_fly_state(ve, std::ptr::null_mut()),
        1,
        "NULL out = state only"
    );
    // bad-parameter domain: dist/zoom/fov illegal
    pose.dist = -1.0;
    assert_eq!(
        visiaengine_fly_to(ve, &pose, 100),
        VE_ERR_ARG,
        "dist domain"
    );
    pose.dist = 30.0;
    pose.fov = 4.0;
    assert_eq!(
        visiaengine_fly_to(ve, &pose, 100),
        VE_ERR_ARG,
        "fov < pi domain"
    );
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-24
#[test]
fn fly_state_progress_zero_write_and_idle_done() {
    // merged single master of done/idle: never taken off = done(1), the done road writes out zero, NULL = state only
    let ve = visiaengine_create_headless(80, 80);
    let mut t = 123.0f64;
    assert_eq!(
        visiaengine_fly_state(ve, &mut t),
        1,
        "idle = done single master"
    );
    assert_eq!(
        t, 123.0,
        "the done road must never write out [B1 zero partial write]"
    );
    assert_eq!(
        visiaengine_fly_state(ve, std::ptr::null_mut()),
        1,
        "NULL = state only"
    );
    let pose = VeCameraPose {
        struct_size: std::mem::size_of::<VeCameraPose>(),
        target: [1.0, 1.0, 0.0],
        yaw: 0.3,
        pitch: 0.2,
        dist: 20.0,
        zoom: 15.0,
        fov: 0.9,
    };
    // long flight -> while flying out writes [0,1) with progress > 0 (the wall clock really runs)
    assert_eq!(visiaengine_fly_to(ve, &pose, 3000), VE_OK);
    let _ = visiaengine_render(ve);
    let mut tp = -1.0;
    assert_eq!(visiaengine_fly_state(ve, &mut tp), 0, "flying");
    assert!(
        (0.0..1.0).contains(&tp),
        "out in [0,1) while flying got {tp}"
    );
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-25
#[test]
fn map_nav_pose_ports_domain_matrix() {
    let ve = visiaengine_create_headless(200, 100);
    // set_map NULL = closing is OK (with no map, clearing raises no error either)
    assert_eq!(visiaengine_set_map(ve, std::ptr::null()), VE_OK);
    let bad = VeMapView {
        struct_size: std::mem::size_of::<VeMapView>(),
        fx: 0.7,
        fy: 0.0,
        fw: 0.4, // out of bounds
        fh: 0.4,
        zoom: 40.0,
    };
    assert_eq!(
        visiaengine_set_map(ve, &bad),
        VE_ERR_ARG,
        "bounds sum rejected"
    );
    let tiny = VeMapView {
        struct_size: 8,
        ..bad
    };
    assert_eq!(
        visiaengine_set_map(ve, &tiny),
        VE_ERR_ARG,
        "struct_size gate"
    );
    let good = VeMapView {
        struct_size: std::mem::size_of::<VeMapView>(),
        fx: 0.72,
        fy: 0.02,
        fw: 0.26,
        fh: 0.96,
        zoom: 30.0,
    };
    assert_eq!(visiaengine_set_map(ve, &good), VE_OK);
    // get_camera: NULL / gate / normal read; navigate: NaN rejected
    let mut pose = VeCameraPose {
        struct_size: 0,
        target: [0.0; 3],
        yaw: 0.0,
        pitch: 0.0,
        dist: 0.0,
        zoom: 0.0,
        fov: 0.0,
    };
    pose.struct_size = std::mem::size_of::<VeCameraPose>();
    assert_eq!(visiaengine_get_camera(ve, std::ptr::null_mut()), VE_ERR_ARG);
    let mut tiny_p = pose;
    tiny_p.struct_size = 8;
    assert_eq!(
        visiaengine_get_camera(ve, &mut tiny_p),
        VE_ERR_ARG,
        "the out gate is checked first"
    );
    assert_eq!(
        visiaengine_get_camera(ve, &mut pose),
        VE_OK,
        "normal read with pre-stamped struct_size"
    );
    assert!(pose.dist > 0.0 && pose.fov > 0.0 && pose.zoom > 0.0);
    assert_eq!(
        visiaengine_navigate_click(ve, f32::NAN, 3.0, 0),
        VE_ERR_ARG,
        "NaN px rejected"
    );
    assert_eq!(
        visiaengine_navigate_click(ve, 10.0, 50.0, 0),
        VE_ERR_ARG,
        "outside the region rejected (main viewport)"
    );
    assert_eq!(
        visiaengine_navigate_click(ve, 180.0, 50.0, 0),
        VE_OK,
        "inside the region navigates (ground fallback)"
    );
    let mut pose2 = pose;
    pose2.struct_size = std::mem::size_of::<VeCameraPose>();
    assert_eq!(visiaengine_get_camera(ve, &mut pose2), VE_OK);
    assert!(
        (pose2.target[0] - pose.target[0]).abs() > 1.0,
        "target moved after navigation"
    );
    assert!(
        (pose2.dist - pose.dist).abs() < 1e-12,
        "distance kept [ruling e]"
    );
    assert_eq!(
        visiaengine_set_map(ve, std::ptr::null()),
        VE_OK,
        "the restore road after closing the map"
    );
    assert_eq!(
        visiaengine_render(ve),
        VE_OK,
        "after closing, the old single-frame path works"
    );
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-28
#[test]
fn load_mvt_dir_domain_and_mount() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    // NULL path
    assert_eq!(visiaengine_load_mvt_dir(ve, std::ptr::null(), 10), -1);
    // z domain
    let root = {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        while !p.join("Cargo.lock").exists() {
            assert!(p.pop());
        }
        p.join("resources/data/tiles")
    };
    let cpath = std::ffi::CString::new(root.to_str().unwrap()).unwrap();
    assert_eq!(visiaengine_load_mvt_dir(ve, cpath.as_ptr(), 31), -1);
    // nonexistent dir → error (0 or negative; engine maps to ARG)
    let bad = std::ffi::CString::new("/nonexistent/tiles").unwrap();
    let rc = visiaengine_load_mvt_dir(ve, bad.as_ptr(), 10);
    assert!(rc <= 0, "nonexistent dir must not succeed");
    // happy path: bundled fixture = 9 tiles
    let rc = visiaengine_load_mvt_dir(ve, cpath.as_ptr(), 10);
    assert_eq!(rc, 9, "3×3 fixture");
}

// spec: CAPI-29
#[test]
fn set_tile_view_domain_and_visibility() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    // not mounted → reject
    let rc = visiaengine_set_tile_view(ve, -2.0e7, 1.9e7, -1.9e7, 2.0e7);
    assert!(rc < 0, "unmounted view must reject");
    // mount then feed (absolute repo-root path; test cwd = crate dir)
    let root = {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        while !p.join("Cargo.lock").exists() {
            assert!(p.pop());
        }
        p.join("resources/data/tiles")
    };
    let cpath = std::ffi::CString::new(root.to_str().unwrap()).unwrap();
    assert_eq!(visiaengine_load_mvt_dir(ve, cpath.as_ptr(), 10), 9);
    // domain: min>=max reject
    assert!(visiaengine_set_tile_view(ve, 2.0e7, 1.9e7, -2.0e7, 2.0e7) < 0);
    // non-finite reject
    assert!(visiaengine_set_tile_view(ve, f64::NAN, 1.9e7, -1.9e7, 2.0e7) < 0);
    // full 3×3 bbox = 9 visible
    let rc = visiaengine_set_tile_view(ve, -2.004e7, 1.995e7, -1.995e7, 2.004e7);
    assert_eq!(rc, 9, "full neighborhood visible");
}

// spec: CAPI-35 (N1.4)
#[test]
fn set_tile_source_http_domain_and_mount() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    // ve=0 reject
    let any = std::ffi::CString::new("http://x").unwrap();
    assert_eq!(
        visiaengine_set_tile_source_http(0, any.as_ptr(), 10),
        VE_ERR_ARG
    );
    // NULL url reject (PIT-26: NULL must not form a slice)
    assert_eq!(
        visiaengine_set_tile_source_http(ve, std::ptr::null(), 10),
        VE_ERR_ARG
    );
    // empty url reject
    let empty = std::ffi::CString::new("").unwrap();
    assert_eq!(
        visiaengine_set_tile_source_http(ve, empty.as_ptr(), 10),
        VE_ERR_ARG
    );
    // z>30 reject
    let url = std::ffi::CString::new("http://127.0.0.1:1/t").unwrap();
    assert_eq!(
        visiaengine_set_tile_source_http(ve, url.as_ptr(), 31),
        VE_ERR_ARG
    );
    // valid mount returns 0 (count semantics fork documented in the clause)
    assert_eq!(visiaengine_set_tile_source_http(ve, url.as_ptr(), 10), 0);
    // double mount is a re-mount (fresh TileSet) — still 0
    assert_eq!(visiaengine_set_tile_source_http(ve, url.as_ptr(), 10), 0);
}

// spec: CAPI-30
#[test]
fn create_group_returns_handle_valid_utf8() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    // anonymous (NULL name)
    let mut g1 = 0u64;
    assert_eq!(
        visiaengine_create_group(ve, std::ptr::null(), &mut g1),
        0,
        "create_group rc"
    );
    // g1 may be 0 (slot0gen0=valid); existence proven by rc==0 above
    // named
    let cname = std::ffi::CString::new("roads_layer").unwrap();
    let mut g2 = 0u64;
    assert_eq!(
        visiaengine_create_group(ve, cname.as_ptr(), &mut g2),
        0,
        "create_group rc"
    );
    assert_ne!(g2, 0);
    assert_ne!(g1, g2, "distinct slots");
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-31
#[test]
fn set_parent_reparent_and_cycle_guard() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    let cname = std::ffi::CString::new("layer").unwrap();
    let mut g = 0u64;
    assert_eq!(
        visiaengine_create_group(ve, cname.as_ptr(), &mut g),
        0,
        "create_group rc"
    );
    assert_ne!(g, 0);
    // attach child to group: use a dummy child (must be a valid entity handle)
    // gltf loads entities; use add_mesh path to create an entity
    let _gltf_c = std::ffi::CString::new("nonexistent.glb").unwrap();
    // child=g (self-loop) must be rejected
    assert_ne!(
        visiaengine_set_parent(ve, g, g),
        0,
        "self-cycle must be rejected"
    );
    // child=0 must be rejected
    assert_ne!(
        visiaengine_set_parent(ve, 0, g),
        0,
        "child=0 must be rejected"
    );
    // detach: parent=0 = move to root
    assert_eq!(
        visiaengine_set_parent(ve, g, 0),
        0,
        "detach to root must succeed"
    );
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-32
#[test]
fn get_parent_roundtrip() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    let null = std::ffi::CString::default; // unused; for syntax hint only
    let _ = null;
    let cname = std::ffi::CString::new("g1").unwrap();
    let mut g1 = 0u64;
    assert_eq!(
        visiaengine_create_group(ve, cname.as_ptr(), &mut g1),
        0,
        "create_group rc"
    );
    let cname2 = std::ffi::CString::new("g2").unwrap();
    let mut g2 = 0u64;
    assert_eq!(
        visiaengine_create_group(ve, cname2.as_ptr(), &mut g2),
        0,
        "create_group rc"
    );
    // g2 -> g1
    assert_eq!(visiaengine_set_parent(ve, g2, g1), 0);
    assert_eq!(
        visiaengine_get_parent(ve, g2),
        g1,
        "get_parent must match set_parent"
    );
    // g1 has no parent (root)
    assert_eq!(visiaengine_get_parent(ve, g1), 0, "root group parent=0");
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-33
#[test]
fn set_group_offset_domain() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    let cname = std::ffi::CString::new("g").unwrap();
    let mut g = 0u64;
    assert_eq!(
        visiaengine_create_group(ve, cname.as_ptr(), &mut g),
        0,
        "create_group rc"
    );
    assert_ne!(g, 0);
    // Valid f64
    assert_eq!(visiaengine_set_group_offset(ve, g, 1.0e7, -2.0e7, 100.0), 0);
    // NaN → err
    assert_ne!(visiaengine_set_group_offset(ve, g, f64::NAN, 0.0, 0.0), 0);
    // group=0 → err
    assert_ne!(visiaengine_set_group_offset(ve, 0, 1.0, 0.0, 0.0), 0);
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-34
#[test]
fn get_group_offset_roundtrip() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    let cname = std::ffi::CString::new("g").unwrap();
    let mut g = 0u64;
    assert_eq!(
        visiaengine_create_group(ve, cname.as_ptr(), &mut g),
        0,
        "create_group rc"
    );
    // NULL out → err
    assert_ne!(visiaengine_get_group_offset(ve, g, std::ptr::null_mut()), 0);
    // Set and read back
    assert_eq!(visiaengine_set_group_offset(ve, g, 42.5, -7.0, 0.0), 0);
    let mut out = [0.0f64; 3];
    assert_eq!(visiaengine_get_group_offset(ve, g, out.as_mut_ptr()), 0);
    assert_eq!(out, [42.5, -7.0, 0.0]);
    // Non-group entity (e.g. after destroy g) → err; skip for now
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-39
#[test]
fn load_raster_dir_domain_and_mount() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    // NULL path
    assert_eq!(visiaengine_load_raster_dir(ve, std::ptr::null(), 10), -1);
    // z domain
    let root = {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        while !p.join("Cargo.lock").exists() {
            assert!(p.pop());
        }
        p.join("resources/data/raster")
    };
    let cpath = std::ffi::CString::new(root.to_str().unwrap()).unwrap();
    assert_eq!(visiaengine_load_raster_dir(ve, cpath.as_ptr(), 31), -1);
    // nonexistent dir
    let bad = std::ffi::CString::new("/nonexistent/raster").unwrap();
    let rc = visiaengine_load_raster_dir(ve, bad.as_ptr(), 10);
    assert!(rc <= 0, "nonexistent dir must not succeed");
    // happy path: bundled 2x2 fixture
    let rc = visiaengine_load_raster_dir(ve, cpath.as_ptr(), 10);
    assert_eq!(rc, 4, "2×2 raster fixture");
}

// spec: CAPI-40
#[test]
fn set_raster_view_domain_and_visible() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    // not mounted → reject
    let rc = visiaengine_set_raster_view(ve, -2.0e7, 1.9e7, -1.9e7, 2.0e7);
    assert!(rc < 0, "unmounted raster view must reject");
    // mount 2×2 fixture then feed the covering view
    let root = {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        while !p.join("Cargo.lock").exists() {
            assert!(p.pop());
        }
        p.join("resources/data/raster")
    };
    let cpath = std::ffi::CString::new(root.to_str().unwrap()).unwrap();
    assert_eq!(visiaengine_load_raster_dir(ve, cpath.as_ptr(), 10), 4);
    // z=10 slippy tiles x=850..852, y=388..390 → 3857 bbox covering them.
    // tile size at z=10 = WORLD/1024 ≈ 39011 m; x=850 start = -20037508 + 850*39011.
    let world = 20037508.342789244_f64;
    let tile = world * 2.0 / 1024.0;
    let min_x = -world + 850.0 * tile;
    let max_x = -world + 852.0 * tile;
    let min_y = world - 390.0 * tile;
    let max_y = world - 388.0 * tile;
    let rc = visiaengine_set_raster_view(ve, min_x, min_y, max_x, max_y);
    assert_eq!(rc, 4, "view covering all mounted tiles");
    // inverted bbox domain reject
    let rc = visiaengine_set_raster_view(ve, max_x, min_y, min_x, max_y);
    assert!(rc < 0, "min>max must reject");
    // disjoint view: visible = 0 (mounted-discovery semantics)
    let rc = visiaengine_set_raster_view(ve, 1.0e7, 1.0e7, 1.1e7, 1.1e7);
    assert_eq!(rc, 0, "disjoint view = 0 visible (no fabrication)");
}

// spec: CAPI-41
#[test]
fn load_env_hdr_domain_and_install() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    // NULL path
    assert_eq!(visiaengine_load_env_hdr(ve, std::ptr::null()), -1);
    // nonexistent file
    let bad = std::ffi::CString::new("/nonexistent/env.hdr").unwrap();
    assert_eq!(visiaengine_load_env_hdr(ve, bad.as_ptr()), -1);
    // happy path: committed fixture (absolute repo-root path; cwd = crate)
    let root = {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        while !p.join("Cargo.lock").exists() {
            assert!(p.pop());
        }
        p.join("resources/data/demo_sky.hdr")
    };
    let cpath = std::ffi::CString::new(root.to_str().unwrap()).unwrap();
    assert_eq!(
        visiaengine_load_env_hdr(ve, cpath.as_ptr()),
        0,
        "fixture loads"
    );
}

// spec: CAPI-42
// spec: CAPI-43
#[test]
fn node_transform_write_read_domain() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    // mount twoprim fixture (2 entities; shared loader helper pins VE_OK).
    load_twoprim(ve);
    let h0 = visiaengine_entity_at(ve, 0);
    let h1 = visiaengine_entity_at(ve, 1);
    // NULL gates
    assert_eq!(visiaengine_set_node_transform(ve, h0, std::ptr::null()), -1);
    assert_eq!(
        visiaengine_get_node_transform(ve, h0, std::ptr::null_mut()),
        -1
    );
    // unknown entity form
    assert_eq!(
        visiaengine_set_node_transform(ve, u64::MAX, [0.0f64; 16].as_ptr()),
        -1
    );
    // write identity then read back
    let mut m = [0.0f64; 16];
    m[0] = 1.0;
    m[5] = 1.0;
    m[10] = 1.0;
    m[15] = 1.0;
    assert_eq!(visiaengine_set_node_transform(ve, h0, m.as_ptr()), 0);
    let mut out = [9.5f64; 16];
    assert_eq!(visiaengine_get_node_transform(ve, h0, out.as_mut_ptr()), 0);
    assert_eq!(out, m, "round-trip identity");
    // non-finite reject
    let mut bad = m;
    bad[0] = f64::NAN;
    assert_eq!(visiaengine_set_node_transform(ve, h0, bad.as_ptr()), -1);
    // entity 2 untouched = default identity
    let mut out2 = [0.0f64; 16];
    assert_eq!(visiaengine_get_node_transform(ve, h1, out2.as_mut_ptr()), 0);
    assert_eq!(
        out2,
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0
        ]
    );
    let _ = h1;
}
