use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("../wasmvm-sys/bindings.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("init_cache")
        .allowlist_function("release_cache")
        .allowlist_function("store_code")
        .allowlist_function("remove_wasm")
        .allowlist_function("load_wasm")
        .allowlist_function("pin")
        .allowlist_function("unpin")
        .allowlist_function("instantiate")
        .allowlist_function("execute")
        .allowlist_function("migrate")
        .allowlist_function("migrate_with_info")
        .allowlist_function("sudo")
        .allowlist_function("reply")
        .allowlist_function("query")
        .allowlist_function("ibc_channel_open")
        .allowlist_function("ibc_channel_connect")
        .allowlist_function("ibc_channel_close")
        .allowlist_function("ibc_packet_receive")
        .allowlist_function("ibc_packet_ack")
        .allowlist_function("ibc_packet_timeout")
        .allowlist_function("analyze_code")
        .allowlist_function("get_pinned_metrics")
        .allowlist_function("get_metrics")
        .allowlist_function("version_str")
        .allowlist_function("destroy_unmanaged_vector")
        .allowlist_type("UnmanagedVector")
        .allowlist_type("ByteSliceView")
        .allowlist_type("Metrics")
        .allowlist_type("AnalysisReport")
        .allowlist_type("OptionalU64")
        .allowlist_type("cache_t")
        .allowlist_type("GasReport")
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("could not write bindings");
}
