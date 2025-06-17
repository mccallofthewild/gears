#![allow(unsafe_code)]
//! Rust implementation mirroring the CosmWasm `wasmvm` Go library.
//!
//! This crate provides a thin wrapper around `cosmwasm_vm` with an API
//! closely resembling the Go bindings found in
//! [`CosmWasm/wasmvm`](https://github.com/CosmWasm/wasmvm). The goal is to
//! offer a drop–in replacement for the Go library when writing Rust
//! applications. At present only a subset of the functionality is
//! implemented. All other entry points are marked with `todo!()` and should
//! be completed to match the behaviour of the upstream implementation.

use sha2::{Digest, Sha256};
use thiserror::Error;

// Import low-level FFI bindings generated from `wasmvm-sys` headers. These
// functions mirror the C interface used by the Go bindings and are provided by
// the `wasmvm-sys` crate compiled as a cdylib.
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

/// SHA‑256 checksum identifying a stored contract.
///
/// This mirrors the `Checksum` type in the Go library which wraps a fixed
/// length array of 32 bytes. Checksums are created via [`create_checksum`].
pub type Checksum = [u8; 32];

/// Raw WASM bytecode blob.
///
/// In the Go implementation this is an alias for `[]byte` and is used when
/// uploading or retrieving contract code.
pub type WasmCode = Vec<u8>;

/// Cost per byte charged for compiling Wasm code. This mirrors the constant in
/// the Go implementation and is expressed in CosmWasm gas units.
const COST_PER_BYTE: u64 = 3 * 140_000;

fn compile_cost(len: usize) -> u64 {
    COST_PER_BYTE * len as u64
}

/// Result of [`VM::analyze_code`]. Mirrors the `AnalysisReport` type returned by
/// the Go bindings but converts UTF‑8 vectors into native Rust strings for ease
/// of use.
#[derive(Debug)]
pub struct Analysis {
    /// Whether the contract exports all required IBC entry points.
    pub has_ibc_entry_points: bool,
    /// List of all exported entry points such as `instantiate` or `execute`.
    pub entrypoints: Vec<String>,
    /// Capabilities required by the contract like `staking` or `iterator`.
    pub required_capabilities: Vec<String>,
    /// Optional migrate version of the contract.
    pub contract_migrate_version: Option<u64>,
}

/// Create a [`ffi::ByteSliceView`] from a byte slice for passing to FFI calls.
fn view_bytes(data: &[u8]) -> ffi::ByteSliceView {
    ffi::ByteSliceView {
        is_nil: false,
        ptr: if data.is_empty() {
            core::ptr::null()
        } else {
            data.as_ptr()
        },
        len: data.len(),
    }
}

/// Convert an [`ffi::UnmanagedVector`] into an owned [`Vec<u8>`] and destroy the
/// original to avoid memory leaks.
unsafe fn consume_vector(vec: ffi::UnmanagedVector) -> Vec<u8> {
    if vec.is_none {
        Vec::new()
    } else {
        let out = core::slice::from_raw_parts(vec.ptr, vec.len).to_vec();
        ffi::destroy_unmanaged_vector(vec);
        out
    }
}

/// Convert an unmanaged vector that contains UTF-8 data into a [`String`].
unsafe fn consume_string(vec: ffi::UnmanagedVector) -> anyhow::Result<String> {
    let bytes = consume_vector(vec);
    Ok(String::from_utf8(bytes)?)
}

/// Errors returned when creating checksums or invoking VM operations.
#[derive(Debug, Error)]
pub enum WasmvmError {
    /// The provided wasm byte slice was empty.
    #[error("wasm bytes nil or empty")]
    EmptyWasm,
    /// The provided wasm blob was smaller than the 4 byte magic header.
    #[error("wasm bytes shorter than 4 bytes")]
    TooShort,
    /// The blob does not start with the expected `\0asm` magic number.
    #[error("wasm bytes do not start with Wasm magic number")]
    MissingMagic,
    /// Placeholder for any other failure.
    #[error("unimplemented")]
    Other,
}

/// Computes the CosmWasm checksum for a given wasm byte array.
///
/// The function performs the same validation as `CreateChecksum` in the Go
/// bindings. It ensures the input is not empty, is at least four bytes long and
/// begins with the wasm magic number. The SHA‑256 hash of the bytes is returned
/// on success.
pub fn create_checksum(wasm: &[u8]) -> Result<Checksum, WasmvmError> {
    if wasm.is_empty() {
        return Err(WasmvmError::EmptyWasm);
    }
    if wasm.len() < 4 {
        return Err(WasmvmError::TooShort);
    }
    // Magic number for WebAssembly modules is "\0asm"
    if wasm[..4] != [0x00, 0x61, 0x73, 0x6d] {
        return Err(WasmvmError::MissingMagic);
    }
    let hash = Sha256::digest(wasm);
    Ok(hash.into())
}

/// Returns the version string of the linked `libwasmvm`.
pub fn version() -> &'static str {
    unsafe {
        let ptr = ffi::version_str();
        // SAFETY: `version_str` guarantees a valid, NUL terminated C string with static lifetime.
        std::ffi::CStr::from_ptr(ptr)
            .to_str()
            .expect("invalid utf-8")
    }
}

/// Main entry point replicating the `VM` struct from the Go library.
///
/// Internally this will wrap a `cosmwasm_vm::cache::Cache` and expose high level
/// methods such as `store_code`, `instantiate` and `execute`. Only the structure
/// and method stubs are provided for now; the detailed behaviour should follow
/// the reference implementation closely.
#[derive(Debug)]
pub struct VM {
    cache: *mut ffi::cache_t,
    print_debug: bool,
}

impl VM {
    /// Creates a new virtual machine instance.
    ///
    /// Parameters mirror `NewVM` in the Go bindings. At a minimum a base
    /// directory for caching compiled contracts must be supplied. Additional
    /// options controlling memory limits and capabilities will be added later.
    pub fn new(
        data_dir: &str,
        supported_capabilities: &[&str],
        memory_limit: u32,
        print_debug: bool,
        cache_size: u32,
    ) -> anyhow::Result<Self> {
        use cosmwasm_vm::{CacheOptions, Config, Size};
        use std::collections::HashSet;

        let caps: HashSet<String> = supported_capabilities
            .iter()
            .map(|s| s.to_string())
            .collect();
        let options = CacheOptions::new(
            data_dir,
            caps,
            Size::mebi(cache_size as usize),
            Size::mebi(memory_limit as usize),
        );
        let config = Config::new(options);
        let config_bytes = serde_json::to_vec(&config)?;

        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let view = view_bytes(&config_bytes);
        let cache = unsafe { ffi::init_cache(view, &mut err) };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }

        Ok(Self { cache, print_debug })
    }

    /// Releases all resources associated with this VM instance.
    ///
    /// Corresponds to the `Cleanup` method in the Go version.
    pub fn cleanup(&mut self) {
        unsafe {
            ffi::release_cache(self.cache);
        }
    }

    /// Stores the provided wasm code and returns its checksum.
    pub fn store_code(
        &mut self,
        code: WasmCode,
        gas_limit: u64,
    ) -> anyhow::Result<(Checksum, u64)> {
        let cost = compile_cost(code.len());
        if gas_limit < cost {
            anyhow::bail!("out of gas");
        }
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let view = view_bytes(&code);
        let checksum_vec = unsafe { ffi::store_code(self.cache, view, true, true, &mut err) };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        let data = unsafe { consume_vector(checksum_vec) };
        let checksum: Checksum = data
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid checksum length"))?;
        Ok((checksum, cost))
    }

    /// Retrieves raw Wasm bytes for a previously stored module.
    pub fn load_wasm(&mut self, checksum: &Checksum) -> anyhow::Result<Vec<u8>> {
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let wasm = unsafe { ffi::load_wasm(self.cache, view_bytes(checksum), &mut err) };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok(unsafe { consume_vector(wasm) })
    }

    /// Removes all artifacts associated with the given module from the cache.
    pub fn remove_wasm(&mut self, checksum: &Checksum) -> anyhow::Result<()> {
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        unsafe { ffi::remove_wasm(self.cache, view_bytes(checksum), &mut err) };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok(())
    }

    /// Pins the specified module into the in‑memory cache.
    pub fn pin(&mut self, checksum: &Checksum) -> anyhow::Result<()> {
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        unsafe { ffi::pin(self.cache, view_bytes(checksum), &mut err) };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok(())
    }

    /// Unpins a previously pinned module allowing it to be evicted.
    pub fn unpin(&mut self, checksum: &Checksum) -> anyhow::Result<()> {
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        unsafe { ffi::unpin(self.cache, view_bytes(checksum), &mut err) };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok(())
    }

    /// Analyses a stored contract and returns metadata about its exports and required capabilities.
    pub fn analyze_code(&mut self, checksum: &Checksum) -> anyhow::Result<Analysis> {
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let report = unsafe { ffi::analyze_code(self.cache, view_bytes(checksum), &mut err) };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }

        // Convert comma separated lists into vectors of strings
        let entrypoints = unsafe { consume_string(report.entrypoints)? };
        let required_caps = unsafe { consume_string(report.required_capabilities)? };

        let entrypoints = if entrypoints.is_empty() {
            Vec::new()
        } else {
            entrypoints.split(',').map(|s| s.to_string()).collect()
        };
        let required_capabilities = if required_caps.is_empty() {
            Vec::new()
        } else {
            required_caps.split(',').map(|s| s.to_string()).collect()
        };

        let version = if report.contract_migrate_version.is_some {
            Some(report.contract_migrate_version.value)
        } else {
            None
        };

        Ok(Analysis {
            has_ibc_entry_points: report.has_ibc_entry_points,
            entrypoints,
            required_capabilities,
            contract_migrate_version: version,
        })
    }

    /// Returns per-module metrics about pinned artifacts as MsgPack encoded bytes.
    pub fn get_pinned_metrics(&self) -> anyhow::Result<Vec<u8>> {
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let data = unsafe { ffi::get_pinned_metrics(self.cache, &mut err) };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok(unsafe { consume_vector(data) })
    }

    /// Instantiates a contract previously stored via [`store_code`].
    pub fn instantiate(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        info: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        // Dummy implementations of the DB/API/Querier interfaces. These mirror
        // the zero value used in the Go bindings when no custom callbacks are
        // supplied by the caller.
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };

        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };

        let res = unsafe {
            ffi::instantiate(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(info),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };

        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }

        let data = unsafe { consume_vector(res) };
        Ok((data, gas))
    }

    /// Executes a contract function.
    pub fn execute(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        info: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };
        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let res = unsafe {
            ffi::execute(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(info),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        let data = unsafe { consume_vector(res) };
        Ok((data, gas))
    }

    /// Queries a contract for read‑only data.
    pub fn query(
        &self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };
        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let res = unsafe {
            ffi::query(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        let data = unsafe { consume_vector(res) };
        Ok((data, gas))
    }

    /// Migrates an existing contract to new code.
    pub fn migrate(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };

        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };

        let res = unsafe {
            ffi::migrate(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };

        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        let data = unsafe { consume_vector(res) };
        Ok((data, gas))
    }

    /// Migrates with explicit migrate info passed separately.
    pub fn migrate_with_info(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        migrate_info: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };

        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };

        let res = unsafe {
            ffi::migrate_with_info(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                view_bytes(migrate_info),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };

        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        let data = unsafe { consume_vector(res) };
        Ok((data, gas))
    }

    /// Calls a privileged sudo entry point.
    pub fn sudo(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };
        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let res = unsafe {
            ffi::sudo(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok((unsafe { consume_vector(res) }, gas))
    }

    /// Replies with the result of a submessage.
    pub fn reply(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };
        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let res = unsafe {
            ffi::reply(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok((unsafe { consume_vector(res) }, gas))
    }

    /// IBC channel open callback.
    pub fn ibc_channel_open(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };
        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let res = unsafe {
            ffi::ibc_channel_open(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok((unsafe { consume_vector(res) }, gas))
    }

    /// IBC packet receive callback.
    pub fn ibc_packet_receive(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };
        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let res = unsafe {
            ffi::ibc_packet_receive(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok((unsafe { consume_vector(res) }, gas))
    }

    /// IBC channel connect callback.
    pub fn ibc_channel_connect(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };
        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let res = unsafe {
            ffi::ibc_channel_connect(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok((unsafe { consume_vector(res) }, gas))
    }

    /// IBC channel close callback.
    pub fn ibc_channel_close(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };
        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let res = unsafe {
            ffi::ibc_channel_close(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok((unsafe { consume_vector(res) }, gas))
    }

    /// Acknowledgement for a previously sent IBC packet.
    pub fn ibc_packet_ack(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };
        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let res = unsafe {
            ffi::ibc_packet_ack(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok((unsafe { consume_vector(res) }, gas))
    }

    /// Packet timeout callback.
    pub fn ibc_packet_timeout(
        &mut self,
        checksum: &Checksum,
        env: &[u8],
        msg: &[u8],
        gas_limit: u64,
    ) -> anyhow::Result<(Vec<u8>, ffi::GasReport)> {
        let db = ffi::Db {
            gas_meter: core::ptr::null_mut(),
            state: core::ptr::null_mut(),
            vtable: ffi::DbVtable {
                read_db: None,
                write_db: None,
                remove_db: None,
                scan_db: None,
            },
        };
        let api = ffi::GoApi {
            state: core::ptr::null(),
            vtable: ffi::GoApiVtable {
                humanize_address: None,
                canonicalize_address: None,
                validate_address: None,
            },
        };
        let querier = ffi::GoQuerier {
            state: core::ptr::null(),
            vtable: ffi::QuerierVtable {
                query_external: None,
            },
        };
        let mut gas = ffi::GasReport {
            limit: 0,
            remaining: 0,
            used_externally: 0,
            used_internally: 0,
        };
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let res = unsafe {
            ffi::ibc_packet_timeout(
                self.cache,
                view_bytes(checksum),
                view_bytes(env),
                view_bytes(msg),
                db,
                api,
                querier,
                gas_limit,
                self.print_debug,
                &mut gas,
                &mut err,
            )
        };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok((unsafe { consume_vector(res) }, gas))
    }

    /// Returns metrics about the internal cache.
    pub fn get_metrics(&self) -> anyhow::Result<ffi::Metrics> {
        let mut err = ffi::UnmanagedVector {
            is_none: true,
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        let metrics = unsafe { ffi::get_metrics(self.cache, &mut err) };
        if !err.is_none {
            let msg = unsafe { String::from_utf8_lossy(&consume_vector(err)).into_owned() };
            anyhow::bail!(msg);
        }
        Ok(metrics)
    }
}
