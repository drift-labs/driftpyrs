# driftpyrs Async Architecture

This document explains how driftpyrs bridges Rust's Tokio async runtime with Python's asyncio. Understanding this architecture is essential for contributing to the project.

## The Core Problem

We have two async runtimes with fundamentally different execution models:

**Python's asyncio:** Cooperative multitasking on a single thread. The Global Interpreter Lock (GIL) ensures only one Python bytecode instruction executes at a time. When a coroutine awaits, it yields to the event loop, which picks another coroutine to run. All of this happens on one thread.

**Rust's Tokio:** A work-stealing scheduler that multiplexes tasks across a thread pool. When a Tokio task awaits, the scheduler might resume it on a different OS thread. This is real parallelism.

driftpyrs needs to let Python call into drift-rs, which is built entirely on Tokio. We can't just call Rust async functions from Python—we need to translate between two different models of concurrent computation.

## The Solution: Separation of Concerns

The architecture separates **state access** from **update notification**:

```
┌─────────────────────────────────────────────────────────────────┐
│                         Python Process                          │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                  Python Thread (GIL)                       │  │
│  │                                                            │  │
│  │    asyncio Event Loop                                      │  │
│  │    ┌────────────────────────────────────────────────────┐  │  │
│  │    │                                                    │  │  │
│  │    │  await client.subscribe()  ───► one-shot async op  │  │  │
│  │    │                                                    │  │  │
│  │    │  async for update in stream: ──► channel-backed    │  │  │
│  │    │                                  async iterator    │  │  │
│  │    │                                                    │  │  │
│  │    │  client.get_user("xyz")  ───► sync cache read      │  │  │
│  │    │                               (no await needed)    │  │  │
│  │    │                                                    │  │  │
│  │    └────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                               │                                  │
│                               │ PyO3 boundary                    │
│                               │                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                 Tokio Runtime (thread pool)                │  │
│  │                                                            │  │
│  │   ┌─────────────────────────────────────────────────────┐  │  │
│  │   │              Background Tasks                        │  │  │
│  │   │                                                      │  │  │
│  │   │   WebSocket Task ───► receives market updates        │  │  │
│  │   │         │                                            │  │  │
│  │   │         ▼                                            │  │  │
│  │   │   ┌──────────┐                                       │  │  │
│  │   │   │ DashMap  │ ◄─── concurrent cache                 │  │  │
│  │   │   │ (cache)  │      (lock-free reads)                │  │  │
│  │   │   └──────────┘                                       │  │  │
│  │   │         ▲                                            │  │  │
│  │   │         │                                            │  │  │
│  │   │   gRPC Task ─────► receives account updates          │  │  │
│  │   │                                                      │  │  │
│  │   └─────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key principle: Python never waits for Tokio, Tokio never waits for Python.** They operate on shared data, not shared control flow.

## Runtime Lifecycle

### 1. Module Import: Tokio Starts

When Python imports driftpyrs, the Tokio runtime initializes:

```rust
use pyo3::prelude::*;

#[pymodule]
fn driftpyrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Starts Tokio runtime with default thread pool
    // (typically one thread per CPU core)
    pyo3_async_runtimes::tokio::init_runtime()?;
    
    m.add_class::<DriftClient>()?;
    // ... register other functions/classes
    
    Ok(())
}
```

```python
import driftpyrs  # Tokio threads spawn here, sit idle waiting for work
```

### 2. Connection: First Async Work

When Python calls an async function, work gets scheduled onto Tokio:

```rust
#[pyfunction]
fn connect<'py>(py: Python<'py>, config: Config) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        // This block runs on a Tokio thread
        let client = drift_rs::DriftClient::new(config).await?;
        Ok(DriftClient { inner: Arc::new(client) })
    })
}
```

`future_into_py` does the bridging:

1. Takes the async block
2. Spawns it onto the Tokio runtime
3. Returns a Python awaitable
4. When the async block completes, signals Python's event loop

```python
client = await driftpyrs.connect(config)  # Awaits Tokio task completion
```

### 3. Subscription: Background Tasks Start

Subscribing spawns long-lived background tasks:

```rust
#[pymethods]
impl DriftClient {
    fn subscribe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            // This call internally spawns background tasks
            // that keep running after subscribe() returns
            inner.subscribe().await?;
            Ok(())
        })
    }
}
```

Inside drift-rs, `subscribe()` spawns tasks that run forever:

```rust
// Simplified drift-rs internals
impl DriftClient {
    pub async fn subscribe(&self) -> Result<()> {
        let cache = Arc::clone(&self.cache);
        
        // This task keeps running after subscribe() returns
        tokio::spawn(async move {
            let mut ws = connect_websocket().await?;
            loop {
                let msg = ws.recv().await?;
                cache.insert(msg.key, msg.value);  // Update shared cache
            }
        });
        
        Ok(())
    }
}
```

```python
await client.subscribe()  # Returns once connected
# Background tasks now running, updating cache continuously
```

### 4. Cache Reads: Synchronous Access

Python reads from the cache synchronously—no async, no waiting:

```rust
#[pymethods]
impl DriftClient {
    fn get_user(&self, py: Python<'_>, pubkey: &str) -> PyResult<Option<PyObject>> {
        // DashMap read is lock-free, returns instantly
        match self.inner.cache.get(pubkey) {
            Some(user) => Ok(Some(user.to_py_dict(py)?)),
            None => Ok(None),
        }
    }
}
```

```python
# Instant - reads from memory, doesn't wait for network or Tokio
user = client.get_user("ABC123")
```

This works because DashMap provides lock-free reads. Multiple threads can read simultaneously while other threads write. Python reads current state; Tokio updates it in the background.

### 5. Update Streams: Async Iteration

When Python needs to react to updates (not just read current state), we use channels:

```rust
#[pyclass]
pub struct UpdateStream {
    rx: Arc<tokio::sync::Mutex<mpsc::Receiver<AccountUpdate>>>,
}

#[pymethods]
impl UpdateStream {
    fn __aiter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        let rx = Arc::clone(&self.rx);
        
        Some(pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = rx.lock().await;
            match guard.recv().await {
                Some(update) => Ok(update.to_py_dict()),
                None => Err(pyo3::exceptions::PyStopAsyncIteration::new_err(())),
            }
        }))
    }
}
```

```python
stream = await client.subscribe_with_stream()

async for update in stream:
    # Genuinely awaits - event loop runs other tasks while waiting
    print(f"Got update: {update}")
```

This is **not** polling. Each `__anext__` call creates a future that suspends until data arrives. The Tokio side sends to the channel; pyo3-async-runtimes wakes Python's event loop.

## Timeline Visualization

```
Python                                   Tokio Runtime
──────                                   ─────────────

import driftpyrs
       │
       └──► init_runtime() ─────────────► [thread pool spawns, idle]


client = await connect(config)
       │
       └──► future_into_py() ───────────► [task: connect to RPC]
       │                                           │
       │         [Python event loop runs           │
       │          other coroutines]                │
       │                                           ▼
       ◄─────────────────────────────────  [task completes]


await client.subscribe()
       │
       └──► future_into_py() ───────────► [task: open WebSocket]
       │                                  [spawn background task]
       │                                           │
       │                                           ▼
       ◄─────────────────────────────────  [subscribe() returns]
                                                   │
                                          [background task loops forever]
                                          [receiving updates, writing cache]
                                                   │
                                                   ▼ (continuous)
                                                   
user = client.get_user("xyz")
       │
       └──► [sync read from DashMap] ───► [cache.get("xyz")]
       │                                           │
       ◄───────────────────────────────────────────┘
       
       (instant, no async, no waiting)
```

## Three Categories of Operations

### 1. Pure Functions (Sync)

No runtime involvement, no async:

```rust
#[pyfunction]
fn derive_user_account(authority: &str, sub_account_id: u16) -> PyResult<String> {
    let authority = parse_pubkey(authority)?;
    Ok(drift_rs::Wallet::derive_user_account(&authority, sub_account_id).to_string())
}
```

```python
address = driftpyrs.derive_user_account(authority, 0)
```

### 2. One-Shot Async (Request/Response)

Uses `future_into_py`, returns when complete:

```rust
#[pyfunction]
fn get_account<'py>(py: Python<'py>, rpc_url: String, pubkey: String) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let client = RpcClient::new(&rpc_url);
        let account = client.get_account(&pubkey).await?;
        Ok(account.to_py_dict())
    })
}
```

```python
account = await driftpyrs.get_account(rpc_url, pubkey)
```

### 3. Long-Lived Subscriptions

Spawns background tasks, optionally returns update stream:

```rust
#[pymethods]
impl DriftClient {
    /// Subscribe and get update stream
    fn subscribe_with_stream<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (tx, rx) = mpsc::channel(1000);
            
            inner.subscribe_with_callback(move |update| {
                // Runs on Tokio thread, no GIL needed
                let _ = tx.try_send(update.clone());
            }).await?;
            
            Ok(UpdateStream { 
                rx: Arc::new(tokio::sync::Mutex::new(rx)) 
            })
        })
    }
}
```

```python
stream = await client.subscribe_with_stream()

async for update in stream:
    process(update)
```

## Why Not Just Use Callbacks?

You might wonder: why not call Python directly from Rust when updates arrive?

```rust
// Tempting but problematic
inner.subscribe_with_callback(|update| {
    Python::with_gil(|py| {
        python_callback.call1(py, (update,))?;
    });
}).await?;
```

Problems:

1. **GIL contention:** Acquiring the GIL blocks all Python code. If updates arrive at 500/sec, that's 500 GIL acquisitions per second from Tokio threads—each one potentially blocking while Python reaches a yield point.

2. **No async callbacks:** The Rust callback is synchronous. You can't `await` inside it. So the Python callback can't do any I/O.

3. **Priority inversion:** Fast Rust code waits on slow Python code. Under load, this causes cascading delays.

The channel approach avoids all of this. Tokio sends to the channel (non-blocking, no GIL). Python receives from the channel (async, yields properly). The two runtimes stay decoupled.

## Practical Usage Patterns

### Pattern 1: Subscribe and Poll Cache

Best for: Trading bots, anything that runs a loop and checks state.

```python
async def trading_loop():
    client = await DriftClient.connect(config)
    await client.subscribe()
    
    while True:
        # Instant cache reads
        user = client.get_user(my_pubkey)
        markets = client.get_all_markets()
        
        # Your logic
        if should_trade(user, markets):
            await client.send_order(...)
        
        await asyncio.sleep(0.1)
```

### Pattern 2: React to Updates

Best for: Event-driven systems, logging, analytics.

```python
async def event_processor():
    client = await DriftClient.connect(config)
    stream = await client.subscribe_with_stream()
    
    async for update in stream:
        # Runs each time cache updates
        await log_to_database(update)
        await notify_websocket_clients(update)
```

### Pattern 3: Combined

```python
async def main():
    client = await DriftClient.connect(config)
    stream = await client.subscribe_with_stream()
    
    async def process_updates():
        async for update in stream:
            await handle_update(update)
    
    async def trading_loop():
        while True:
            user = client.get_user(my_pubkey)  # Instant read
            await maybe_trade(user)
            await asyncio.sleep(0.1)
    
    # Run both concurrently
    await asyncio.gather(
        process_updates(),
        trading_loop(),
    )
```

## Data Flow Summary

```
Network
   │
   ▼
┌─────────────────────────────────────────┐
│            Tokio Runtime                 │
│                                          │
│  WebSocket ──► deserialize ──► DashMap   │
│                                   │      │
│                                   │      │
│                    ┌──────────────┘      │
│                    │                     │
│                    ▼                     │
│              mpsc::channel ──────────────┼──► Python async iterator
│                                          │
└─────────────────────────────────────────┘
                     │
                     │ (sync read)
                     ▼
              Python get_user()
```

- **Network → DashMap:** Tokio tasks receive data, update cache
- **DashMap → Python:** Sync reads, instant, lock-free
- **Channel → Python:** Async iteration, proper await semantics

## Common Pitfalls

### Don't Block in Callbacks

If you do use sync callbacks (not recommended), keep them fast:

```python
# BAD - blocks everything
def on_update(update):
    time.sleep(1)
    requests.post(url, data=update)  # Blocking I/O!

# GOOD - fast, no I/O
def on_update(update):
    logger.debug(f"Update: {update.pubkey}")
```

### Don't Forget to Subscribe

Cache reads return `None` until you subscribe:

```python
client = await DriftClient.connect(config)

user = client.get_user("xyz")  # None! Not subscribed yet

await client.subscribe()

user = client.get_user("xyz")  # Now returns data
```

### Handle Channel Backpressure

The update channel has bounded capacity. If Python processes updates slower than they arrive, old updates are dropped:

```python
stream = await client.subscribe_with_stream()  # Channel capacity: 1000

async for update in stream:
    await slow_database_write(update)  # If this is too slow, updates drop
```

Solutions:

- Process updates faster
- Batch writes
- Increase channel capacity (trades memory for tolerance)
- Accept that stale updates can be dropped (often fine for market data)

## Testing

### Unit Tests (Rust)

Test async logic in isolation:

```rust
#[tokio::test]
async fn test_subscription_channel() {
    let (tx, mut rx) = mpsc::channel(10);
    tx.send(AccountUpdate::mock()).await.unwrap();
    assert!(rx.recv().await.is_some());
}
```

### Integration Tests (Python)

Test the full bridge:

```python
import pytest
import asyncio

@pytest.mark.asyncio
async def test_subscribe_and_read():
    client = await driftpyrs.DriftClient.connect(config)
    await client.subscribe()
    
    # Should have data now
    user = client.get_user(known_pubkey)
    assert user is not None

@pytest.mark.asyncio
async def test_update_stream():
    client = await driftpyrs.DriftClient.connect(config)
    stream = await client.subscribe_with_stream()
    
    # Should receive update within timeout
    update = await asyncio.wait_for(
        stream.__anext__(),
        timeout=5.0
    )
    assert update is not None
```

## Further Reading

- [PyO3 User Guide](https://pyo3.rs/)
- [pyo3-async-runtimes](https://github.com/PyO3/pyo3-async-runtimes)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [DashMap Documentation](https://docs.rs/dashmap/)
