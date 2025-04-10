# Eclipse OpenBSW - Rust integration proof of concept

This is a comprehensive update to integrate a Rust async runtime using the Embassy framework into OpenBSW. The changes include:

1. **Adding support for async execution of Rust code (`openbsw-runtime`, `openbsw-timer`).**
2. **Adding plumbing libraries for console output (`openbsw-console-out`), panic handling (`openbsw-panic-handler`), and logging (`openbsw-logger`).**
3. **Showcasing all the provided functionality in the `rustHelloWorld` crate.**
4. **Exploring interoperability between C++ and Rust with bindgen, cbindgen, direct linking to extern functions, and integrating into CMake with Corrosion.**

### Concept outline

#### General level of integration
Rust and C++ interaction is not trivial. However, there is no way around it if one wants to extend existing and proven C++ code bases with Rust.

Taking into account the constrained environment of microcontrollers in which OpenBSW operates, several different layers of separation come to mind.

- **Dedicated core**: On multicore chips, one entire core could be dedicated to Rust. This core could talk to C++ code via mechanisms like shared memory.
- **Dedicated OS task**: Using dedicated tasks for Rust systems is similar to having a dedicated core but allows tighter integration and thus lower hardware requirements.
- **Deep integration**: Allow Rust code to run side by side with C++. This reduces overhead and allows for the most flexibility, but with potentially the most amount of interaction with the language barrier.

We came to the conclusion that deep integration makes the most sense in this environment. Based on the following reasoning:

1. We think Rust is going to have a bright future. It is ready for production, bringing increased safety and development comfort. The choice of integration should not limit the adoption of Rust to only some specific components.
2. When interacting with C++ components, there is always the need to jump over the language barrier. No matter which integration path is chosen, all exposed C++ components need to provide their functionality in Rust.
3. Creating artificial barriers will hinder adoption. Experience shows the need for flexibility in complicated projects.
4. OpenBSW uses cooperative multitasking per priority level. This ensures highest efficiency. Using a different approach would lead to inefficiencies.

#### Benefits of async Rust
As previously explained, we want to strive for deep integration. In OpenBSW there already is a concept of Async, which allows the cooperative scheduling of Runnables.
Direct interaction with sync Rust is cumbersome as persisting state would then require using static mutable memory.
Using async Rust makes this much more ergonomic by compiling down to a State Machine that can be placed into static memory. The runtime then ensures safe access.

The only difference between Async in OpenBSW and async Rust is that Runnables cannot return control during execution. Instead, Rust will return control to the executor at each await point.
This can however be easily modeled in OpenBSW Async by rescheduling the executor at each await point.

#### Current limitations
The quality and quantity of bindings to C++ need improvement. This will be a lot of work, but it can be done bit by bit.

Relying on an embassy runtime enables this proof of concept; however, as embassy is mostly designed to take full control of a system, the Runnable hosting the Embassy runtime would currently monopolize all the runtime under high load.
This violates the approach of cooperative multitasking. In the medium term, the Rust executor needs to be modified to return control back to the OpenBSW Async runtime more often to ensure fair scheduling across Rust and C++.

### Key Changes Explained

#### 1. New Crates

- **[`openbsw-runtime`](libs/bsw/async/rust)**: This crate provides an async executor based on Embassy's raw executor. This executor is designed to be run in an OpenBSW Async Runnable ([ExecutorRunnable](libs/bsw/async/rust/include/ExecutorRunnable.h)).

- **[`openbsw-timer`](libs/bsw/timer/rust)**: Implements the Embassy time driver on top of OpenBSW Timers.

- **[`openbsw-can`](libs/bsw/cpp2can/rust)**: Handles CAN frame conversion between Rust and C++ types using the `embedded_can` crate.

- **[`openbsw-logger`](libs/bsw/logger/rust)**: Implements logging by sending log messages to the C++ Logger or printing them to stdout as a fallback.

- **[`openbsw-console-out`](libs/rust/console_out)**: Provides an interface for printing to the console, used by the logger.

- **[`openbsw-panic-handler`](libs/rust/panic_handler)**: Defines custom panic handling behavior for Rust code.

#### 2. Integration into `rustHelloWorld`

- **Initialization and Polling**:
  - `init_demo_runtime`: Initializes the async runtime with tasks.
  - `runtime_poll`: Drives the async runtime.

- **CAN Frame Handling**:
  - `receive_new_can_frame`: Receives a CAN frame from C++ using Embassy's watch channel.

- **Async Tasks**:
  - `loopy`, `loopy2`, and `can_task` are defined as async tasks. They perform periodic work, different types of logging, and handle incoming CAN frames respectively.

#### 3. Bindings

- **`bindgen`**: Used to generate Rust bindings for C++ header files (`CANFrame.h`). This allows the Rust code to directly interact with C++ data structures.
- **`cbindgen`**: Used to generate C bindings from Rust crates.
