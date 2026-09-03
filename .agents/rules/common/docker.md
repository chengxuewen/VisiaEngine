# Docker & Network Constraints

> Topic file, loaded on demand (not in opencode.json instructions). Docker, network, and container-specific constraints for VisiaEngine development.

## Volume Mount Performance

- Bind mounts on macOS (osxfs / VirtioFS) are slow for build-heavy trees. If a full workspace bind-mount makes builds crawl, keep build artifacts (`target/`, `node_modules/`) inside a named volume or the container image, and bind-mount only source.
- When the container is recreated, tools installed ad-hoc inside it (`apt install` debuggers, profilers) are lost — install debug tooling in the Dockerfile dev target instead of per-container.

## Dependency Cache Discipline

- Persist the package-manager cache across container rebuilds:
  - Rust: mount a cargo cache volume (`~/.cargo/registry`, `~/.cargo/git`) or use a bake-stage image.
  - JS: volume for `~/.npm` or use npm ci with a layer-cached `node_modules`.
- First full dependency build of a heavy native tree inside Docker can take 15–30 minutes (vs. minutes natively) — cache aggressively, measure against a clean build.

## UDP Port Ranges

- RTC / media worker processes need a dedicated UDP port range. Open the range in `docker-compose` / firewall mapping:

  ```yaml
  ports:
    - "<start>-<end>:<start>-<end>/udp"
  ```

- Default ranges are usually wide for development; for production, narrow the range to the minimum needed for expected concurrent sessions — fewer ports reduce the firewall surface.

## ICE/STUN Required for Local P2P (if using WebRTC)

- WebRTC (even on localhost) requires **ICE negotiation** with STUN to discover candidate pairs. Without a STUN server, localhost connections fail because no candidate pairs are formed.
- **Development setup**: Run a STUN server locally (`coturn`, `stuntman`), or configure the transport to use a host-loopback ICE candidate.
- **Common gotcha**: Host and client on the same machine assume localhost WebRTC "just works" — it does not. ICE must be configured explicitly even for loopback.
- ICE-Lite (server-side) reduces the handshake to one round trip but still requires the client to send a STUN binding request.

## curl / Proxy Pitfalls (host shell)

- When a shell environment has `http_proxy` set, `curl http://127.0.0.1:<port>` walks into the proxy and times out — use `curl --noproxy "*" http://127.0.0.1:<port>/` for local services.
- Container tcpdump filtering: traffic from the host to a container subnet has its source IP rewritten to the gateway address by NAT — distinguish by source port, not by source IP.

