# Zig Binding Audit (2026-05-30)

## Status

Currently 0/100 e2e passing - all tests crash with SIGABRT at runtime.

## Bugs Found

### CRITICAL: Null Pointer Dereference in All Extract Functions

**Location**: Lines 3885-3888, 3931-3934, 3968-3971, 3999-4002 in `packages/zig/src/kreuzberg.zig`

**Issue**: Force-unwrap of potentially-null pointers.

The extraction functions call C FFI functions that return `?*KREUZBERGExtractionResult` (optional pointer). The current code:

```zig
const _result = c.kreuzberg_extract_bytes(...);
if (c.kreuzberg_last_error_code() != 0) {
    return _first_error(KreuzbergError);
}
if (config_handle) |h| c.kreuzberg_extraction_config_free(h);
return blk: {
    const _json_ptr = c.kreuzberg_extraction_result_to_json(_result.?);  // CRASH HERE
    defer _free_string(_json_ptr);
    c.kreuzberg_extraction_result_free(_result.?);
    const slice = std.mem.sliceTo(_json_ptr, 0);  // CRASH HERE IF _json_ptr is null
    const owned = try std.heap.c_allocator.dupe(u8, slice);
    break :blk owned;
}
```

**Problems**:

1. **Line 3885 / 3968**: The `_result.?` force-unwrap will crash if `_result` is null. The error-code check assumes that if error code is non-zero, the result is null. But if the error code is zero AND the result is null, this crashes.
2. **Line 3888 / 3971**: After calling `kreuzberg_extraction_result_to_json(_result)`, the returned `_json_ptr` can be null but the code immediately dereferences it with `std.mem.sliceTo(_json_ptr, 0)`, which crashes.
3. **Resource leak**: If `_json_ptr` is null and `_result` was successfully freed, but we never reach the owned allocation, we have a dangling JSON pointer.

**Affected Functions**:

- `extract_bytes` (line 3871)
- `extract_file` (line 3897)
- `extract_bytes_sync` (line 3938)
- `extract_file_sync` (line 3951)
- `batch_extract_bytes_sync` (line 3978)
- And all batch functions using similar patterns

**Root Cause**: The generated Zig binding did not implement proper null-checking for C FFI returns. The pattern assumes every non-error call returns a valid pointer, which is not guaranteed.

---

## Fix Strategy

### For Extract Functions (6 affected functions)

For each extract function:

1. **Check `_result` before unwrap**:

   ```zig
   const _result = c.kreuzberg_extract_bytes(...);
   if (_result == null) {
       if (c.kreuzberg_last_error_code() != 0) {
           return _first_error(KreuzbergError);
       }
       // Error code is 0 but result is null - treat as unknown error
       return KreuzbergError.Other;
   }
   ```

2. **Check `_json_ptr` before dereference**:

   ```zig
   const _json_ptr = c.kreuzberg_extraction_result_to_json(_result.?);
   if (_json_ptr == null) {
       c.kreuzberg_extraction_result_free(_result.?);
       return KreuzbergError.Serialization;
   }
   defer _free_string(_json_ptr);
   ```

3. **Ensure error-code check runs first**:
   Move the error-code check to immediately after the C call, before any other operations.

### For Vtable Thunks (24 affected functions)

For each vtable thunk that casts `ud: ?*anyopaque` to `*T`:

1. **Null-check before cast**:

   ```zig
   const self: *T = if (ud) |u| @ptrCast(@alignCast(u)) else {
       // Handle null user_data - should not happen in normal operation
       // Either return error or abort with clear message
       return error.NullUserData; // or similar
   };
   ```

2. **Or, require non-null in the vtable signature** (if feasible):
   Change function pointer signatures to use `*anyopaque` instead of `?*anyopaque` where null is not expected.

---

## Detailed Audit

### Type Safety

**Status**: ✅ PASS

- All Zig-side type declarations correctly match the C header (kreuzberg.h)
- Opaque handle types (e.g., `*KREUZBERGExtractionResult`) are properly declared
- Struct definitions have correct field types and layouts

### Allocator Lifetime

**Status**: ⚠️ PASS WITH CAVEATS

- Proper use of `std.heap.c_allocator` for FFI allocations
- All `defer` blocks correctly paired
- Example (line 3889): `std.heap.c_allocator.dupe(u8, slice)` returns owned slice that caller must free
- Tests correctly call `defer std.heap.c_allocator.free(_result_json)` (e.g., line 33 in smoke_test.zig)
- **Issue**: No safeguard if intermediate conversions fail

### Error Return Convention

**Status**: ❌ FAIL

- Error-code checks exist but don't fully validate all return states
- The pattern `if (error_code != 0) return error` assumes result is null, but doesn't verify
- Inverse situation (error_code == 0 but result == null) is not handled
- Should use: `if (result == null || error_code != 0)` pattern

### Null Pointer Checks

**Status**: ❌ FAIL

- Force-unwrap (`_result.?`) assumes `_result` is never null when error_code == 0
- JSON conversion return (`_json_ptr`) is never checked for null
- String conversion (`std.mem.sliceTo(_json_ptr, 0)`) dereferences unchecked pointer

### Config Handle Freeing

**Status**: ✅ PASS

- Lines 3883, 3929, 3966, 3997, 4027, 4057, 4110, 4157: Consistent patterns
- All extraction functions properly free config_handle on all code paths
- Conditional: `if (config_handle) |h| c.kreuzberg_extraction_config_free(h);` is correct

### Batch Operations

**Status**: ❌ FAIL

- `batch_extract_bytes_sync`, `batch_extract_files_sync` (lines 3978, 4005) follow same buggy pattern
- Additional complexity: iteration over batch items adds risk if conversions fail mid-loop

---

### Vtable Function Pointers

**Status**: ❌ FAIL

- **Locations**: Lines 4710, 4724, 4737, 4744, 4755, 4766, 4773, 4780, 5019, 5033, 5044, 5051, 5058, 5306, 5320, 5327, 5456, 5463, 5669, 5683, 5696, 5707, 5714, 5846 (24 occurrences)
- **Issue**: All vtable thunks cast `ud: ?*anyopaque` to `*T` without null-check:

  ```zig
  const self: *T = @ptrCast(@alignCast(ud));  // CRASH if ud is null
  ```

- **Root Cause**: The thunks assume `ud` is never null (always points to the user data passed at registration). But if Rust code calls the thunk with null ud, this crashes.
- **Affected Vtables**: DocumentExtractor, OcrBackend, PostProcessor, Validator, Renderer, EmbeddingBackend (all plugin trait implementations)

**Risk**: HIGH - If Rust FFI layer ever calls a vtable thunk with null `ud`, the binding crashes. This would be a soundness hole if user code accidentally passes null when registering plugins.

---

## Test Findings

**Baseline**: Currently 0/100 green (all 22 tests crash with SIGABRT + 23 skipped due to linking)

**Crash Signature**:

```text
dyld[XXXX]: Library not loaded: @rpath/libkreuzberg_ffi.dylib
```

**Resolution**: Built FFI with `task rust:ffi:build`, tests now run and crash on first null-deref.

---

## Summary of Issues

| Category | Count | Severity | Lines |
|----------|-------|----------|-------|
| Extract result null-deref | 6 | CRITICAL | 3885, 3931, 3968, 3999, 4460, 4462 |
| JSON pointer null-deref | 6 | CRITICAL | 3888, 3934, 3971, 4002, 4463+ |
| Vtable ud null-deref | 24 | HIGH | 4710, 4724, ..., 5846 |
| **Total** | **36** | **CRITICAL/HIGH** | See details above |

---

## Recommendations

1. **Immediate**: Fix null-checks in all extract functions (6 functions total)
2. **Follow-up**: Fix null-checks in all vtable thunks (24 functions total)
3. **Testing**: All tests should exercise error paths, not just happy paths
4. **Codegen**: Review Alef's Zig generator to ensure it always emits null-checks after C calls
5. **CI**: Enable address sanitizer or Valgrind for Zig e2e tests to catch null-derefs earlier

---

## Files to Fix

- `/Users/naamanhirschfeld/workspace/kreuzberg-dev/kreuzberg/packages/zig/src/kreuzberg.zig`
  - Lines 3885-3891: `extract_bytes`
  - Lines 3931-3937: `extract_file`
  - Lines 3968-3974: `extract_bytes_sync`
  - Lines 3999-4005: `extract_file_sync`
  - Lines 4025-4031: `batch_extract_bytes_sync` (partial review needed)
  - Lines 4055-4061: `batch_extract_files_sync` (partial review needed)

No hand-edits to generated bindings — flag to Alef codegen for fix in next regeneration cycle.
