use std::fmt;
use thiserror::Error;
use alloy_primitives::{hex, Address};
use alloy_primitives::hex::FromHex;
use miden_bridge::notes::bridge::croschain;
use miden_objects::{Felt, FieldElement, NoteError, StarkField, Word};
use miden_objects::note::{NoteInputs, NoteRecipient};
use miden_objects::utils::DeserializationError;

#[derive(Error, Debug)]
pub enum AddressFormatError {
    #[error(transparent)]
    MalformedEvmAddress(#[from] hex::FromHexError),
    #[error(transparent)]
    FeltDeserializationError(#[from] DeserializationError),
    #[error(transparent)]
    FmtError(#[from] fmt::Error),
}
pub fn evm_address_to_felts(address: String) -> Result<[Felt; 3], AddressFormatError> {

    let evm_dest_address = Address::from_hex(address.as_str())
        .map_err(AddressFormatError::MalformedEvmAddress)?;

    let address_felts = [
        Felt::try_from(
            &evm_dest_address.0[..8]
        ).map_err(AddressFormatError::FeltDeserializationError)?,
        Felt::try_from(
            &evm_dest_address.0[8..16]
        ).map_err(AddressFormatError::FeltDeserializationError)?,
        Felt::from_bytes_with_padding(
            &evm_dest_address.0[16..20]
        )
    ];

    Ok(address_felts)
}

pub fn build_crosschain_recipient(
    serial_number: Word,
    bridge_note_serial_number: Word,
    dest_chain: u32,
    dest_addr: [Felt; 3],
) -> Result<NoteRecipient, NoteError> {
    Ok(NoteRecipient::new(
        serial_number,
        croschain(),
        NoteInputs::new(vec![
            bridge_note_serial_number[3],
            bridge_note_serial_number[2],
            bridge_note_serial_number[1],
            bridge_note_serial_number[0],
            Felt::from(dest_chain.clone()),
            dest_addr[2],
            dest_addr[1],
            dest_addr[0],
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
        ])?
    ))
}