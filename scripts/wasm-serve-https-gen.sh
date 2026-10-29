#!/usr/bin/env bash
# wasm-serve-https-gen — local CA + LAN leaf cert for the serve-https lane
# (owner ruling 2026-10-16). Artifacts: .certs/server.pem + server-key.pem,
# gitignored. Idempotent: regenerates only when the leaf is missing or the
# detected address set changed. Flags are measured against `mkcert -help`
# of the installed 1.4.4 build; no imagined options.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/.certs"
MKCERT="${MKCERT_BIN:-}"
[ -x "$MKCERT" ] || MKCERT="$HOME/.pixi/bin/mkcert"
[ -x "$MKCERT" ] || MKCERT="$HOME/.pixi/global/bin/mkcert"
[ -x "$MKCERT" ] || { echo "GEN FAIL: mkcert missing: pixi global install mkcert"; exit 1; }

# Local CA: mkcert -install creates CAROOT files, then tries system trust
# stores; in a sudo-less container that second step exits nonzero while the
# CA itself is fine — tolerate it loudly, proceed once rootCA.pem exists.
CAR=$("$MKCERT" -CAROOT)
if [ ! -f "$CAR/rootCA.pem" ]; then
  "$MKCERT" -install >/dev/null 2>&1 || true
  [ -f "$CAR/rootCA.pem" ] || { echo "GEN FAIL: local CA could not be created"; exit 1; }
fi

# Desired SAN set: fixed loopback names + every IPv4 hostname -I reports
# (dynamic, never hardcoded; on this host: a libvirt-style 192.168.x and an
# internal 10.x — both RFC1918, both included; a public-looking address
# would also be included but flagged here for the reader).
need=(localhost 127.0.0.1)
for ip in $(hostname -I | tr ' ' '\n' | grep -E '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$'); do
  case "$ip" in
    10.*|192.168.*|172.1[6-9].*|172.2[0-9].*|172.3[01].*) need+=("$ip") ;;
    127.*) ;;
    *) need+=("$ip") ; echo "GEN NOTE: non-private address $ip included" ;;
  esac
done

have=()
if [ -f "$DIR/server.pem" ]; then
  san=$(openssl x509 -in "$DIR/server.pem" -noout -ext subjectAltName 2>/dev/null | tr ',' '\n')
  while read -r v; do [ -n "$v" ] && have+=("$v"); done < <(
    printf '%s\n' "$san" | sed -E 's/.*DNS: *([^,]*).*/\1/; s/.*IP Address: *([^,]*).*/\1/; t; d')
fi
same=1
[ "${#have[@]}" -eq "${#need[@]}" ] || same=0
if [ "$same" -eq 1 ]; then
  for n in "${need[@]}"; do
    hit=0; for h in "${have[@]}"; do [ "$h" = "$n" ] && hit=1; done
    [ "$hit" -eq 1 ] || { same=0; break; }
  done
fi
if [ "$same" -eq 1 ] && [ -f "$DIR/server-key.pem" ]; then
  echo "cert current: $DIR/server.pem  SANs: ${need[*]}"
  exit 0
fi

mkdir -p "$DIR"
"$MKCERT" -cert-file "$DIR/server.pem" -key-file "$DIR/server-key.pem" "${need[@]}" >/dev/null 2>&1 \
  || { echo "GEN FAIL: mkcert leaf generation failed"; exit 1; }
chmod 600 "$DIR/server-key.pem"

# NSS trust for the chromium family, if certutil exists (measured absence on
# this host: pixi mkcert carries no certutil, PATH has none -> tolerated per
# ruling; proof lane then runs with --ignore-certificate-errors).
if command -v certutil >/dev/null 2>&1 && [ -n "$(command -v certutil)" ]; then
  mkdir -p "$HOME/.pki/nssdb"
  certutil -A -d "sql:$HOME/.pki/nssdb" -n "mkcert-lane-root" -t "C,," -i "$CAR/rootCA.pem" 2>/dev/null \
    || echo "GEN NOTE: certutil present but import failed (non-fatal)"
else
  echo "GEN NOTE: no certutil; rootCA not in NSS (client route or flag route per ruling)"
fi
echo "cert ready: $DIR/server.pem  SANs: ${need[*]}"
