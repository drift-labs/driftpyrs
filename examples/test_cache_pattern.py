"""
Test the cache pattern: sync reads, async background updates.

This demonstrates:
1. DashMap-based cache with lock-free concurrent access
2. Python reads synchronously (instant, no await)
3. Tokio background tasks update cache asynchronously
4. Separation of state access from update notification
"""

import asyncio
import time
import driftpyrs


async def test_cache_before_updates():
    """Test that cache is empty before starting background updates."""
    print("Testing cache before updates...")

    demo = driftpyrs.CacheDemo()

    # Before starting updates, cache should be empty
    assert demo.is_empty(), "Cache should be empty initially"
    assert demo.len() == 0, "Cache length should be 0"
    assert demo.get("counter") is None, "Counter should not exist yet"

    print("✓ Cache is empty before updates start")


async def test_cache_sync_reads():
    """Test that we can read from cache synchronously after starting updates."""
    print("\nTesting synchronous cache reads...")

    demo = driftpyrs.CacheDemo()

    # Start background updates (async operation)
    await demo.start_updates()

    # Wait a bit for updates to arrive
    await asyncio.sleep(0.3)

    # Now we can read synchronously - no await needed!
    val1 = demo.get("counter")
    assert val1 is not None, "Counter should exist after updates started"

    # The value is a string representation of a number
    counter1 = int(val1)
    assert counter1 >= 0, f"Counter should be non-negative: {counter1}"

    print(f"✓ Synchronous read works! Counter = {counter1}")


async def test_cache_values_update():
    """Test that cache values change as background task updates them."""
    print("\nTesting that cache values update over time...")

    demo = driftpyrs.CacheDemo()
    await demo.start_updates()

    # Wait for initial updates
    await asyncio.sleep(0.15)

    # Read first value (sync, instant)
    val1 = demo.get("counter")
    assert val1 is not None

    # Wait for more updates (background task updates every 100ms)
    await asyncio.sleep(0.25)

    # Read second value (sync, instant)
    val2 = demo.get("counter")
    assert val2 is not None

    # Values should have changed
    assert val1 != val2, f"Values should have changed: {val1} vs {val2}"
    assert int(val2) > int(val1), f"Counter should increase: {val1} -> {val2}"

    print(f"✓ Cache updates over time! {val1} -> {val2}")


async def test_no_await_for_reads():
    """Test that reads are truly synchronous (no await)."""
    print("\nTesting that reads don't require await...")

    demo = driftpyrs.CacheDemo()
    await demo.start_updates()
    await asyncio.sleep(0.15)

    # These are all synchronous - no await
    start = time.perf_counter()
    for _ in range(1000):
        _ = demo.get("counter")
    elapsed = time.perf_counter() - start

    # 1000 reads should be very fast (< 10ms) if truly synchronous
    assert elapsed < 0.01, f"Reads took too long: {elapsed:.3f}s (should be < 0.01s)"

    print(f"✓ 1000 reads completed in {elapsed*1000:.2f}ms (truly synchronous!)")


async def test_cache_methods():
    """Test cache utility methods."""
    print("\nTesting cache utility methods...")

    demo = driftpyrs.CacheDemo()
    await demo.start_updates()
    await asyncio.sleep(0.15)

    # Check length
    length = demo.len()
    assert length >= 1, f"Cache should have at least 1 entry: {length}"

    # Check keys
    keys = demo.keys()
    assert "counter" in keys, f"'counter' should be in keys: {keys}"

    # Check not empty
    assert not demo.is_empty(), "Cache should not be empty"

    # Clear and check
    demo.clear()
    assert demo.is_empty(), "Cache should be empty after clear()"
    assert demo.len() == 0, "Length should be 0 after clear()"

    print("✓ All cache methods work correctly")


async def test_multiple_instances():
    """Test that multiple CacheDemo instances are independent."""
    print("\nTesting multiple independent cache instances...")

    demo1 = driftpyrs.CacheDemo()
    demo2 = driftpyrs.CacheDemo()

    # Start both
    await asyncio.gather(demo1.start_updates(), demo2.start_updates())
    await asyncio.sleep(0.15)

    # Both should have data
    val1 = demo1.get("counter")
    val2 = demo2.get("counter")

    assert val1 is not None, "Demo1 should have data"
    assert val2 is not None, "Demo2 should have data"

    # Clear one, other should be unaffected
    demo1.clear()
    assert demo1.is_empty(), "Demo1 should be empty"
    assert not demo2.is_empty(), "Demo2 should still have data"

    print("✓ Multiple instances are independent")


async def main():
    await test_cache_before_updates()
    await test_cache_sync_reads()
    await test_cache_values_update()
    await test_no_await_for_reads()
    await test_cache_methods()
    await test_multiple_instances()
    print("\n✓ All cache pattern tests passed!")


if __name__ == "__main__":
    asyncio.run(main())
