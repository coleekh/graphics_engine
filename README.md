# wgpu Graphics Engine

A physically-based 3D graphics renderer written in Rust using **wgpu** and **winit**.

## Features

| Feature | Details |
|---|---|
| **PBR Shading** | Cook-Torrance GGX microfacet BRDF, metallic-roughness workflow |
| **Multiple light types** | Directional, point, and spot lights |
| **Keyboard navigation** | WASD / Arrow keys + Q/E to fly the camera |
| **Cross-platform** | Vulkan, Metal, DX12, DX11, WebGPU backends via wgpu |

---

## Quick Start

### Prerequisites

- Rust **1.75+** (`rustup update stable`)
- A GPU with Vulkan, Metal, DX11, or DX12 support

### Build & Run

```bash
cd graphics_engine
cargo run --release
```

---

## Controls

| Key | Action |
|---|---|
| **W / ↑** | Move camera forward |
| **S / ↓** | Move camera backward |
| **A / ←** | Strafe left |
| **D / →** | Strafe right |
| **Q** | Move up |
| **E** | Move down |
| **Esc** | Quit |

---

## Architecture

```
graphics_engine/
├── Cargo.toml
├── assets/
│   └── shaders/
│       └── pbr.wgsl          # Cook-Torrance GGX PBR shader
└── src/
    ├── main.rs               # Entry point, event loop, demo scene
    ├── scene/
    │   └── mod.rs            # Camera, Light, Material, SceneObject, MeshBuilder
    ├── renderer/
    │   ├── mod.rs            # Engine: owns GPU state, renders scenes
    │   ├── gpu.rs            # GpuContext: adapter/device/surface init
    │   ├── pipeline.rs       # PBR render pipeline creation
    │   ├── mesh_store.rs     # GPU buffer management (vertex + index)
    │   └── uniforms.rs       # CameraUniform, LightUniform, ObjectUniform
    └── utils/
        └── mod.rs            # Texture loader, FPS counter, Timer
```

### Key Data Flow

```
EventLoop → main.rs
    → update_scene()  (animate positions/rotations)
    → Engine::render(&scene)
          ├─ write CameraUniform → GPU buffer
          ├─ write LightsBlock   → GPU buffer
          └─ for each SceneObject:
                ├─ write ObjectUniform → per-draw GPU buffer
                └─ draw_indexed()  via pbr.wgsl
```

---

## Extending the Engine

### Add a texture to a material

Use `utils::load_texture()` and extend `ObjectUniform` + the WGSL shader to sample it.

### Add a new render pass (e.g. shadow maps)

1. Create a depth-only texture for each directional light.
2. Add a `shadow_pipeline` (vertex-only, no fragment).
3. Render all objects into the shadow map before the main pass.
4. Sample the shadow map in `pbr.wgsl` using `textureSampleCompare`.

---
