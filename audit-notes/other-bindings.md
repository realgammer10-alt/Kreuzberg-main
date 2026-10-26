# Hand-Edit Audit: 11 Green Bindings

Cycle baseline: `bd1bef129d` ("fix(wasm): exclude tree-sitter-wasm to avoid WASI linkage issues", 2026-05-30 10:12 +0200).
Tip at audit time: `e0cad0e6c5`.

The audit covers all paths owned by each of the 11 currently-green bindings (Python, Node, PHP, Java, Ruby, Elixir, R, Zig, Go, C#, Rust). Scope was inspected with:

```text
git log --oneline bd1bef129d..HEAD -- <paths>
git diff       bd1bef129d..HEAD -- <paths>
```

Working-tree dirt was also checked (`git status --short`); the only unstaged work in the repo touches kotlin-android (out of scope for this audit).

The pre-cycle root-cause rename `5393349c7a` ("fix(rust)!: rename Uri to ExtractedUri to avoid dart:core collision") landed at 08:01 — *before* the baseline — so its per-language ripples are already absorbed by every binding listed here. No follow-up hand-edits to any of the 11 bindings exist in this cycle as a result.

---

## Python — `packages/python/`, `crates/kreuzberg-py/`, `e2e/python/`

No hand-edits in scope. `git diff bd1bef129d..HEAD -- packages/python/ crates/kreuzberg-py/ e2e/python/` is empty.

## Node — `packages/typescript/`, `crates/kreuzberg-node/`, `e2e/node/`

No hand-edits in scope. `git diff bd1bef129d..HEAD -- packages/typescript/ crates/kreuzberg-node/ e2e/node/` is empty.

## PHP — `packages/php/`, `crates/kreuzberg-php/`, `e2e/php/`

No hand-edits in scope. `git diff bd1bef129d..HEAD -- packages/php/ crates/kreuzberg-php/ e2e/php/` is empty.

## Java — `packages/java/`, `e2e/java/`

No hand-edits in scope. `git diff bd1bef129d..HEAD -- packages/java/ e2e/java/` is empty.

## Ruby — `packages/ruby/`, `e2e/ruby/`

No hand-edits in scope. `git diff bd1bef129d..HEAD -- packages/ruby/ e2e/ruby/` is empty.

## Elixir — `packages/elixir/`, `e2e/elixir/`

No hand-edits in scope. `git diff bd1bef129d..HEAD -- packages/elixir/ e2e/elixir/` is empty.

## R — `packages/r/`, `e2e/r/`

No hand-edits in scope. `git diff bd1bef129d..HEAD -- packages/r/ e2e/r/` is empty.

## Zig — `packages/zig/`, `e2e/zig/`

No hand-edits in scope. `git diff bd1bef129d..HEAD -- packages/zig/ e2e/zig/` is empty. (`packages/zig/` does exist.)

## Go — `packages/go/`, `e2e/go/`

No hand-edits in scope. `git diff bd1bef129d..HEAD -- packages/go/ e2e/go/` is empty.

## C# — `packages/csharp/`, `e2e/csharp/`

No hand-edits in scope. `git diff bd1bef129d..HEAD -- packages/csharp/ e2e/csharp/` is empty.

## Rust core — `crates/kreuzberg/`, `crates/kreuzberg-ffi/`, `crates/kreuzberg-cli/`, `e2e/rust/`

Two in-scope changes, both motivated by non-rust binding consumers (not hand-edits to alef-generated output):

### 1. `crates/kreuzberg-ffi/Cargo.toml` — add `rlib` to `crate-type`

- Commit: `66ca4f40eb` ("fix(kotlin-android): force-link kreuzberg-ffi symbols into JNI cdylib").
- Diff: single line, `crate-type = ["cdylib", "staticlib"]` → `crate-type = ["cdylib", "staticlib", "rlib"]`.
- Reason: lets `kreuzberg-jni` depend on `kreuzberg-ffi` as a Rust crate so its `#[used]` symbol-pinning trick can resolve every `kreuzberg_ffi_*` export at link time. Without `rlib`, only the C-ABI surface is exported and the JNI shim's `extern "C"` forwards resolve to null at runtime.
- **Category**: ROOT_CAUSE (FFI crate manifest change in the shared FFI surface, applied for a single downstream consumer but harmless / additive for everyone).
- **Suggested upstream fix**: none required against alef templates — `crates/kreuzberg-ffi/Cargo.toml` is hand-written, not alef-generated. The change is already in the right place. Worth noting that other static-link consumers (Swift, Zig, C#, Go, R) all use the `staticlib`/`cdylib` artifacts as before and don't regress.

### 2. `crates/kreuzberg/src/extraction/pst.rs` — wasm32 gate + fallback

- Commit: `86f4510cfd` ("fix(wasm): gate tempfile usage in PST extraction for wasm32 target").
- Diff: 11 lines. Existing `extract_pst_messages` gains `#[cfg(all(feature = "email", not(target_arch = "wasm32")))]`; a sibling `#[cfg(all(feature = "email", target_arch = "wasm32"))]` returns `KreuzbergError::Validation` with the message "PST extraction is not supported on WebAssembly targets".
- Reason: `outlook_pst::open_store()` needs a file path, which requires `tempfile`, which needs WASI mkstemp — unavailable on `wasm32-unknown-unknown`.
- **Category**: ROOT_CAUSE (Rust core).
- **Suggested upstream fix**: none. The gate lives in the core because the constraint is a property of the WASM target, not of any binding template. Already documented in `audit-notes/wasm.md` (item 9).

No other in-scope hand-edits exist for the rust core or `e2e/rust/`.

---

## Summary

| Language | Hand-edit count | Categories present |
|----------|-----------------|--------------------|
| Python   | 0 | — |
| Node     | 0 | — |
| PHP      | 0 | — |
| Java     | 0 | — |
| Ruby     | 0 | — |
| Elixir   | 0 | — |
| R        | 0 | — |
| Zig      | 0 | — |
| Go       | 0 | — |
| C#       | 0 | — |
| Rust     | 2 | ROOT_CAUSE x2 (`kreuzberg-ffi` crate-type, `pst.rs` wasm gate) |

All 11 bindings are green at the cycle tip without any hand-edits to alef-generated output. The two rust-core touches in scope (`crates/kreuzberg-ffi/Cargo.toml`, `crates/kreuzberg/src/extraction/pst.rs`) are root-cause fixes in hand-written Rust source that benefit downstream bindings (kotlin-android JNI, WASM respectively) and require no alef template work.

The Uri → ExtractedUri rename landed pre-baseline in `5393349c7a` and was inherited cleanly by every binding in this list; no per-language follow-up hand-edits were needed in any of them.

---

## Cross-cutting observations

1. **The kotlin-jni `rlib` treatment is JNI-specific.** Adding `rlib` to `crates/kreuzberg-ffi/Cargo.toml` was needed because kotlin-jni is the *only* downstream consumer that links the FFI crate as a Rust dependency (so the symbol-pinning `#[used]` array compiles). Every other binding in this audit consumes `kreuzberg-ffi` via its C ABI (`cdylib`/`staticlib`) or via its own Rust binding crate (`kreuzberg-py`, `kreuzberg-node`, `kreuzberg-php`, `kreuzberg-wasm`) and is unaffected. No other binding should adopt the JNI pattern — the additional `rlib` artifact is the entire fix.
2. **All 11 green bindings are pure regenerations on top of alef + rust-core changes.** Across the cycle baseline → tip, no alef-headered file under any of `packages/{python,typescript,php,java,ruby,elixir,r,zig,go,csharp}/`, `crates/kreuzberg-{py,node,php}/`, or `e2e/{python,node,php,java,ruby,elixir,r,zig,go,csharp,rust}/` was hand-touched. The active hand-edit pressure in this cycle has been concentrated entirely on the four trailing bindings (dart, swift, wasm, kotlin-android), which are covered by their dedicated audit notes (`audit-notes/dart.md`, `audit-notes/swift.md`, `audit-notes/wasm.md`, and the kotlin-android working tree).
3. **`crates/kreuzberg/src/extraction/pst.rs` wasm gate is the only rust-core change of substance.** It's binding-specific knowledge intentionally encoded in the core (single error site, clear error message for WASM consumers) and matches the policy already documented for similar gates (paddle-ocr, layout-detection, embeddings, auto-rotate). No alef template change required.
4. **No `Uri → ExtractedUri` aftershocks.** Because `5393349c7a` landed before the baseline and pre-regenerated every binding, the green eleven inherited the rename without per-language fallout. The only place where fallout still showed up post-baseline is the Swift `RustBridge` module (`e222de3a59`, `9c74d4ef08`, `796b57e6ac`), which is already tracked in `audit-notes/swift.md`.
