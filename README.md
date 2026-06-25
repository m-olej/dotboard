# Dotboard
> Don't OverThink dashboard

Mind organization tool, implemented as an extendable TUI dashboard.

## Core elements

1) **View**:
    - Defines a single view in TUI (outside of static application headers, etc..)
    - Defines layout configuration (based on a grid system)
    - Assigns modules to each *block* of the layout
2) **Module**:
    - Core element of the system
    - Defines custom *element* implementations
    - Defines *element* layout configuration
    - Owns the runtime inherited by all child *elements*
    - Defines *element* data pipelines (based on relations e1 -> e2 -> e3. )
3) **Component**:
    - Defines interaction behavior: static, scrollable, selectable, extendable, input
    - Can be assigned data: statically, dynamically (via tasks)
    - Responsible for formatting data
4) **Task**:
    - Functional *element* in the context of the system
    - Can be ran: synchronously, asynchronously, periodically (in the background)

## Core application layers

1) **API layer** (external system integrations)

2) **Automation layer** (external scripts integrations)

3) **TUI layer** (visual representation definition)

4) **Configuration layer** (Application configuration definitions)

5) **Application core layer** (core app engine integrations)

  - asynchronous execution engine (task offloading)

  - notification engine (system notifications)

  - memory engine (persistant and dynamic memory)

  - daemon engine (background tasks)

### Api & Automation layers

Generic wrappers that handles integration of externally executed logic into the applications core
Implementation logic is wrapped in a Generic that allows integration into application elements (components, tasks)

**API**: Abstracts networking, schemas, authorization from application elements. 
**Scripts**: Abstracts system access, script running, pre-post procesessing from application elements. 


### TUI layer

Ready to use collection of components, interaction behavior methods and data passing interface

### Configuration layer

File format parsers (yaml, json, toml), configuration definitions and integration into core application logic.

### Application core layers

Core application logic that is abstracted and used by all other elements. Implements complex logic and low-level operations that allow to integrate directly with the operating system.

#### Asynchronous engine

Using the asynchronous runtime that application runs on to delegate tasks and return constructs that allow elements to react to changes in task execution (async generator pattern)

#### Notification engine

Scheduling notifications (notify-send) to the operating system from the application.

#### Memory engine

Abstracts managing non operational memory management that is saved to the disk (temporary and persistant)

#### Daemon engine

Scheduling periodic execution of tasks that can be ran when TUI isn't running

# Application architecture overview

Outlining the architecture for a hyper-minimal, blazing-fast Terminal User Interface (TUI) dashboard tailored for Wayland/Hyprland environments. 

To achieve sub-millisecond startup times and isolate heavy computation from the rendering thread, the system utilizes a strict **Client-Server Architecture** communicating via Inter-Process Communication (IPC). 
The architecture is fundamentally Linux-native, heavily exploiting kernel-level features like Anonymous Shared Memory (`memfd`) and socket ancillary data (`SCM_RIGHTS`) to achieve zero-copy state initialization.

---

## 1. High-Level Architectural Paradigm

The system is bifurcated into two distinct binaries:

* **The Daemon (`dashboard-daemon`):** A persistent, stateful background server managed by `systemd`. It handles all asynchronous execution, memory management, OS integrations, and state mutations.
* **The Client (`dashboard-tui`):** A stateless, immediate-mode rendering client. It performs zero disk I/O, holds no persistent state, and acts solely as a visual translation layer for the memory map and delta stream provided by the daemon.

---

## 2. Low-Level OS Integration (The IPC Bridge)

The bridge between the daemon and the TUI is designed to eliminate serialization overhead during the client's critical boot path. 

### Zero-Copy State Initialization
When the TUI client boots, it requires the current application state to render the first frame. Passing this entire state over a socket buffer requires serialization, transmission, and deserialization—consuming valuable CPU cycles. 

Instead, the system utilizes **Anonymous Shared Memory**:
1.  **Memory Allocation:** The daemon utilizes the `memfd_create` syscall to allocate a RAM-backed file descriptor that does not exist on the virtual filesystem.
2.  **Serialization:** The daemon maintains a serialized "Materialized View" of the current UI state within this memory region.
3.  **FD Passing:** When the TUI connects via the Unix Domain Socket (`AF_UNIX`), the daemon transmits the file descriptor of the `memfd` to the client using `SCM_RIGHTS` (ancillary socket messages).
4.  **Memory Mapping:** The TUI client receives the file descriptor and uses the `mmap` syscall to map that memory directly into its own process address space. The first frame is rendered instantly from this pointer.

### Delta Event Stream
Once the initial state is mapped, maintaining synchronization is handled via a continuous stream of lightweight Deltas sent over the same Unix Domain Socket.
* Deltas are strictly targeted (e.g., `UpdateComponent(ModuleID, ComponentID, NewData)`).
* The connection is framed using a length-delimited codec to ensure the TCP-like continuous byte stream of the UDS is parsed into discrete, predictable message blocks.

---

## 3. Daemon (Server) Architecture

The daemon is the cognitive center of the application. It is structured around an asynchronous runtime (`tokio`) to ensure high-throughput processing without blocking the core event loop.

### Processing Cohesiveness
To prevent lock contention (e.g., heavily relying on `Arc<Mutex<State>>` across dozens of threads), the daemon utilizes an **Actor-inspired Message Passing** model. 

* **The Core State Manager:** A single, dedicated asynchronous task owns the canonical application state and the `memfd` buffer. It does not share memory with worker tasks.
* **Worker Tasks:** The asynchronous engine spawns isolated tasks for fetching APIs, running shell scripts, or awaiting OS signals.
* **Mutation Flow:** When a worker task completes, it sends a message (via `tokio::sync::mpsc`) to the Core State Manager. The State Manager updates the `memfd` buffer and broadcasts the corresponding Delta to all connected TUI clients.

### Sub-Engines
* **Asynchronous Engine:** The `tokio` runtime scheduler, delegating blocking I/O and heavy computation to background thread pools.
* **Memory Engine:** Manages the serialization of the View Model into the `memfd` buffer. Can optionally persist configuration or cache data to disk upon daemon shutdown.
* **Daemon Engine:** A cron-like scheduler (`tokio::time::interval`) that triggers periodic tasks regardless of whether a TUI client is currently active.
* **Notification Engine:** Integrates with D-Bus to dispatch system notifications for critical task completions or alerts.

---

## 4. TUI (Client) Architecture

The client is optimized entirely for rapid boot and high frame rates. 

### Boot Sequence
1.  Initialize raw terminal mode.
2.  Open `/run/user/1000/dashboard.sock`.
3.  Receive FD via `SCM_RIGHTS`.
4.  `mmap` the file descriptor.
5.  Render the initial frame directly from the mapped memory using zero-copy deserialization.
6.  Enter the non-blocking event loop.

### Rendering Loop
The TUI merges standard user input (keystrokes, mouse events) with the asynchronous Delta stream from the socket. 
* **Input Handling:** User inputs are captured and immediately flushed down the socket to the daemon as `Action` requests. The TUI does not mutate its own state based on input; it waits for the daemon to process the action and send back a Delta.
* **Visual Translation:** The visual layout is strictly driven by the data mapped in memory and updated by Deltas, utilizing an immediate-mode paradigm to paint the terminal buffer frame by frame.

---

## 5. Rust Crate Ecosystem

To implement this architecture effectively, the following libraries are considered:

* **`tokio`**: The foundational asynchronous runtime for the daemon, handling UDS networking, task spawning, and MPSC channels.
* **`rustix` or `nix`**: Provides safe Rust bindings for low-level POSIX and Linux-specific syscalls, strictly required for `memfd_create`, `mmap`, and `SCM_RIGHTS` FD passing.
* **`rkyv`**: A blazing-fast, zero-copy deserialization framework. Essential for structuring the data inside the `memfd` buffer so the TUI can read it without allocating new memory.
* **`ratatui`**: The premier Rust library for building complex, modular terminal user interfaces.
* **`crossterm`**: For low-level terminal manipulation (raw mode, capturing input events).
* **`tokio-util`**: Specifically for `LengthDelimitedCodec`, simplifying the framing of the continuous Unix Domain Socket byte stream into discrete Delta messages.
* **`serde` & `toml`**: For strict, one-time configuration parsing on daemon boot.
* **`notify-rust`**: For seamless integration with the desktop environment's notification daemon via D-Bus.

# Architectural Optimizations: Memory & Synchronization

## 1. The "Sealed Bootstrapper" Synchronization Pattern

To achieve a zero-copy, instant boot sequence for the TUI client, the daemon must provide the initial application state (the Materialized View) without serialization overhead on the socket. 

Maintaining a single, concurrently shared memory map (`memfd`) between the daemon and the TUI introduces the risk of **data tearing** (the client reading partially updated state) unless heavy IPC locks are used, which would bottleneck the daemon.

To avoid this, the architecture implements the **Sealed Bootstrapper** pattern. The daemon manages state entirely in its local heap and only utilizes shared memory dynamically during a client boot request.

### 1.1 The Boot Request Flow (From the Ground Up)

1. **Local State Mastery:** The daemon maintains the canonical application state in local, standard heap memory. It processes background tasks and updates this state without any IPC locking constraints.
2. **Client Connection:** The TUI client executes and opens the Unix Domain Socket (`AF_UNIX`). It sends an initial `SyncRequest`.
3. **On-Demand Allocation:** The daemon issues a `memfd_create` syscall, requesting the kernel to allocate a temporary, anonymous RAM-backed file descriptor.
4. **Zero-Copy Serialization:** The daemon serializes its current heap state directly into the `memfd` buffer using the `rkyv` crate. 
5. **The Kernel Seal:** Before transmitting the file descriptor, the daemon issues an `fcntl` syscall to apply `F_SEAL_WRITE` and `F_SEAL_SHRINK` to the memory. This guarantees at the kernel level that the memory is now strictly read-only and immutable.
6. **FD Transmission:** The daemon passes the sealed file descriptor over the socket using `SCM_RIGHTS` (ancillary data) and immediately drops its own write access.
7. **Client Mapping:** The TUI client receives the descriptor, issues an `mmap` syscall to map the RAM into its virtual address space, and renders its first frame mathematically guaranteed free of race conditions.
8. **Stream Handover:** Once mapped, the client listens to the socket for the ongoing Delta stream to update its local representation. The kernel automatically reclaims the `memfd` memory the moment the TUI process exits.

---

## 2. Daemon-Side Memory Optimization Best Practices

Because the daemon trades memory complexity for computational speed, strict memory management is required to prevent unbound RAM growth and heap fragmentation during continuous background operation.

### 2.1 Single-Threaded Asynchronous Runtime
By default, the `tokio` runtime utilizes a multi-threaded work-stealing scheduler, allocating worker threads and separate thread stacks for every logical CPU core. For an I/O-bound daemon orchestrating API calls and IPC sockets, this is massive overhead.

**Implementation:**
The daemon must be strictly constrained to a **Current-Thread Runtime**. This executes all asynchronous tasks concurrently on a single OS thread, eliminating cross-thread synchronization locks and drastically reducing the baseline memory footprint.

### 2.2 The Flyweight Pattern (Data vs. Presentation)

Storing pre-formatted strings (e.g., `String::from("ToDo Progress: 4/5")`) in the Materialized View causes rapid heap allocations and fragmentation every time a background tick occurs.

**Implementation:**
The daemon's state tree must be entirely data-driven. It stores and transmits only raw primitives (`struct ToDoState { total: u8, complete: u8 }`). The responsibility of string interpolation and visual formatting is offloaded entirely to the TUI client during the render loop.

### 2.3 String Interning for Static Identifiers

A modular UI requires tracking hundreds of static `String` identifiers (Module IDs, Component classes, API endpoints). Storing these as standard Rust `String` objects duplicates the exact same byte arrays across the heap.

**Implementation:**
Utilize a string interning library (such as `ustr` or `string_cache`). Interning allocates the string byte-data exactly once in a global pool. The daemon's state tree stores only lightweight, integer-sized pointers to this pool, shrinking the structural memory footprint and turning string comparisons into O(1) integer checks.

### 2.4 Arena Allocation (Flattening the State Tree)

Deeply nested architectures (where a `View` owns a `Box<Module>`, which owns a `Vec<Component>`) scatter application state unpredictably across the heap, destroying CPU cache locality and adding pointer overhead.

**Implementation:**
Implement an Entity Component System (ECS) or Arena pattern (using crates like `slotmap` or `bumpalo`). All components are stored in a single, contiguous memory array. Structural relationships are mapped using lightweight `u32` indices rather than deep pointers. This ensures sequential memory access when the daemon iterates over the state to serialize the `memfd` buffer, maximizing CPU cache hits.

### 2.5 Bounded LRU Caching

The daemon's `MemoryEngine` handles caching for external API payloads and shell script outputs. Allowing standard `HashMap` structures to grow indefinitely creates a persistent memory leak.

**Implementation:**
All dynamic state caches must utilize a strictly bounded Least Recently Used (LRU) cache or enforce Time-To-Live (TTL) eviction policies (using a crate like `moka`). This imposes a hard mathematical ceiling on the daemon's RAM usage, guaranteeing stability regardless of system uptime.
