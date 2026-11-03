# T014 — deterministic multi-monitor geometry and surface-lifetime records

**Status: PASS for platform-independent model; Win32 monitor enumeration and actual multi-monitor rendering remain pending.**

Added `rust/src/monitor.rs`: opaque stable `MonitorId`, signed virtual-desktop `MonitorBounds`, independent `SurfaceLifetime` per `MonitorRecord`, validated `MonitorTopology`, and `choose_index(draw)`.

The area-weighted selection intentionally mirrors the frozen C# behavior in `master`:

```csharp
var s = Screen.AllScreens.Select(screen => (double)screen.Bounds.Width * screen.Bounds.Height).ToArray();
s = s.Select(size => size / s.Sum()).ToArray();
var st = random.NextDouble();
foreach (var t in s) { st -= t; if (st <= 0) break; nextScreen++; }
```

No new easing, geometry offsets, sampling distributions or extra draw steps were substituted. A supplied deterministic `draw` in `[0,1)` chooses one monitor and the equality case returns the preceding monitor, matching `if (st <= 0)`. Invalid draws are rejected instead of risking an array out-of-range index. The identity and state remain per-monitor; the image queue is globally shared in later work.

**TDD evidence (2026-11-03):** `cargo test --locked --test monitor_geometry --target x86_64-unknown-linux-gnu` initially failed with E0432 (module `monitor` absent). After implementation, all 5 tests passed:

1. Negative-monitor origins and virtual desktop union
2. Exact area-weight boundary behavior
3. Empty/duplicate display IDs and zero geometry rejection
4. Invalid random draws
5. Independent surface generations/lifetimes

`cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings` passed.

**Remaining validation:** D3D11 per-monitor surfaces and actual display enumeration/refresh rates on WindowsEnv and real multi-monitor hardware are not implemented here. Golden `.NET MathF` equivalence for animation itself remains T031, separately frozen from T014.
