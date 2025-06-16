use crate::consts::keeper::{HISTORICAL_INFO_KEY, VALIDATOR_QUEUE_KEY};
use gears::{
    tendermint::types::time::timestamp::Timestamp,
    types::address::{AccAddress, ValAddress},
};

/// Returns a key prefix for indexing a redelegation
/// from an address to a destination validator.
pub fn redelegations_by_delegator_to_validator_destination_index_key(
    dst_val_addr: &ValAddress,
    delegator_addr: &AccAddress,
) -> Vec<u8> {
    [
        dst_val_addr.prefix_len_bytes(),
        delegator_addr.prefix_len_bytes(),
    ]
    .concat()
}

pub fn historical_info_key(height: u32) -> Vec<u8> {
    [&HISTORICAL_INFO_KEY, height.to_string().as_bytes()].concat()
}

pub(super) fn validator_queue_key(end_time: &Timestamp, end_height: u32) -> Vec<u8> {
    let height_bz = (end_height as u64).to_be_bytes();
    let time_bz = end_time.format_bytes_rounded();

    let mut bz = VALIDATOR_QUEUE_KEY.to_vec();
    bz.extend_from_slice(&(time_bz.len() as u64).to_be_bytes());
    bz.extend_from_slice(&time_bz);
    bz.extend_from_slice(&height_bz);
    bz
}

pub(super) fn parse_validator_queue_key(key: &[u8]) -> anyhow::Result<(Timestamp, u32)> {
    // TODO: there are no checks on index out of bounds
    let prefix_len = VALIDATOR_QUEUE_KEY.len();
    if key[..prefix_len] != VALIDATOR_QUEUE_KEY {
        return Err(anyhow::anyhow!(
            "Invalid validators queue key. Invalid prefix."
        ));
    }
    let time_len = u64::from_be_bytes(key[prefix_len..prefix_len + 8].try_into()?);
    let time_bytes = key[prefix_len + 8..prefix_len + 8 + time_len as usize].to_vec();
    let time = Timestamp::try_from_formatted_bytes(&time_bytes)?;
    let height =
        u64::from_be_bytes(key[prefix_len + 8 + time_len as usize..].try_into()?).try_into()?;
    Ok((time, height))
}

// This is the key for use in the unbonding queue sub store (UNBONDING_QUEUE_KEY prefix)
pub(super) fn unbonding_delegation_time_key(time: &Timestamp) -> Vec<u8> {
    time.format_bytes_rounded()
}

pub(super) fn redelegation_time_key(time: &Timestamp) -> Vec<u8> {
    time.format_bytes_rounded()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gears::tendermint::types::time::timestamp::Timestamp;

    #[test]
    fn test_validator_queue_key() {
        let time = Timestamp::try_new(100, 100).unwrap();
        let height = 100;
        let key = validator_queue_key(&time, height);
        assert_eq!(
            key,
            vec![
                67, 0, 0, 0, 0, 0, 0, 0, 29, 49, 57, 55, 48, 45, 48, 49, 45, 48, 49, 84, 48, 48,
                58, 48, 49, 58, 52, 48, 46, 48, 48, 48, 48, 48, 48, 48, 48, 48, 0, 0, 0, 0, 0, 0,
                0, 100
            ]
        );
    }

    #[test]
    fn test_validator_queue_key_2() {
        let time = Timestamp::try_new(1814400, 0).unwrap();
        let height = 1;
        let key = validator_queue_key(&time, height);
        assert_eq!(
            key,
            vec![
                67, 0, 0, 0, 0, 0, 0, 0, 29, 49, 57, 55, 48, 45, 48, 49, 45, 50, 50, 84, 48, 48,
                58, 48, 48, 58, 48, 48, 46, 48, 48, 48, 48, 48, 48, 48, 48, 48, 0, 0, 0, 0, 0, 0,
                0, 1
            ]
        );

        let (parsed_time, parsed_height) = parse_validator_queue_key(&key).unwrap();

        assert_eq!(time, parsed_time);
        assert_eq!(height, parsed_height);
    }

    #[test]
    fn test_parse_validator_queue_key() {
        let time = Timestamp::try_new(100, 0).unwrap();
        let height = 100;
        let key = validator_queue_key(&time, height);
        let (parsed_time, parsed_height) = parse_validator_queue_key(&key).unwrap();
        assert_eq!(time, parsed_time);
        assert_eq!(height, parsed_height);
    }
}
