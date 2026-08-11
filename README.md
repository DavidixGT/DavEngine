\# Custom WebGPU Vector Engine Architecture



A modular, hardware-accelerated 2D/3D graphics engine architecture built completely from scratch using \*\*Rust\*\*, \*\*WebGPU (`wgpu`)\*\*, and the \*\*Vulkan\*\* backend. 



Instead of relying on heavy monolithic engines, this project implements a highly decoupled, data-driven framework built on clean object-oriented and systems engineering principles.



\---



\## 🏗️ Project Architecture



The engine pipeline is split into separate, single-responsibility layers to isolate pure game logic from underlying GPU driver controls:



\*   \*\*`src/main.rs`\*\* \*(Entry Point)\*: The minimalist application initializer. Registers project files as modules and triggers the high-level framework runner.

\*   \*\*`src/triangle.rs`\*\* \*(Application Context)\*: Interfaces with `winit` to handle the native operating system window lifecycle (resizing, desktop close hooks) and contains the central game trait definition.

\*   \*\*`src/renderer.rs`\*\* \*(Hardware Context)\*: The core graphics backend. Initializes GPU adapters, configures swapchain buffers, and implements flash-free batch pass rendering (`render\_scene`) to completely eliminate display flickering.

\*   \*\*`src/material.rs`\*\* \*(Pipeline Allocator)\*: Handles runtime WGSL shader compilation, memory bind groups, and allocates a flexible VRAM uniform configuration block (`Shader`).

\*   \*\*`src/mesh.rs`\*\* \*(Geometry Data)\*: The independent object container. Encapsulates raw vector position coordinates (`\[\[f32; 2]; 3]`) and exposes a clean `.draw()` abstraction.

\*   \*\*`src/game.rs`\*\* \*(Gameplay Manager)\*: The runtime playground. Uses `init()` to instantiate entities on startup and a continuous loop `update()` function to drive game math, stream uniform payloads, and request frame repaints.



\---



\## 🛠️ Key Technical Features



\### 1. Inversion of Geometry Mapping (Pure Vector Pipeline)

Stripped out rigid, hardcoded CPU vertex structures. The drawing functions ingest raw, flexible multi-dimensional vector arrays (`\[\[f32; 2]; 3]`) directly. Vertex formatting and aesthetic behaviors are decoupled and processed globally on the hardware.



\### 2. Universal Uniform Interfacing (`update\_shader\_buffer`)

Engineered a generic uniform buffer loader using byte-casting macros (`bytemuck`). This allows the application logic to pack any arbitrary, padded dataset layouts (such as running time variables, scales, speeds, or matrices) and stream them into any target WGSL shader with a single method call.



```rust

// Easily pass any padded data structure directly from game loops to VRAM

renderer.update\_shader\_buffer(\&self.custom\_shader, \&current\_frame\_data);

```



\### 3. Flicker-Free Closed Batch Rendering

Bypassed the standard multiple single-buffer update loops that crash frame synchronization. By wrapping drawing calls inside an execution closure block pattern, the engine opens exactly one render pass, attaches all visible meshes sequentially, and presents the entire finished canvas to the monitor stream in a single hardware operation.



\---



\## 🚀 Getting Started



\### Prerequisites

\*   \[Rust toolchain](https://rust-lang.org) installed.

\*   Graphics drivers supporting Vulkan, DirectX 12, or Metal.



\### Installation \& Run

1\. Clone the repository:

&#x20;  ```bash

&#x20;  git clone https://github.com

&#x20;  cd wgpu\_triangle

&#x20;  ```

2\. Build and run the project locally:

&#x20;  ```bash

&#x20;  cargo run

&#x20;  ```



\### Sharing the Executable

To compile an optimized, standalone production executable without console terminal debugging hooks, build the release profile:

```bash

cargo build --release

```

The resulting standalone `.exe` can be found inside the `\\target\\release\\` directory.



