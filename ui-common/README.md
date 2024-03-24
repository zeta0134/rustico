# Rustico - UI (common)

Platform independent interface components for running the Rustico emulator. This implements a rather basic event pump, and is designed to help abstract away the differences between the shell programs for tasks that are very commonly required, like loading a cartridge, saving SRAM, or stepping through the simulation by cycle.

This is rather WIP and changing rapidly as the shell programs are developed. Right this moment, the `/wasm` build has probably the most complete reference implementation of a shell consuming its featureset. Pin versions and tread carefully if using this crate as a dependency.
