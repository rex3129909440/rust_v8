# Microsoft Edge HTTPS evidence

Run `node tools/capture_edge_evidence.mjs` on Windows with Microsoft Edge
installed. The capture launches a clean, headless Edge profile, navigates to
`https://example.com/`, and writes deterministic TSV evidence for:

- Window own-property order and descriptors;
- interface prototype members and native function shapes;
- DedicatedWorkerGlobalScope own properties and prototype chain;
- targeted behavior for evidence-driven descriptor differences;
- browser version, counts, and SHA-256 digests.

The runtime does not load these files as API definitions. They are test
evidence used to review and update explicit Rust implementations.
