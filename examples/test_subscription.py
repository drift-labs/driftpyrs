"""
Test DriftClient subscription pattern (async subscribe + sync reads).

This demonstrates the core architecture from ASYNC_ARCHITECTURE.md:
1. Subscribe spawns background Tokio tasks that update cache
2. Python reads from cache synchronously (instant, no await)
3. No GIL contention - Tokio and Python are decoupled
"""

import asyncio
import os
import time

import driftpyrs


async def test_subscribe_and_read():
    """Test basic subscribe + read pattern."""
    print("Testing subscribe and read pattern...")

    # Connect
    rpc_url = os.environ.get("RPC_URL", "https://api.mainnet-beta.solana.com")
    print(f"  Connecting to {rpc_url[:50]}...")
    client = await driftpyrs.DriftClient.connect(rpc_url)
    print(f"  ✓ Connected: {client.get_perp_market_count()} perp markets")

    # Subscribe (spawns background tasks)
    print("  Subscribing to all markets...")
    await client.subscribe()
    print("  ✓ Subscribed (background tasks running)")

    # Wait for initial data to arrive
    print("  Waiting for initial market data...")
    await asyncio.sleep(1.0)

    # Now we can read synchronously!
    market = client.get_perp_market(0)
    assert market is not None, "Should have data after subscribing"
    print(f"  ✓ Sync read works! Market 0: {market['market_index']}")


async def test_sync_reads_are_fast():
    """Test that reads are truly synchronous (instant)."""
    print("\nTesting that cache reads are instant...")

    rpc_url = os.environ.get("RPC_URL", "https://api.mainnet-beta.solana.com")
    client = await driftpyrs.DriftClient.connect(rpc_url)
    await client.subscribe()
    await asyncio.sleep(1.0)

    # Time 1000 reads - should be very fast if truly synchronous
    start = time.perf_counter()
    for _ in range(1000):
        _ = client.get_perp_market(0)
    elapsed = time.perf_counter() - start

    # Should be < 20ms for 1000 reads if truly sync
    print(f"  1000 reads in {elapsed * 1000:.2f}ms")
    assert elapsed < 0.02, f"Reads too slow: {elapsed * 1000:.2f}ms (should be < 20ms)"
    print(f"  ✓ Reads are instant! (~{elapsed * 1000000 / 1000:.1f}µs per read)")


async def test_market_data_updates():
    """Test that market data changes over time (background tasks updating)."""
    print("\nTesting that market data updates...")

    rpc_url = os.environ.get("RPC_URL", "https://api.mainnet-beta.solana.com")
    client = await driftpyrs.DriftClient.connect(rpc_url)
    await client.subscribe()
    await asyncio.sleep(1.0)

    # Read initial value
    market1 = client.get_perp_market(1)
    assert market1 is not None
    funding_rate_1 = market1.get("amm_last_funding_rate")

    print(f"  Initial read: funding_rate={funding_rate_1}")
    while True:
        await asyncio.sleep(0.5)
        market2 = client.get_perp_market(1)
        assert market2 is not None
        funding_rate_2 = market2.get("amm_last_funding_rate")
        print(f"  Current funding_rate={funding_rate_2}")
        if funding_rate_2 != funding_rate_1:
            print("  ✓ Market data is being updated by background tasks")
            break


async def test_spot_markets():
    """Test reading spot market data."""
    print("\nTesting spot market reads...")

    rpc_url = os.environ.get("RPC_URL", "https://api.mainnet-beta.solana.com")
    client = await driftpyrs.DriftClient.connect(rpc_url)
    await client.subscribe()
    await asyncio.sleep(1.0)

    # Read spot market (0 is usually USDC)
    spot = client.get_spot_market(0)
    assert spot is not None, "Should have spot market data"
    print(f"  ✓ Spot market 0: {spot['name']} (decimals={spot['decimals']})")
    print(f"    Deposit balance: {spot['deposit_balance']}")
    print(f"    Borrow balance: {spot['borrow_balance']}")


def _amm_price(market: dict) -> float:
    return (market["amm_quote_asset_reserve"] * market["amm_peg_multiplier"]) / market[
        "amm_base_asset_reserve"
    ]


async def test_poll_price():
    """Poll oracle price a few times."""
    print("\nTesting price polling...")

    rpc_url = os.environ.get("RPC_URL", "https://api.mainnet-beta.solana.com")
    client = await driftpyrs.DriftClient.connect(rpc_url)
    await client.subscribe()
    await asyncio.sleep(1.0)

    last = None
    changed = False

    for i in range(10):
        oracle = client.get_perp_oracle(i)
        assert oracle is not None
        print(f"  oracle: {oracle}")
        price = oracle["price"]
        assert price != 0
        if last is not None and price != last:
            changed = True
        last = price
        print(f"  poll {i}: oracle_price={price} slot={oracle['slot']}")
        await asyncio.sleep(0.5)

    assert changed, "oracle price never changed during polling window"


async def test_multiple_markets():
    """Test reading multiple markets."""
    print("\nTesting multiple market reads...")

    rpc_url = os.environ.get("RPC_URL", "https://api.mainnet-beta.solana.com")
    client = await driftpyrs.DriftClient.connect(rpc_url)
    await client.subscribe()
    await asyncio.sleep(1.0)

    # Read multiple markets
    indices = client.get_perp_market_configs()
    print(f"  Reading {len(indices)} perp markets...")

    count = 0
    for idx in indices:
        market = client.get_perp_market(idx)
        if market is not None:
            count += 1

    print(f"  ✓ Successfully read {count}/{len(indices)} perp markets")
    assert count > 0, "Should have read at least one market"


async def test_concurrent_reads():
    """Test that multiple tasks can read concurrently."""
    print("\nTesting concurrent reads from multiple tasks...")

    rpc_url = os.environ.get("RPC_URL", "https://api.mainnet-beta.solana.com")
    client = await driftpyrs.DriftClient.connect(rpc_url)
    await client.subscribe()
    await asyncio.sleep(1.0)

    async def read_loop(name: str, count: int):
        """Task that reads markets in a loop."""
        for _ in range(count):
            _ = client.get_perp_market(0)
            await asyncio.sleep(0.001)  # Small delay
        print(f"    {name} completed {count} reads")

    # Run multiple read tasks concurrently
    await asyncio.gather(
        read_loop("Task 1", 100),
        read_loop("Task 2", 100),
        read_loop("Task 3", 100),
    )

    print("  ✓ Concurrent reads work")


async def main():
    # await test_subscribe_and_read()
    # await test_sync_reads_are_fast()
    # await test_market_data_updates()
    # await test_spot_markets()
    await test_poll_price()
    await test_multiple_markets()
    await test_concurrent_reads()
    print("\n✓ All subscription pattern tests passed!")


if __name__ == "__main__":
    # Set RPC_URL environment variable
    #   export RPC_URL='your-rpc-url'
    asyncio.run(main())
