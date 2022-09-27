#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use frame_system::{
    offchain::{
        AppCrypto, CreateSignedTransaction, SendSignedTransaction,
        Signer,
    },
};


use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");
pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_TYPE);

    pub struct OcwAuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
        type RuntimeAppPublic = Public;
        type GenericPublic = sp_core::sr25519::Public;
        type GenericSignature = sp_core::sr25519::Signature;
    }

    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
    for OcwAuthId
    {
        type RuntimeAppPublic = Public;
        type GenericPublic = sp_core::sr25519::Public;
        type GenericSignature = sp_core::sr25519::Signature;
    }
}


#[frame_support::pallet]
pub mod pallet{
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use serde::{Deserialize, Deserializer};
    use sp_io::offchain_index;
    use sp_runtime::offchain::storage::StorageValueRef;
    use sp_runtime::traits::Zero;
    use sp_std::vec::Vec;
    // #[derive(Debug, Deserialize, Encode, Decode, Default)]
    // struct IndexingData(Vec<u8>, u64);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);


    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        WriteSignDataSuccess(Vec<u8>,  ([u8;32], u64)),
    }

    #[pallet::error]
    pub enum Error<T> {

    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>{
        fn offchain_worker(block_number: T::BlockNumber) {
            log::info!("offchain_index pallet offchain workers start!: {:?}", block_number);
            //奇数区块使用offChain_index向链下存储写入数据
            if block_number % 2u32.into() != Zero::zero(){
                let random_slice = sp_io::offchain::random_seed();
                log::info!("in odd block, value to  local random_slice write: {:?}", random_slice);
                //  get a local timestamp
                let timestamp_u64 = sp_io::offchain::timestamp().unix_millis();
                log::info!("in odd block, value to  local timestamp write: {:?}", timestamp_u64);
                // combine to a tuple and print it
                let value = (random_slice, timestamp_u64);
                log::info!("in odd block, value to write: {:?}", value);
                //发送线上交易，通过offchainIndex写入链下存储
            //    let payload: Vec<u8> = vec![1,2,3,4,5,6,7,8];
                _ = Self::send_signed_tx(value);
                //  write or mutate tuple content to key
              //  val_ref.set(&value);
            }else{
                //偶数 even
                let key = Self::derive_key(block_number - 1u32.into());
                let mut val_ref = StorageValueRef::persistent(&key);
                if let Ok(Some(value)) = val_ref.get::<([u8;32], u64)>() {
                    // print values
                    log::info!("in even block, value read: {:?}", value);
                    // delete that key
                    val_ref.clear();
                }
            }

        }
    }


    #[pallet::call]
    impl<T: Config> Pallet<T>{
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn do_something(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }
        #[pallet::weight(0)]
        pub fn submit_data(origin: OriginFor<T>, value: ([u8;32], u64)) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            log::info!("in submit_data call: {:?}", value);
            // Off-chain indexing write
            let key = Self::derive_key(frame_system::Module::<T>::block_number());
            log::info!("in submit_data key: {:?}", key);
            offchain_index::set(&key, &value.encode());
            log::info!("Leave from submit_data function!: {:?}", frame_system::Module::<T>::block_number());
            //添加事件
            // Emit an event.
            Self::deposit_event(Event::WriteSignDataSuccess(key, value.clone()));
            Ok(().into())
        }

    }


    impl<T: Config> Pallet<T>{
        #[deny(clippy::clone_double_ref)]
        fn derive_key(block_number: T::BlockNumber) -> Vec<u8> {
            block_number.using_encoded(|encoded_bn| {
                log::info!("using_encoded {:?}",encoded_bn);
                b"node-template::storage::"
                    .iter()
                    .chain(encoded_bn)
                    .copied()
                    .collect::<Vec<u8>>()
            })
        }
        fn send_signed_tx(value: ([u8;32], u64)) -> Result<(), &'static str> {
            let signer = Signer::<T, T::AuthorityId>::all_accounts();
            if !signer.can_sign() {
                return Err(
                    "No local accounts available. Consider adding one via `author_insertKey` RPC.",
                )
            }

            let results = signer.send_signed_transaction(|_account| {

                Call::submit_data { value: value.clone() }
            });

            for (acc, res) in &results {
                match res {
                    Ok(()) => log::info!("[{:?}] Submitted data:{:?}", acc.id, value),
                    Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
                }
            }

            Ok(())
        }
    }

}