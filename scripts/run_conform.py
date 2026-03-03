#!/usr/bin/env python3
"""All-in-one conformance test runner.

Starts grey-conform, replays a trace, compares results, and captures the
server log.  Useful for quick iteration without manually managing the server
process.

Usage:
    python3 scripts/run_conform.py [TRACE_DIR] [--blocks N] [--log FILE]
"""

import argparse
import os
import signal
import socket
import struct
import subprocess
import sys
import time
from pathlib import Path

DEFAULT_TRACE = "res/conformance/fuzz-proto/examples/0.7.2/no_forks"
DEFAULT_LOG = "/tmp/grey_conform.log"
BINARY = "target/release/grey-conform"


def send_msg(sock, data):
    sock.sendall(struct.pack("<I", len(data)) + data)


def recv_msg(sock, timeout=120):
    sock.settimeout(timeout)
    hdr = b""
    while len(hdr) < 4:
        chunk = sock.recv(4 - len(hdr))
        if not chunk:
            return None
        hdr += chunk
    length = struct.unpack("<I", hdr)[0]
    data = b""
    while len(data) < length:
        chunk = sock.recv(min(65536, length - len(data)))
        if not chunk:
            return None
        data += chunk
    return data


def find_binary():
    """Find the grey-conform binary, building if necessary."""
    if os.path.exists(BINARY):
        return BINARY
    alt = os.path.join(os.path.dirname(__file__), "..", BINARY)
    if os.path.exists(alt):
        return alt
    print(f"Binary not found at {BINARY}. Building...")
    result = subprocess.run(
        ["cargo", "build", "--release", "--bin", "grey-conform"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"Build failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    return BINARY


def main():
    parser = argparse.ArgumentParser(description="Run conformance trace")
    parser.add_argument(
        "trace_dir",
        nargs="?",
        default=DEFAULT_TRACE,
        help=f"Trace directory (default: {DEFAULT_TRACE})",
    )
    parser.add_argument(
        "--blocks",
        type=int,
        default=0,
        help="Stop after N blocks (0 = all)",
    )
    parser.add_argument(
        "--log",
        default=DEFAULT_LOG,
        help=f"Server log file (default: {DEFAULT_LOG})",
    )
    args = parser.parse_args()

    trace_dir = Path(args.trace_dir)
    if not trace_dir.exists():
        print(f"Error: trace dir {trace_dir} not found", file=sys.stderr)
        sys.exit(1)

    binary = find_binary()
    sock_path = f"/tmp/grey_conform_{os.getpid()}.sock"

    # Clean up stale socket
    try:
        os.unlink(sock_path)
    except FileNotFoundError:
        pass

    # Collect trace files
    fuzzer_files = sorted(trace_dir.glob("*_fuzzer_*.bin"))
    target_files = {f.name[:8]: f for f in trace_dir.glob("*_target_*.bin")}

    if args.blocks > 0:
        # +2 for peer_info and initialize messages
        fuzzer_files = fuzzer_files[: args.blocks + 2]

    # Start server
    env = os.environ.copy()
    rust_log = env.get("RUST_LOG", "grey_state=info,grey=info")
    env["RUST_LOG"] = rust_log

    log_fd = os.open(args.log, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o644)
    proc = subprocess.Popen(
        [binary, sock_path],
        env=env,
        stdout=log_fd,
        stderr=log_fd,
    )
    os.close(log_fd)

    # Wait for socket
    for _ in range(20):
        if os.path.exists(sock_path):
            break
        time.sleep(0.1)
    else:
        print("Error: server did not create socket", file=sys.stderr)
        proc.kill()
        sys.exit(1)

    passed = 0
    failed = 0
    first_fail = None

    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.settimeout(120)
        s.connect(sock_path)

        for fuzz_file in fuzzer_files:
            seq = fuzz_file.name[:8]
            request = fuzz_file.read_bytes()

            send_msg(s, request)
            resp = recv_msg(s, timeout=120)

            if resp is None:
                print(f"msg {seq}: no response")
                break

            # Find matching target file
            target_key = seq
            matching = [
                t for k, t in target_files.items() if k == target_key
            ]

            if not matching:
                disc = resp[0]
                if disc == 0x00:
                    print(f"msg {seq}: peer_info ({len(resp)} bytes)")
                else:
                    print(f"msg {seq}: response ({len(resp)} bytes)")
                continue

            expected = matching[0].read_bytes()

            if resp == expected:
                if resp[0] == 0x02:
                    print(f"msg {seq}: PASS")
                else:
                    print(f"msg {seq}: PASS (peer_info)")
                passed += 1
            elif resp[0] == 0x00 and expected[0] == 0x00:
                # PeerInfo: implementation version may differ
                print(f"msg {seq}: PASS (peer_info)")
                passed += 1
            elif resp[0] == 0xFF and expected[0] == 0xFF:
                print(f"msg {seq}: PASS (error)")
                passed += 1
            else:
                failed += 1
                if first_fail is None:
                    first_fail = seq
                if resp[0] == 0x02 and expected[0] == 0x02:
                    exp_root = expected[1:33].hex()
                    got_root = resp[1:33].hex()
                    print(f"msg {seq}: FAIL")
                    print(f"  expected: {exp_root}")
                    print(f"  got:      {got_root}")
                elif resp[0] == 0xFF:
                    msg = resp[1:].decode("utf-8", errors="replace")[:120]
                    print(f"msg {seq}: ERROR: {msg}")
                else:
                    print(
                        f"msg {seq}: FAIL (type 0x{resp[0]:02x}"
                        f" vs 0x{expected[0]:02x})"
                    )

        s.close()
    except Exception as e:
        print(f"\nException: {e}", file=sys.stderr)
    finally:
        proc.send_signal(signal.SIGTERM)
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
        try:
            os.unlink(sock_path)
        except FileNotFoundError:
            pass

    log_size = os.path.getsize(args.log)
    print(f"\n{'='*50}")
    print(f"Results: {passed} passed, {failed} failed")
    if first_fail:
        print(f"First failure: msg {first_fail}")
    print(f"Server log: {args.log} ({log_size:,} bytes)")


if __name__ == "__main__":
    main()
