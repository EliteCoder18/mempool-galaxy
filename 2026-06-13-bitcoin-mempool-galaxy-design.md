# Bitcoin Mempool Galaxy — Design Spec

**Date:** 2026-06-13  
**Author:** Rishit Modi  
**Goal:** Real-time terminal visualizer of the Bitcoin mempool as a particle-physics galaxy. Built on BDK primitives. Resume signal + virality target.

---

## 1. Goals

- **Visual:** Terminal-rendered galaxy where each unconfirmed transaction is a particle. Fee rate encodes position (high-fee = bright core, low-fee = outer arms) and color (cold blue → white-hot). CPFP chains render as connected particles. Blocks "eat" stars with a nova animation.
- **Technical resume signal:** Custom terminal renderer (no TUI framework), parallel particle physics with rayon, Bitcoin Core RPC + ZMQ integration via BDK primitives, `bdk_chain::TxGraph` for CPFP dependency tracking.
- **Virality:** Ships with a `--demo` flag that replays a bundled recorded mempool session — no node required. README has a GIF. Anyone can `cargo install mempool-galaxy` and see it instantly.

---

## 2. Architecture

Five components communicating via channels and shared state:

```
┌─────────────────┐     ┌──────────────────────────────────┐
│  Data Pipeline  │────▶│  MempoolState (Arc<RwLock<_>>)   │
│  (tokio task)   │     │  txs: HashMap<Txid, TxEntry>     │
│                 │     │  graph: TxGraph<()>               │
│  Primary:       │     │  events: broadcast channel        │
│  Bitcoin Core   │     └──────────┬───────────────────────┘
│  RPC + ZMQ      │                │
│                 │       ┌────────▼────────┐   ┌────────────────┐
│  Fallback:      │       │ Physics Engine  │   │   Renderer     │
│  mempool.space  │       │ (std thread +  │──▶│ (crossterm +   │
│  WebSocket      │       │  rayon)        │   │  half-blocks)  │
└─────────────────┘       └─────────────────┘   └───────┬────────┘
                                                         │
                          ┌──────────────────────────────▼──────┐
                          │  Input Handler (crossterm events)    │
                          └─────────────────────────────────────┘
```

### Crate dependencies

| Crate | Role |
|---|---|
| `bdk_chain` | `TxGraph<()>` for CPFP ancestor/descendant tracking |
| `bitcoincore_rpc` | Bitcoin Core JSON-RPC (`getrawmempool`, `getmempoolentry`) |
| `bitcoin` | Core types: `Txid`, `Transaction`, `Amount`, `BlockHash` |
| `tokio` | Async runtime for data pipeline and input handler |
| `tokio-tungstenite` | WebSocket fallback connection to mempool.space |
| `tokio-zmq` | ZMQ subscriber for `rawtx` and `rawblock` topics from Bitcoin Core |
| `rayon` | Parallel force calculations in physics engine |
| `crossterm` | Terminal I/O, raw mode, color output |
| `serde` / `serde_json` | RPC and WebSocket message parsing |
| `clap` | CLI argument parsing |

---

## 3. Data Pipeline

### Connection manager (tokio task)

Tries sources in order at startup:

```
1. Bitcoin Core RPC (bitcoincore_rpc)
   - getrawmempool verbose=true   → bulk load on connect
   - ZMQ rawtx topic              → new txs in real time
   - ZMQ rawblock topic           → block arrivals

2. mempool.space WebSocket (on RPC failure)
   - {"action":"init"}            → initial snapshot
   - live tx/block stream
```

Connection status is broadcast to the HUD.

### MempoolState

```rust
struct TxEntry {
    txid:       Txid,
    fee_rate:   f64,      // sat/vB
    vsize:      u32,
    fee:        Amount,
    first_seen: Instant,
}

struct MempoolState {
    txs:      HashMap<Txid, TxEntry>,
    graph:    TxGraph<()>,     // bdk_chain — tracks spending relationships
    stats:    MempoolStats,    // count, total vsize, fee percentiles
}
```

`TxGraph<()>` stores `bitcoin::Transaction` objects and tracks which unconfirmed txs spend outputs of other unconfirmed txs. A single graph walk yields the full ancestor/descendant set for any tx — the CPFP cluster, ready to render as connected particles.

### Event bus

```rust
enum MempoolEvent {
    TxAdded(Txid),
    TxRemoved(Txid),                           // RBF replacement
    BlockArrived { height: u32, confirmed: Vec<Txid> },
}
```

Physics engine and renderer both subscribe via `tokio::sync::broadcast`.

---

## 4. Physics Engine

Runs on a dedicated `std::thread` (CPU-bound, not on the tokio executor). rayon parallelizes per-frame force calculations.

### Particle

```rust
struct Particle {
    txid:       Txid,
    pos:        Vec2,
    vel:        Vec2,
    fee_rate:   f64,
    vsize:      u32,
    cluster_id: Option<usize>,   // CPFP chain membership
}

struct CpfpCluster {
    indices:  Vec<usize>,   // particle indices in this chain
    centroid: Vec2,
}
```

### Force model (per frame, parallelized with rayon)

1. **Central gravity** — pulls every particle toward origin, strength ∝ `fee_rate`. High-fee txs orbit the bright core; low-fee txs drift to outer arms.
   `F = k_g × fee_rate / r²`

2. **CPFP spring** — particles in the same CPFP cluster attract each other with a spring at a fixed rest length. They render as visually connected; a parent tx and its high-fee child get pulled inward together — package relay made visible.
   `F = k_s × (r − rest_length)`

3. **Short-range repulsion** — prevents particle overlap.
   `F = k_r / r²` when `r < threshold`

4. **Velocity damping** — `vel *= 0.98` each frame to prevent chaos.

Physics constants (`k_g`, `k_s`, `k_r`) are tunable at runtime via config. Reasonable starting values: `k_g = 0.15`, `k_s = 0.08`, `k_r = 0.05`, `rest_length = 8.0` (in framebuffer pixel units).

New transactions spawn at the outer ring with a tangential velocity for an orbital feel, then drift inward according to their fee rate.

### Block arrival — nova effect

When `BlockArrived` fires:
- Confirmed particles receive a strong inward vacuum pull over ~0.5s
- Accelerate toward center, flash white
- Removed from particle list
- Surrounding particles receive a brief outward shockwave repulsion

### LOD (Level of Detail)

| Zoom | Mode | What simulates |
|---|---|---|
| `zoom ≥ 0.3` | Particles | Individual txs; CPFP links as lines |
| `zoom < 0.3` | Buckets | 7 fee-tier super-particles sized by total vsize |

Transition: 0.3s crossfade. Physics continues in both modes.

**Performance cap:** 10,000 particles max. When exceeded, lowest-fee txs are dropped (they sit at the outer edge — least visually critical). Spatial grid reduces force calculation from O(n²) to O(n×k). At 10k particles + rayon: stable 60fps physics.

---

## 5. Renderer

Physics at 60fps, render at 30fps — decoupled via double buffer.

### Framebuffer + half-blocks

Each terminal cell = 2 vertical pixels using Unicode half-block characters (`▀`, `▄`, `█`). Top and bottom get independent RGB via crossterm fg/bg.

```rust
struct Framebuffer {
    pixels: Vec<Color>,   // width × (height × 2)
    prev:   Vec<Color>,   // last frame for differential flush
    width:  usize,
    height: usize,
}
```

Flush rule per cell: same top/bottom → `█`; different → `▀` fg=top bg=bottom. Only cells that changed from `prev` are written — differential rendering cuts terminal I/O ~80% on stable frames and eliminates flicker.

### Star rendering

Each particle draws a gaussian glow falloff:

| Distance from center | Brightness |
|---|---|
| 0 | 100% |
| 1 | 55% |
| 2 | 18% |

`vsize` scales the halo radius. CPFP links are dim Bresenham lines between connected particles.

### Color encoding — fee rate as temperature

| sat/vB | Color | Feel |
|---|---|---|
| < 1 | `#222222` | dim dust |
| 1–5 | `#3355ff` | cool blue |
| 5–20 | `#00aaff` | cyan |
| 20–50 | `#00ff88` | green |
| 50–100 | `#ffee00` | yellow |
| 100–300 | `#ff8800` | orange |
| 300+ | `#ffffff` | white-hot |

During mempool fee spikes, the whole galaxy visually heats up as particles shift inward and the palette warms.

### Render pipeline (per frame)

1. Fill framebuffer black
2. Paint static background stars (fixed seed, decorative)
3. Paint particle glows back-to-front by fee rate (low-fee first — high-fee stars win overlaps)
4. Paint CPFP connecting lines
5. Flush diff to crossterm
6. Paint HUD in raw terminal cells on top (no half-blocks — text stays crisp)

---

## 6. HUD

**Top-left — mempool stats:**
```
mempool  247,413 txs  312 vMB
fees     p10:2  p25:8  p50:21  p75:67  p90:180  sat/vB
```

**Top-right — connection + chain:**
```
Bitcoin Core RPC  ●  block 898,241  14m ago
```
`●` green = connected, yellow = reconnecting, red = failed.

**Bottom — key bindings (dim):**
```
+/- zoom   arrows pan   p pause   r reset   d demo   q quit
```

**Block arrival banner (centered, fades after 3s):**
```
█  BLOCK 898,242  —  2,847 txs confirmed  █
```

---

## 7. Controls

| Input | Action |
|---|---|
| `+` / `-` | Zoom in / out (triggers LOD transition at threshold) |
| Arrow keys | Pan |
| Mouse scroll | Zoom |
| Mouse drag | Pan |
| `p` | Pause physics (galaxy freezes, data still streams) |
| `r` | Reset view to default zoom + center |
| `d` | Toggle demo mode (replay bundled recording) |
| `f` | Force WebSocket fallback (debug) |
| `q` / `Ctrl-C` | Quit |

---

## 8. CLI

```
mempool-galaxy [OPTIONS]

  --rpc-url   <URL>    Bitcoin Core RPC  [default: http://127.0.0.1:8332]
  --rpc-user  <USER>
  --rpc-pass  <PASS>
  --zmq-url   <URL>    ZMQ endpoint      [default: tcp://127.0.0.1:28332]
  --demo               Run bundled demo recording without a node
  --no-rpc             Skip to WebSocket fallback immediately
```

Config also loadable from `~/.config/mempool-galaxy/config.toml`.

---

## 9. Demo Mode

A separate binary or subcommand records a live mempool session to a compact binary log (txid + fee_rate + vsize + timestamp per event). The bundled recording ships in `assets/demo.mgl`. `--demo` replays it at real-time speed.

**`demo.mgl` format** — simple length-prefixed binary stream:
```
[u8 event_type][u32 timestamp_ms][u32 payload_len][u8* payload]
```
Event types: `0x01` TxAdded (txid + fee_rate + vsize), `0x02` TxRemoved (txid), `0x03` BlockArrived (height + confirmed txid list). Replay reads events and feeds them into `MempoolState` at original timing.

This is the virality mechanism: the README GIF is generated from demo mode. Anyone can `cargo install mempool-galaxy --features demo` and see the full visualization without running a Bitcoin node.

---

## 10. Crate Structure

```
mempool-galaxy/
├── src/
│   ├── main.rs
│   ├── pipeline/
│   │   ├── mod.rs        # connection manager, source selection
│   │   ├── rpc.rs        # Bitcoin Core RPC + ZMQ
│   │   └── websocket.rs  # mempool.space WS fallback
│   ├── state/
│   │   ├── mod.rs        # MempoolState, TxEntry
│   │   └── events.rs     # MempoolEvent broadcast
│   ├── physics/
│   │   ├── mod.rs        # engine thread, rayon dispatch
│   │   ├── particle.rs   # Particle, CpfpCluster
│   │   └── forces.rs     # gravity, spring, repulsion, damping
│   ├── renderer/
│   │   ├── mod.rs        # render loop, double buffer
│   │   ├── framebuffer.rs
│   │   ├── halfblock.rs  # unicode encoding
│   │   └── color.rs      # fee_rate → Color
│   ├── hud.rs
│   ├── input.rs
│   └── demo.rs           # record + replay
├── assets/
│   └── demo.mgl          # bundled recording
└── Cargo.toml
```

---

## 11. Non-goals (v1)

- No Tor/anonymity routing
- No historical mempool replay beyond the bundled demo
- No fee estimation or prediction
- No mobile / non-true-color terminal support
- No Windows support (crossterm handles it in theory but untested)
