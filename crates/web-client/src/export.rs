use miden_client::Word;
use miden_client::account::AccountFile;
use miden_client::store::NoteExportType;
use miden_client::utils::{Serializable, get_public_keys_from_account};
use wasm_bindgen::prelude::*;

use crate::models::account_id::AccountId;
use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "exportNoteFile")]
    pub async fn export_note_file(
        &mut self,
        note_id: String,
        export_type: String,
    ) -> Result<JsValue, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let note_id = Word::try_from(note_id)
                .map_err(|err| js_error_with_context(err, "failed to parse input note id"))?
                .into();

            let output_note = client
                .get_output_note(note_id)
                .await
                .map_err(|err| js_error_with_context(err, "failed to get output notes"))?
                .ok_or(JsValue::from_str("No output note found"))?;

            let export_type = match export_type.as_str() {
                "Id" => NoteExportType::NoteId,
                "Full" => NoteExportType::NoteWithProof,
                "Details" => NoteExportType::NoteDetails,
                // Fail fast on unspecified/invalid export type instead of defaulting
                other => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid export type: {}. Expected one of: Id | Full | Details",
                        if other.is_empty() { "<empty>" } else { other }
                    )));
                },
            };

            let note_file = output_note.into_note_file(&export_type).map_err(|err| {
                js_error_with_context(err, "failed to convert output note to note file")
            })?;

            let input_note_bytes = note_file.to_bytes();

            let serialized_input_note_bytes = serde_wasm_bindgen::to_value(&input_note_bytes)
                .map_err(|_| JsValue::from_str("Serialization error"))?;

            Ok(serialized_input_note_bytes)
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Retrieves the entire underlying web store and returns it as a `JsValue`
    ///
    /// Meant to be used in conjunction with the `force_import_store` method
    #[wasm_bindgen(js_name = "exportStore")]
    pub async fn export_store(&mut self) -> Result<JsValue, JsValue> {
        let store = self.store.as_ref().ok_or(JsValue::from_str("Store not initialized"))?;
        let export = store
            .export_store()
            .await
            .map_err(|err| js_error_with_context(err, "failed to export store"))?;

        Ok(export)
    }

    #[wasm_bindgen(js_name = "exportAccountFile")]
    pub async fn export_account_file(&mut self, account_id: AccountId) -> Result<JsValue, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let account = client
                .get_account(account_id.into())
                .await
                .map_err(|err| {
                    js_error_with_context(
                        err,
                        &format!(
                            "failed to get account for account id: {}",
                            account_id.to_string()
                        ),
                    )
                })?
                .ok_or(JsValue::from_str("No account found"))?;

            let keystore = self.keystore.clone().expect("Keystore not initialized");
            let account_seed = account.seed().copied();
            let account = account.into();

            let mut key_pairs = vec![];

            for pub_key in get_public_keys_from_account(&account) {
                key_pairs.push(
                    keystore
                        .get_key(pub_key)
                        .await
                        .map_err(|err| {
                            js_error_with_context(err, "failed to get public key for account")
                        })?
                        .ok_or(JsValue::from_str("Auth not found for account"))?,
                );
            }

            let account_data = AccountFile::new(account, account_seed, key_pairs);

            let serialized_input_note_bytes =
                serde_wasm_bindgen::to_value(&account_data.to_bytes())
                    .map_err(|_| JsValue::from_str("Serialization error"))?;

            Ok(serialized_input_note_bytes)
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }
}
