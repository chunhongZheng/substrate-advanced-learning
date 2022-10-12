#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    pub use frame_support::pallet_prelude::*;
    pub use frame_system::pallet_prelude::*;
    use super::WeightInfo;
    pub use sp_std::vec::Vec;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        #[pallet::constant]
        type MaxClaimLength: Get<u32>;

        /// Information on runtime weights.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://docs.substrate.io/v3/runtime/storage
    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    pub(super) type Proofs<T: Config> =
    StorageMap<_, Blake2_128Concat, BoundedVec<u8,T::MaxClaimLength>, (T::AccountId, T::BlockNumber)>;

    // Hooks
    // Define some logic that should be executed
    // regularly in some context, for e.g. on_initialize.
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}


    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, Vec<u8>),
        /// Event emitted when a claim is revoked by the owner. [who, claim]
        ClaimRevoked(T::AccountId, Vec<u8>),
        /// Event emitted when a claim is transferred by the owner. [from_who, to_who, claim]
        ClaimTransferred(T::AccountId, T::AccountId, Vec<u8>),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// The proof has already been claimed.
        ProofAlreadyExist,
        /// The proof does not exist, so it cannot be revoked.
        NoSuchClaim,
        /// The proof is claimed by another account, so caller can't revoke it.
        NotClaimOwner,
        /// The proof is too long.
        ClaimTooLong,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
      //  #[pallet::weight(0)]
        #[pallet::weight(T::WeightInfo::create_claim(_claim.len() as u32))]
        pub fn create_claim(_origin: OriginFor<T>, _claim: Vec<u8>) -> DispatchResultWithPostInfo {

             let sender = ensure_signed(_origin)?;
             let bounded_claim = BoundedVec::<u8,T::MaxClaimLength>::try_from(_claim.clone())
                 .map_err(|_| Error::<T>::ClaimTooLong)?;
            // // Verify that the specified proof has not already been claimed.
             ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);
            //
            // // Get the block number from the FRAME System pallet.
             let current_block = <frame_system::Pallet<T>>::block_number();
            //
            // // Store the proof with the sender and block number.
             Proofs::<T>::insert(&bounded_claim, (sender.clone(), current_block));
            //
            // // Emit an event.
             Self::deposit_event(Event::ClaimCreated(sender, _claim));

            Ok(().into())
        }

      //  #[pallet::weight(0)]
        #[pallet::weight(T::WeightInfo::revoke_claim(_claim.len() as u32))]
        pub fn revoke_claim(_origin: OriginFor<T>, _claim: Vec<u8>) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let sender = ensure_signed(_origin)?;
            let bounded_claim = BoundedVec::<u8,<T as Config>::MaxClaimLength>::try_from(_claim.clone())
                .map_err(|_| Error::<T>::ClaimTooLong)?;


            // Verify that the specified proof has been claimed.
            // Get owner of the claim.
            let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::NoSuchClaim)?;

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotClaimOwner);

            // Remove claim from storage.
            Proofs::<T>::remove(&bounded_claim);

            // Emit an event that the claim was erased.
            Self::deposit_event(Event::ClaimRevoked(sender, _claim));

            Ok(().into())
        }

       // #[pallet::weight(0)]
        #[pallet::weight(T::WeightInfo::transfer_claim(_claim.len() as u32))]
        pub fn transfer_claim(
            origin: OriginFor<T>,
            to: T::AccountId,
            _claim: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let sender = ensure_signed(origin)?;
            let bounded_claim = BoundedVec::<u8,<T as Config>::MaxClaimLength>::try_from(_claim.clone())
                .map_err(|_| Error::<T>::ClaimTooLong)?;


            // Verify that the specified proof has been claimed.
            // Get owner of the claim.
            let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::NoSuchClaim)?;

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotClaimOwner);
            // Get the block number from the FRAME System pallet.
            let current_block = <frame_system::Pallet<T>>::block_number();
            Proofs::<T>::insert(
                &bounded_claim,
                (to.clone(),current_block)
            );

            // Emit an event that the claim was transferred.
            Self::deposit_event(Event::ClaimTransferred(sender, to, _claim));

            Ok(().into())
        }

    }
}
