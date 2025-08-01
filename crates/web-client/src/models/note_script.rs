use miden_client::note::{NoteScript as NativeNoteScript, WellKnownNote};
use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteScript(NativeNoteScript);

#[wasm_bindgen]
impl NoteScript {
    pub fn p2id() -> Self {
        WellKnownNote::P2ID.script().into()
    }

    pub fn p2ide() -> Self {
        WellKnownNote::P2IDE.script().into()
    }

    pub fn swap() -> Self {
        WellKnownNote::SWAP.script().into()
    }

    #[wasm_bindgen(js_name = "rootHex")]
    pub fn root_hex(&self) -> String {
        self.0.root().to_hex()
    }
}
// CONVERSIONS
// ================================================================================================

impl From<NativeNoteScript> for NoteScript {
    fn from(native_note_script: NativeNoteScript) -> Self {
        NoteScript(native_note_script)
    }
}

impl From<&NativeNoteScript> for NoteScript {
    fn from(native_note_script: &NativeNoteScript) -> Self {
        NoteScript(native_note_script.clone())
    }
}

impl From<NoteScript> for NativeNoteScript {
    fn from(note_script: NoteScript) -> Self {
        note_script.0
    }
}

impl From<&NoteScript> for NativeNoteScript {
    fn from(note_script: &NoteScript) -> Self {
        note_script.0.clone()
    }
}
