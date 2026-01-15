"""
Test DriftClient connection (one-shot async pattern).

This demonstrates:
1. Using future_into_py for one-shot async operations
2. Connecting to Drift protocol via drift-rs
3. Reading RPC URL from environment variable (for security)
"""

import asyncio
import os

import driftpyrs


async def test_client_connection():
    """Test that we can connect to Drift protocol."""
    print("Testing DriftClient connection...")

    # Get RPC URL from environment variable
    rpc_url = os.environ.get("RPC_URL")
    if not rpc_url:
        print("⚠ RPC_URL not set in environment")
        print("  Set it with: export RPC_URL='your-rpc-url'")
        print("  Or create a .env file (see .env.example)")
        print("  Using default public RPC (may be slow)...")
        rpc_url = "https://api.mainnet-beta.solana.com"

    print(f"  RPC URL: {rpc_url[:50]}...")

    # Connect (this is async - waits for connection to complete)
    client = await driftpyrs.DriftClient.connect(rpc_url, context="mainnet")

    print(f"✓ Connected: {client}")
    print(f"  Context: {client.context_name()}")
    print(f"  Perp markets: {client.get_perp_market_count()}")
    print(f"  Spot markets: {client.get_spot_market_count()}")
    print(f"  Perp market configs: {client.get_perp_market_configs()}")


async def test_devnet_connection():
    """Test connecting to devnet."""
    print("\nTesting devnet connection...")

    # Use devnet RPC
    rpc_url = "https://api.devnet.solana.com"
    print(f"  RPC URL: {rpc_url}")

    # Connect to devnet
    client = await driftpyrs.DriftClient.connect(rpc_url, context="devnet")

    print(f"✓ Connected to devnet: {client}")
    print(f"  Context: {client.context_name()}")
    assert client.context_name() == "devnet", "Should be devnet"


async def test_invalid_context():
    """Test that invalid context raises error."""
    print("\nTesting invalid context handling...")

    try:
        _ = await driftpyrs.DriftClient.connect(
            "https://api.mainnet-beta.solana.com", context="invalid"
        )
        assert False, "Should have raised ValueError"
    except ValueError as e:
        print(f"✓ Correctly raised ValueError: {e}")


async def test_concurrent_connections():
    """Test that multiple clients can connect concurrently."""
    print("\nTesting concurrent connections...")

    rpc_url = os.environ.get("RPC_URL", "https://api.mainnet-beta.solana.com")

    # Connect multiple clients concurrently
    clients = await asyncio.gather(
        driftpyrs.DriftClient.connect(rpc_url),
        driftpyrs.DriftClient.connect(rpc_url),
        driftpyrs.DriftClient.connect(rpc_url),
    )

    assert len(clients) == 3, "Should have 3 clients"
    for i, client in enumerate(clients):
        print(f"  Client {i + 1}: {client.get_perp_market_count()} perp markets")

    print("✓ Concurrent connections work")


async def main():
    await test_client_connection()
    await test_devnet_connection()
    await test_invalid_context()
    await test_concurrent_connections()
    print("\n✓ All DriftClient tests passed!")


if __name__ == "__main__":
    # Note: For production, use python-dotenv to load .env file
    # For testing, you can set RPC_URL manually:
    #   export RPC_URL='your-rpc-url'
    asyncio.run(main())
