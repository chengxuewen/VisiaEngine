//! HttpSource (IO-15, tile-streaming Phase 1): loopback TCP server serves the
//! bundled fixture bytes; NO external network (PIT-7 discipline — CI stays
//! offline; the live-URL path is a separate #[ignore] test).

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use visiaengine_io_tiles::{HttpSource, SourceError, TileId, TileSource};

/// Minimal HTTP/1.0 server on an ephemeral loopback port: serves
/// `/{z}/{x}/{y}.mvt` from the fixture tree; 404 otherwise.
fn spawn_fixture_server() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("addr");
    let handle = thread::spawn(move || {
        for req in listener.incoming() {
            let Ok(mut stream) = req else { break };
            handle_one(&mut stream);
        }
    });
    (format!("http://{addr}"), handle)
}

fn handle_one(stream: &mut TcpStream) {
    let mut buf = [0u8; 2048];
    let _ = stream.read(&mut buf);
    let req = String::from_utf8_lossy(&buf);
    let path = req.split_whitespace().nth(1).unwrap_or("/");
    // path = /z/x/y.mvt
    let rel = path.trim_start_matches('/');
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/data/tiles");
    let full = format!("{root}/{rel}");
    match std::fs::read(&full) {
        Ok(body) => {
            let head = format!("HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n", body.len());
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&body);
        }
        Err(_) => {
            let body = b"not found";
            let head = format!(
                "HTTP/1.0 404 Not Found\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(body);
        }
    }
}

// spec: IO-15
#[test]
fn http_source_fetches_fixture_over_loopback() {
    let (root, _worker) = spawn_fixture_server();
    let src = HttpSource::new(root);
    let bytes = src.load(10, 0, 0).expect("fixture over http");
    assert!(!bytes.is_empty(), "tile bytes non-empty");
    // decode sanity: it IS an MVT (decodable by the sibling module)
    let tile = visiaengine_io_tiles::decode_tile(&bytes).expect("valid MVT");
    assert!(!tile.layers.is_empty());
}

// spec: IO-15
#[test]
fn http_404_maps_to_not_found() {
    let (root, _worker) = spawn_fixture_server();
    let src = HttpSource::new(root);
    let err = src.load(10, 500, 500).expect_err("missing tile");
    assert!(matches!(err, SourceError::NotFound(_)));
}

// spec: IO-15 (live network — #[ignore], never in CI per PIT-7)
#[test]
#[ignore = "live network: run manually with --ignored"]
fn http_live_openstreetmap_tile() {
    let src = HttpSource::new("https://tile.openstreetmap.org");
    let bytes = src.load(0, 0, 0).expect("live z0 tile");
    assert!(!bytes.is_empty());
}

// spec: IO-15
#[test]
fn scheduler_over_http_end_to_end() {
    let (root, _worker) = spawn_fixture_server();
    let mut set = visiaengine_io_tiles::TileSet::new(Box::new(HttpSource::new(root))).expect("set");
    let ids = visiaengine_io_tiles::TileSet::visible(
        visiaengine_io_tiles::TileId::new(10, 0, 0).unwrap().bbox(),
        10,
    );
    assert_eq!(ids.len(), 1);
    let stats = set.ensure(&ids).expect("ensure over http");
    assert_eq!(stats.loaded, 1);
    assert!(set.decoded(&TileId::new(10, 0, 0).unwrap()).is_some());
}
