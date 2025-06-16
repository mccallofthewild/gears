use cosmwasm_std::Binary;
use gears::types::address::AccAddress;
use wasm::types::query::*;

fn sample_addr() -> AccAddress {
    AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux").unwrap()
}

#[test]
fn smart_contract_state_round_trip() {
    let msg = QuerySmartContractState {
        address: sample_addr(),
        query_data: Binary::from(b"{}".to_vec()),
    };
    let raw: proto::ProtoQuerySmartContractStateRequest = msg.clone().into();
    let back = QuerySmartContractState::try_from(raw).unwrap();
    assert_eq!(msg, back);

    let json = serde_json::to_string(&msg).unwrap();
    let de: QuerySmartContractState = serde_json::from_str(&json).unwrap();
    assert_eq!(de, msg);
}

#[test]
fn raw_contract_state_round_trip() {
    let msg = QueryRawContractState {
        address: sample_addr(),
        query_data: Binary::from(vec![0xAA, 0xBB]),
    };
    let raw: proto::ProtoQueryRawContractStateRequest = msg.clone().into();
    let back = QueryRawContractState::try_from(raw).unwrap();
    assert_eq!(msg, back);

    let json = serde_json::to_string(&msg).unwrap();
    let de: QueryRawContractState = serde_json::from_str(&json).unwrap();
    assert_eq!(de, msg);
}

#[test]
fn code_round_trip() {
    let msg = QueryCode { code_id: 42 };
    let raw: proto::ProtoQueryCodeRequest = msg.clone().into();
    let back = QueryCode::try_from(raw).unwrap();
    assert_eq!(msg, back);

    let json = serde_json::to_string(&msg).unwrap();
    let de: QueryCode = serde_json::from_str(&json).unwrap();
    assert_eq!(de, msg);
}

#[test]
fn contract_info_round_trip() {
    let msg = QueryContractInfo {
        address: sample_addr(),
    };
    let raw: proto::ProtoQueryContractInfoRequest = msg.clone().into();
    let back = QueryContractInfo::try_from(raw).unwrap();
    assert_eq!(msg, back);

    let json = serde_json::to_string(&msg).unwrap();
    let de: QueryContractInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(de, msg);
}

#[test]
fn contracts_by_code_round_trip() {
    let msg = QueryContractsByCode {
        code_id: 1,
        pagination: None,
    };
    let raw: proto::ProtoQueryContractsByCodeRequest = msg.clone().into();
    let back = QueryContractsByCode::try_from(raw).unwrap();
    assert_eq!(msg, back);

    let json = serde_json::to_string(&msg).unwrap();
    let de: QueryContractsByCode = serde_json::from_str(&json).unwrap();
    assert_eq!(de, msg);
}
