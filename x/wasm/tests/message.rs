use crate::message::*;
use cosmwasm_std::Binary;
use gears::types::base::coin::UnsignedCoin;
use gears::types::{address::AccAddress, base::coins::UnsignedCoins};
use std::str::FromStr;

fn sample_addr() -> AccAddress {
    AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux").unwrap()
}

#[test]
fn validate_store_code() {
    let msg = MsgStoreCode {
        sender: sample_addr(),
        wasm_byte_code: vec![0u8; 1],
        instantiate_permission: None,
    };
    assert!(msg.validate_basic().is_ok());

    let bad = MsgStoreCode {
        wasm_byte_code: vec![],
        ..msg
    };
    assert!(bad.validate_basic().is_err());
}

#[test]
fn validate_instantiate_contract() {
    let msg = MsgInstantiateContract {
        sender: sample_addr(),
        admin: None,
        code_id: 1,
        label: "contract".into(),
        msg: Binary::from(vec![1]),
        funds: UnsignedCoins::new(vec![UnsignedCoin::from_str("1uatom").unwrap()]).unwrap(),
    };
    assert!(msg.validate_basic().is_ok());
    let bad = MsgInstantiateContract { code_id: 0, ..msg };
    assert!(bad.validate_basic().is_err());
}

#[test]
fn validate_execute_contract() {
    let msg = MsgExecuteContract {
        sender: sample_addr(),
        contract: sample_addr(),
        msg: Binary::from(vec![1]),
        funds: UnsignedCoins::new(vec![UnsignedCoin::from_str("1uatom").unwrap()]).unwrap(),
    };
    assert!(msg.validate_basic().is_ok());
    let bad = MsgExecuteContract {
        msg: Binary::default(),
        ..msg
    };
    assert!(bad.validate_basic().is_err());
}

#[test]
fn validate_migrate_contract() {
    let msg = MsgMigrateContract {
        sender: sample_addr(),
        contract: sample_addr(),
        code_id: 2,
        msg: Binary::from(vec![1]),
    };
    assert!(msg.validate_basic().is_ok());
    let bad = MsgMigrateContract { code_id: 0, ..msg };
    assert!(bad.validate_basic().is_err());
}

#[test]
fn validate_update_admin() {
    let msg = MsgUpdateAdmin {
        sender: sample_addr(),
        new_admin: AccAddress::from_bech32("cosmos1z9det0w6aqr35d4pl3z0gp70wh6tt0q3p0m9q5")
            .unwrap(),
        contract: sample_addr(),
    };
    assert!(msg.validate_basic().is_ok());
    let bad = MsgUpdateAdmin {
        new_admin: msg.sender.clone(),
        ..msg
    };
    assert!(bad.validate_basic().is_err());
}

#[test]
fn validate_clear_admin() {
    let msg = MsgClearAdmin {
        sender: sample_addr(),
        contract: sample_addr(),
    };
    assert!(msg.validate_basic().is_ok());
}
