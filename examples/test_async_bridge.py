"""
Test the async bridge between Python and Rust.

This demonstrates that:
1. Tokio runtime is initialized
2. Python can await Rust async functions
3. The async bridge works correctly with pyo3-async-runtimes
"""

import argparse
import asyncio
import pprint
import socket
import time


def probe_tcp(host: str, port: int, timeout_s: float = 0.2) -> bool:
    try:
        with socket.create_connection((host, port), timeout=timeout_s):
            return True
    except OSError:
        return False


async def test_basic_async(driftpyrs):
    """Test that we can await a simple async function."""
    print("Testing basic async bridge...")

    start = time.time()
    result = await driftpyrs.sleep_and_return(1)
    elapsed = time.time() - start

    assert result == "Slept for 1 seconds", f"Unexpected result: {result}"
    assert elapsed >= 1.0, f"Function returned too quickly: {elapsed}s"
    assert elapsed < 1.2, f"Function took too long: {elapsed}s"

    print(f"✓ Basic async works! Result: '{result}' (took {elapsed:.2f}s)")


async def test_concurrent_async(driftpyrs):
    """Test that multiple async calls can run concurrently."""
    print("\nTesting concurrent async calls...")

    start = time.time()
    results = await asyncio.gather(
        driftpyrs.sleep_and_return(1),
        driftpyrs.sleep_and_return(1),
        driftpyrs.sleep_and_return(1),
    )
    elapsed = time.time() - start

    assert len(results) == 3
    assert all(r == "Slept for 1 seconds" for r in results)
    # Should take ~1 second (concurrent), not ~3 seconds (sequential)
    assert elapsed < 1.5, f"Calls ran sequentially, not concurrently: {elapsed}s"

    print(f"✓ Concurrent async works! 3 calls completed in {elapsed:.2f}s")


async def drive_console_load(driftpyrs, n: int, sleep_s: int, rounds: int):
    for i in range(rounds):
        start = time.time()
        results = await asyncio.gather(
            *[driftpyrs.sleep_and_return(sleep_s) for _ in range(n)]
        )
        elapsed = time.time() - start
        assert len(results) == n
        assert all(r == f"Slept for {sleep_s} seconds" for r in results)
        print(f"✓ Load round {i + 1}/{rounds}: {n} tasks in {elapsed:.2f}s")


async def main():
    p = argparse.ArgumentParser()
    p.add_argument("--console", action="store_true")
    p.add_argument("--n", type=int, default=3)
    p.add_argument("--sleep", type=int, default=1)
    p.add_argument("--rounds", type=int, default=1)
    p.add_argument("--hold", type=float, default=0.0)
    p.add_argument("--bind", type=str, default=None)
    p.add_argument("--wait", type=float, default=0.0)
    args = p.parse_args()

    if args.bind:
        import os

        os.environ["TOKIO_CONSOLE_BIND"] = args.bind

    import driftpyrs

    pprint.pprint(driftpyrs.build_info())

    if args.console:
        bind = args.bind or "127.0.0.1:6669"
        host, port_s = bind.rsplit(":", 1)
        port = int(port_s)
        if args.wait:
            await asyncio.sleep(args.wait)
        print(f"console tcp probe {host}:{port} -> {probe_tcp(host, port)}")

        n = max(args.n, 50)
        rounds = max(args.rounds, 2)
        await drive_console_load(driftpyrs, n=n, sleep_s=args.sleep, rounds=rounds)
        hold = max(args.hold, 10.0)
        if hold:
            await asyncio.sleep(hold)
        return

    await test_basic_async(driftpyrs)
    await test_concurrent_async(driftpyrs)
    print("\n✓ All async bridge tests passed!")


if __name__ == "__main__":
    asyncio.run(main())
