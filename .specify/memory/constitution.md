# YUAUY Wallpaper Constitution

## Core Principles

### I. Preserve the Proven C# Behavior (NON-NEGOTIABLE)

The historical `master` branch is a read-only behavioral reference. The Rust rewrite MUST
preserve the observable animation and multi-monitor behavior already validated there,
including the original hyperbolic-cosine time functions, four animation phases, translation,
rotation, scaling, alpha/cover sequence, image sizing and placement, monitor-area-weighted
selection, independent per-monitor images, and the existing startup/transition cadence.
No easing substitution, normalization of the formulas, or silent redesign of selection logic
is permitted. When `YUAUY_Wallpaper/Form1.cs` and `YUAUY_Wallpaper/Program.cs` differ,
implementation MUST document the difference and seek clarification instead of inventing
which version is authoritative. The C# reference and `directx_build` branch MUST NOT be edited.

### II. Rust and Direct3D on Windows 11

The new product MUST be implemented in Rust, targeting Windows 11 24H2 and newer. Rendering
SHOULD use Direct3D 11 unless evidence and an explicit approved specification justify a
change. Windows shell/WorkerW integration, monitor enumeration, DPI/scaling, resource
recreation after display changes, and clean shutdown MUST be designed and tested as
independent concerns. The unfinished C++ `directx_build` branch MAY be consulted, but MUST
NOT supersede behavior established by `master`.

### III. Complete App with System Tray and YAML Configuration

The shipped product MUST operate as a complete Windows wallpaper application with a system
tray control surface and YAML configuration file. Configuration MUST be validated before
application; invalid YAML or unsafe values MUST produce a useful error and MUST NOT corrupt
previously valid settings. User-specific absolute picture paths found in historical source
MUST NOT be baked into the released executable. Specific tray commands, defaults, and
configuration schema are specified through SDD and MUST NOT be assumed from prototypes.

### IV. Resource-Aware Scheduling Without Animation Drift

The application MUST support a configurable resource-load guard: when a chosen utilization
metric exceeds its configured high-load rule, it MUST defer starting the next picture
transition and retain the currently displayed desktop image. At later measurements it
MUST be able to resume once the configured recovery rule is satisfied. This guard MUST
remain outside the preserved animation mathematics: load sensing and scheduling MAY gate
when a transition begins, but MUST NOT alter its curves, monitor selection algorithm,
or internal animation sequence. GPU utilization is the intended metric on real hardware;
CPU utilization and a software-rendering path MUST support GPU-less WindowsEnv testing.
Exact metric scope, thresholds, sampling, and hysteresis require a feature specification.

### V. Testable Equivalence and Explicit Evidence

Every behavior-protection or scheduling requirement MUST be represented by verifiable tests
or reproducible test procedures. Unit tests MUST cover animation formula samples, monitor
selection and geometry, image sequence, and utilization-gate state transitions. WindowsEnv
(Windows 11 25H2) MUST be used for the available Windows integration and CPU/software
rendering tests; a successful CPU/WARP run MUST NOT be presented as proof of physical GPU
compatibility or hardware utilization detection. Physical GPU and multi-monitor acceptance
MUST be separately demonstrated or explicitly reported as unverified before release.

## Platform and Configuration Constraints

- Preserve historical branches `master` and `directx_build`; all new development originates
  from `rust_build` and feature branches or PRs derived from that work.
- The original C# implementation is a behavioral reference, not a dependency of the Rust
  executable. Newly added GPU/CPU load gating is an approved extension to legacy behavior.
- On Windows systems without usable graphics hardware, use a supported Direct3D software
  route (e.g., WARP) to validate rendering and UI integration where feasible. Availability
  must be verified on the actual testing machine.
- Never require users to disclose or commit local photo collections, personal YAML
  configurations, machine paths, tokens, or private logs.

## Quality and Delivery Gates

- Follow GitHub Spec Kit: constitution -> specify -> clarify -> plan -> tasks -> implement
  -> analyze/verification. Do not treat unresolved product requirements as implemented.
- Before implementing each feature, record its observable behavior, acceptance criteria,
  and reference behavior (when applicable). Use small, reviewable changes on the Rust workline.
- Test deterministic math/selection behavior without a GPU, and isolate Windows hardware
  and shell interactions behind testable interfaces.
- Verify behavior on WindowsEnv and record Windows build, rendering path, test outcome,
  and any limitations. Require a separate physical-GPU and representative multi-monitor
  acceptance gate before claiming full production validation.
- A change that knowingly deviates from the legacy animation or multi-monitor semantics
  requires explicit owner approval and an updated specification; refactoring alone is not
  permission to change behavior.

## Governance

This constitution takes precedence over informal implementation preferences, exploratory
branches, and automated agent suggestions. Changes to it MUST be proposed with rationale,
impact on specifications/tests, and project-owner approval. Apply semantic versioning:
MAJOR for incompatible removal/redefinition of principles, MINOR for adding or materially
expanding principles, PATCH for non-semantic clarification. Before implementation/PR review,
check compliance with all five principles; record any exception and its explicit approval.

**Version**: 1.0.0 | **Ratified**: 2026-10-31 | **Last Amended**: 2026-10-31
