use miden_objects::account::NetworkId as NativeNetworkId;
use miden_objects::address::{
    AccountIdAddress as NativeAccountIdAddress,
    Address as NativeAddress,
    AddressInterface as NativeAddressInterface,
};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use super::account_id::{AccountId, NetworkId};
use super::note_tag::NoteTag;
use crate::js_error_with_context;

#[wasm_bindgen(inspectable)]
#[derive(Clone, Debug)]
pub struct Address(NativeAddress);

#[wasm_bindgen]
/// Specifies which procedures an account accepts, and by extension which notes it can consume.
pub enum AddressInterface {
    Unspecified = "Unspecified",
    BasicWallet = "BasicWallet",
}

#[wasm_bindgen]
impl Address {
    #[wasm_bindgen(js_name = "fromAccountId")]
    pub fn from_account_id(
        account_id: AccountId,
        interface: AddressInterface,
    ) -> Result<Self, JsValue> {
        let interface: NativeAddressInterface = interface.try_into()?;
        let address = NativeAccountIdAddress::new(account_id.into(), interface);

        Ok(Address(NativeAddress::AccountId(address)))
    }

    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(bech32: &str) -> Result<Self, JsValue> {
        let (_net_id, address) = NativeAddress::from_bech32(bech32).map_err(|err| {
            js_error_with_context(err, "could not convert bech32 into an address")
        })?;
        Ok(Self(address))
    }

    pub fn interface(&self) -> Result<AddressInterface, JsValue> {
        self.0.interface().try_into()
    }

    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> Result<AccountId, JsValue> {
        match &self.0 {
            NativeAddress::AccountId(account_id_address) => Ok(account_id_address.id().into()),
            _other => Err("Unsupported Account address type".into()),
        }
    }

    #[wasm_bindgen(js_name = "toNoteTag")]
    pub fn to_note_tag(&self) -> NoteTag {
        self.0.to_note_tag().into()
    }

    #[wasm_bindgen(js_name = "toBech32")]
    pub fn to_bech32(&self, network_id: NetworkId) -> Result<String, JsValue> {
        let net_id: NativeNetworkId = network_id.into();
        Ok(self.0.to_bech32(net_id))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAddress> for Address {
    fn from(native_address: NativeAddress) -> Self {
        Address(native_address)
    }
}

impl From<&NativeAddress> for Address {
    fn from(native_address: &NativeAddress) -> Self {
        Address(native_address.clone())
    }
}

impl From<Address> for NativeAddress {
    fn from(address: Address) -> Self {
        address.0
    }
}

impl From<&Address> for NativeAddress {
    fn from(address: &Address) -> Self {
        address.0.clone()
    }
}

impl TryFrom<AddressInterface> for NativeAddressInterface {
    type Error = &'static str;
    fn try_from(value: AddressInterface) -> Result<Self, &'static str> {
        match value {
            AddressInterface::BasicWallet => Ok(NativeAddressInterface::BasicWallet),
            AddressInterface::Unspecified => Ok(NativeAddressInterface::Unspecified),
            AddressInterface::__Invalid => Err("Non-valid address interface given"),
        }
    }
}

impl TryFrom<NativeAddressInterface> for AddressInterface {
    type Error = JsValue;
    fn try_from(value: NativeAddressInterface) -> Result<Self, Self::Error> {
        match value {
            NativeAddressInterface::BasicWallet => Ok(AddressInterface::BasicWallet),
            NativeAddressInterface::Unspecified => Ok(AddressInterface::Unspecified),
            _other => {
                Err("AddressInterface from miden-objects crate was instantiated with an unsupported value"
                    .into())
            },
        }
    }
}
