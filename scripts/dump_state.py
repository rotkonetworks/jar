#!/usr/bin/env python3
"""Dump state KV pairs at a specific block for debugging.

Replays the trace up to the given block number, then requests a state dump
via the GetState protocol message.  Shows each state component with its
size and hash, useful for narrowing down which component diverged.

Usage:
    # Dump state after block 8 (the last passing block)
    python3 scripts/dump_state.py --block 8

    # Dump state after block 9 (the first failing block)
    python3 scripts/dump_state.py --block 9

    # Against a running server (don't start one)
    python3 scripts/dump_state.py --block 8 --socket /tmp/jam_target.sock
"""

import argparse
import hashlib
import os
import signal
import socket
import struct
import subprocess
import sys
import time
from pathlib import Path

DEFAULT_TRACE = "res/conformance/fuzz-proto/examples/0.7.2/no_forks"
BINARY = "target/release/grey-conform"

# State key names (first byte of 31-byte key, rest zeros)
KEY_NAMES = {
    1: "auth_pool",
    2: "auth_queue",
    3: "recent_blocks",
    4: "safrole",
    5: "judgments",
    6: "entropy",
    7: "pending_validators",
    8: "current_validators",
    9: "previous_validators",
    10: "pending_reports",
    11: "timeslot",
    12: "privileged",
    13: "statistics",
    14: "accumulation_queue",
    15: "accumulation_history",
    16: "accumulation_outputs",
}


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


def read_compact(data, pos):
    """Read a JAM compact-encoded natural number.

    Matches grey-codec's decode_compact:
    - Count leading 1-bits of the header byte (= len, 0..8)
    - If len == 8 (header 0xFF): next 8 bytes as u64 LE
    - Otherwise: threshold = 256 - (1 << (8 - len)),
      header_value = header - threshold, low = next `len` bytes LE,
      value = (header_value << (8 * len)) | low
    """
    if pos >= len(data):
        return 0, pos
    header = data[pos]
    pos += 1

    # Count leading 1-bits
    length = 0
    tmp = header
    while tmp & 0x80:
        length += 1
        tmp = (tmp << 1) & 0xFF

    if length == 8:
        # 0xFF prefix: next 8 bytes are u64 LE
        val = int.from_bytes(data[pos : pos + 8], "little")
        pos += 8
        return val, pos

    if length == 0:
        threshold = 0
    else:
        threshold = 256 - (1 << (8 - length))

    header_value = header - threshold

    low = 0
    for i in range(length):
        low |= data[pos] << (8 * i)
        pos += 1

    val = (header_value << (8 * length)) | low
    return val, pos


def parse_state_response(data):
    """Parse a GetState response (disc 0x05).

    Format: 0x05 + compact(count) + count * (31-byte key + compact(value_len) + value).
    No ancestry in our implementation's response.
    """
    if not data or data[0] != 0x05:
        if data and data[0] == 0xFF:
            msg = data[1:].decode("utf-8", errors="replace")[:200]
            print(f"Error response: {msg}")
        else:
            print(f"Unexpected response type: 0x{data[0]:02x}" if data else "empty")
        return None

    pos = 1

    # State KV pairs: compact(count) + count * (31-byte key + compact_len + value)
    count, pos = read_compact(data, pos)
    kvs = {}
    for _ in range(count):
        if pos + 31 > len(data):
            print(f"Warning: truncated at KV pair {len(kvs)}/{count}")
            break
        key = bytes(data[pos : pos + 31])
        pos += 31
        vlen, pos = read_compact(data, pos)
        val = bytes(data[pos : pos + vlen])
        pos += vlen
        kvs[key] = val

    return kvs


def key_name(key_bytes):
    """Human-readable name for a 31-byte state key."""
    if key_bytes[1:] == b"\x00" * 30:
        idx = key_bytes[0]
        return KEY_NAMES.get(idx, f"C({idx})")
    if key_bytes[0] == 255:
        # Service account key: interleaved bytes
        sid = (
            key_bytes[1]
            | (key_bytes[3] << 8)
            | (key_bytes[5] << 16)
            | (key_bytes[7] << 24)
        )
        return f"service_account({sid})"
    # Service data key
    sid = (
        key_bytes[0]
        | (key_bytes[2] << 8)
        | (key_bytes[4] << 16)
        | (key_bytes[6] << 24)
    )
    return f"service_data({sid})"


def main():
    parser = argparse.ArgumentParser(description="Dump state at a block boundary")
    parser.add_argument(
        "trace_dir",
        nargs="?",
        default=DEFAULT_TRACE,
        help=f"Trace directory (default: {DEFAULT_TRACE})",
    )
    parser.add_argument(
        "--block",
        type=int,
        required=True,
        help="Block number to dump state after (1-indexed, matching msg numbers)",
    )
    parser.add_argument(
        "--socket",
        default=None,
        help="Connect to existing server socket (don't start one)",
    )
    args = parser.parse_args()

    trace_dir = Path(args.trace_dir)
    if not trace_dir.exists():
        print(f"Error: trace dir {trace_dir} not found", file=sys.stderr)
        sys.exit(1)

    fuzzer_files = sorted(trace_dir.glob("*_fuzzer_*.bin"))

    # Start server if needed
    proc = None
    sock_path = args.socket
    if sock_path is None:
        sock_path = f"/tmp/grey_dump_{os.getpid()}.sock"
        try:
            os.unlink(sock_path)
        except FileNotFoundError:
            pass

        binary = BINARY
        if not os.path.exists(binary):
            print("Building grey-conform...")
            subprocess.run(
                ["cargo", "build", "--release", "--bin", "grey-conform"],
                check=True,
                capture_output=True,
            )

        proc = subprocess.Popen(
            [binary, sock_path],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        for _ in range(20):
            if os.path.exists(sock_path):
                break
            time.sleep(0.1)

    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.settimeout(120)
        s.connect(sock_path)

        # Send messages up to the requested block
        # msg 0 = peer_info, msg 1 = initialize, msg 2+ = import_block
        # So --block N means send messages 0..N+1 (N+2 messages)
        n_msgs = min(args.block + 2, len(fuzzer_files))

        last_state_root = None
        last_header_hash = None

        for i, fuzz_file in enumerate(fuzzer_files[:n_msgs]):
            seq = fuzz_file.name[:8]
            request = fuzz_file.read_bytes()
            send_msg(s, request)
            resp = recv_msg(s)

            if resp is None:
                print(f"msg {seq}: no response")
                return

            if resp[0] == 0xFF:
                msg = resp[1:].decode("utf-8", errors="replace")[:200]
                print(f"msg {seq}: ERROR: {msg}")
                return

            if resp[0] == 0x02:
                last_state_root = resp[1:33].hex()
                # The header hash for GetState: we need it from the block data.
                # The block's header hash is computed by the target.
                # We can extract the parent hash from the next block to infer it.
                print(f"msg {seq}: state_root {last_state_root[:32]}...")
            elif resp[0] == 0x00:
                print(f"msg {seq}: peer_info")

        # After the last block, we need to find the header hash to request
        # GetState.  The ancestry is tracked by the target.  We can try
        # sending GetState with a zero hash to see if the target returns the
        # latest state, or we extract it from the next block's parent hash.
        if n_msgs < len(fuzzer_files):
            # Read the next block to get its parent hash (= our header hash)
            next_block = fuzzer_files[n_msgs].read_bytes()
            if next_block[0] == 0x03:  # ImportBlock
                last_header_hash = next_block[1:33]
            elif next_block[0] == 0x01:  # Initialize
                last_header_hash = next_block[1:33]
        else:
            # Last block in trace - try to extract from the response ancestry
            # or just request state for the last known parent
            pass

        if last_header_hash is None:
            print("\nCannot determine header hash for GetState request.")
            print("Tip: use --block N where N < total blocks so the next")
            print("block's parent hash can be extracted.")
            s.close()
            return

        print(f"\nRequesting GetState for header {last_header_hash.hex()[:32]}...")
        get_state_msg = bytes([0x04]) + last_header_hash
        send_msg(s, get_state_msg)
        resp = recv_msg(s, timeout=120)

        kvs = parse_state_response(resp)
        if kvs is None:
            s.close()
            return

        print(f"\nState: {len(kvs)} KV pairs")
        print(f"{'Component':<30} {'Size':>8}  {'Blake2b-256':>32}")
        print("-" * 75)

        for key_bytes in sorted(kvs.keys()):
            val = kvs[key_bytes]
            name = key_name(key_bytes)
            h = hashlib.blake2b(val, digest_size=32).hexdigest()[:32]
            print(f"{name:<30} {len(val):>8}  {h}")

        total_bytes = sum(len(v) for v in kvs.values())
        print("-" * 75)
        print(f"{'Total':<30} {total_bytes:>8}  ({len(kvs)} keys)")
        print(f"\nState root: {last_state_root}")

        s.close()
    finally:
        if proc:
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


if __name__ == "__main__":
    main()
