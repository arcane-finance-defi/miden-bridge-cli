use thiserror::Error;
use alloy_primitives::hex::FromHex;
use miden_bridge::accounts::token_wrapper::bridge_note_tag;
use miden_bridge::notes::bridge::croschain;
use miden_bridge::utils::{evm_address_to_felts, AddressFormatError};
use miden_objects::{AccountIdError, AssetError, Felt, FieldElement, NoteError, Word};
use miden_objects::account::AccountId;
use miden_objects::asset::FungibleAsset;
use miden_objects::note::{NoteAssets, NoteDetails, NoteFile, NoteId, NoteInputs, NoteRecipient, NoteTag};
use miden_objects::utils::{parse_hex_string_as_word};


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

#[derive(Debug, Error)]
pub enum CrosschainNoteReconstructionError {
    #[error("Unparsable hex word: {0}")]
    UnparsableHexError(String),
    #[error(transparent)]
    AddressFormatError(#[from] AddressFormatError),
    #[error(transparent)]
    AccountIdError(#[from] AccountIdError),
    #[error(transparent)]
    NoteError(#[from] NoteError),
    #[error(transparent)]
    AssetError(#[from] AssetError),
}

pub async fn reconstruct_crosschain_note(
    serial_number: &String,
    bridge_note_serial_number: &String,
    dest_chain: &u32,
    dest_address: &String,
    faucet_id: &String,
    asset_amount: &u64
) -> Result<(NoteFile, NoteId), CrosschainNoteReconstructionError> {
    let serial_number = parse_hex_string_as_word(serial_number)
        .map_err(|e| CrosschainNoteReconstructionError::UnparsableHexError(e.to_string()))?;

    let bridge_serial_number = parse_hex_string_as_word(bridge_note_serial_number)
        .map_err(|e| CrosschainNoteReconstructionError::UnparsableHexError(e.to_string()))?;

    let dest_addr = evm_address_to_felts(dest_address.to_string())?;

    let faucet_id = AccountId::from_hex(faucet_id)?;

    let recipient = build_crosschain_recipient(
        serial_number,
        bridge_serial_number,
        *dest_chain,
        dest_addr
    )?;

    let note_details = NoteDetails::new(
        NoteAssets::new(vec![
            FungibleAsset::new(
                faucet_id, *asset_amount
            )?.into()
        ])?,
        recipient,
    );

    let note_id = note_details.id();

    Ok((
        NoteFile::NoteDetails {
            details: note_details,
            after_block_num: 0.into(),
            tag: Some(bridge_note_tag())
        },
        note_id
    ))
}