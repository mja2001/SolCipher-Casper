#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    ApiError, BlockTime, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPoints, Key, Parameter, U256, runtime_args, RuntimeArgs,
};

const ENTRY_POINT_CREATE_SHARE: &str = "create_share";
const ENTRY_POINT_REVOKE_SHARE: &str = "revoke_share";
const ENTRY_POINT_GET_CID_IF_ALLOWED: &str = "get_cid_if_allowed";

const METADATA_DICT: &str = "metadata";
const RECIPIENTS_DICT: &str = "recipients";
const COUNTER_KEY: &str = "share_counter";

#[repr(u16)]
enum CustomError {
    EmptyRecipients = 100,
    NotOwner = 101,
    ShareNotFound = 102,
    RevokedOrExpired = 103,
    NoAccess = 104,
}

impl From<CustomError> for ApiError {
    fn from(e: CustomError) -> Self {
        ApiError::User(e as u64)
    }
}

#[no_mangle]
pub extern "C" fn create_share() {
    let cid: String = runtime::get_named_arg("cid");
    let recipients: Vec<Key> = runtime::get_named_arg("recipients");
    let expiry: BlockTime = runtime::get_named_arg("expiry");

    if recipients.is_empty() {
        runtime::revert(CustomError::EmptyRecipients);
    }

    let owner = runtime::get_caller();

    let share_id = increment_counter();

    let metadata_uref = storage::new_dictionary(METADATA_DICT).unwrap_or_revert();
    let recipients_uref = storage::new_dictionary(RECIPIENTS_DICT).unwrap_or_revert();

    storage::dictionary_put(
        metadata_uref,
        &share_id.to_string(),
        (cid.clone(), owner, expiry, false), // (cid, owner, expiry, revoked)
    );

    storage::dictionary_put(recipients_uref, &share_id.to_string(), recipients);

    // Optional: emit event
}

#[no_mangle]
pub extern "C" fn revoke_share() {
    let share_id: U256 = runtime::get_named_arg("share_id");
    let caller = runtime::get_caller();

    let metadata_uref = get_metadata_dict();
    let (cid, owner, expiry, revoked): (String, Key, BlockTime, bool) = storage::dictionary_get(metadata_uref, &share_id.to_string())
        .unwrap_or_revert()
        .unwrap_or_revert_with(CustomError::ShareNotFound);

    if caller != owner {
        runtime::revert(CustomError::NotOwner);
    }

    storage::dictionary_put(metadata_uref, &share_id.to_string(), (cid, owner, expiry, true));
}

#[no_mangle]
pub extern "C" fn get_cid_if_allowed() {
    let share_id: U256 = runtime::get_named_arg("share_id");
    let caller = runtime::get_caller();

    let metadata_uref = get_metadata_dict();
    let recipients_uref = get_recipients_dict();

    let (cid, owner, expiry, revoked): (String, Key, BlockTime, bool) = storage::dictionary_get(metadata_uref, &share_id.to_string())
        .unwrap_or_revert()
        .unwrap_or_revert_with(CustomError::ShareNotFound);

    if revoked || runtime::get_block_time() > expiry {
        runtime::revert(CustomError::RevokedOrExpired);
    }

    let recipients: Vec<Key> = storage::dictionary_get(recipients_uref, &share_id.to_string())
        .unwrap_or_revert()
        .unwrap_or_revert_with(CustomError::ShareNotFound);

    if caller != owner && !recipients.contains(&caller) {
        runtime::revert(CustomError::NoAccess);
    }

    runtime::ret(CLValue::from_t(cid).unwrap_or_revert())
}

fn get_metadata_dict() -> URef {
    match runtime::get_key(METADATA_DICT) {
        Some(Key::URef(uref)) => uref,
        _ => storage::new_dictionary(METADATA_DICT).unwrap_or_revert(),
    }
}

fn get_recipients_dict() -> URef {
    match runtime::get_key(RECIPIENTS_DICT) {
        Some(Key::URef(uref)) => uref,
        _ => storage::new_dictionary(RECIPIENTS_DICT).unwrap_or_revert(),
    }
}

fn increment_counter() -> U256 {
    let counter_uref = match runtime::get_key(COUNTER_KEY) {
        Some(key) => key,
        None => {
            let new_uref = storage::new_uref(U256::zero());
            runtime::put_key(COUNTER_KEY, new_uref.into());
            new_uref
        }
        _ => runtime::revert(ApiError::UnexpectedKeyVariant),
    };

    let mut counter: U256 = storage::read(counter_uref).unwrap_or_revert().unwrap_or_revert();
    counter += U256::one();
    storage::write(counter_uref, counter);
    counter
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_CREATE_SHARE,
        vec![
            Parameter::new("cid", String::cl_type()),
            Parameter::new("recipients", Vec::<Key>::cl_type()),
            Parameter::new("expiry", BlockTime::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_REVOKE_SHARE,
        vec![Parameter::new("share_id", U256::cl_type())],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_GET_CID_IF_ALLOWED,
        vec![Parameter::new("share_id", U256::cl_type())],
        String::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    let (contract_hash, _) = storage::new_contract(entry_points, None, None, None);

    runtime::put_key("solcipher_contract_hash", contract_hash.into());
}
