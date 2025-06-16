use tendermint_informal::abci::Code;
use wasm::error::WasmError;

#[test]
fn abci_code_mapping() {
    assert_eq!(
        WasmError::NotFound { kind: "contract" }.abci_code(),
        Code::from(5u32)
    );
    assert_eq!(
        WasmError::Unauthorized { action: "execute" }.abci_code(),
        Code::from(4u32)
    );
    assert_eq!(
        WasmError::InvalidRequest {
            reason: "bad".into()
        }
        .abci_code(),
        Code::from(3u32)
    );
    let internal = WasmError::Internal {
        reason: "oops".into(),
    };
    assert_eq!(internal.abci_code(), Code::from(1u32));
}

#[test]
fn display_messages() {
    let e = WasmError::Unauthorized { action: "execute" };
    assert!(format!("{e}").contains("unauthorized"));
}
