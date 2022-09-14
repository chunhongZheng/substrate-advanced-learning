#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	//use std::fmt::Debug;
	use frame_support::pallet_prelude::*;
    use frame_support::{log, sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, CheckedAdd}, traits::{Currency, Randomness, ReservableCurrency}};
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
	// #[cfg(feature = "std")]
    // use frame_support::serde::{Deserialize, Serialize};
	//use sp_core::blake2_128;

	//type KittyIndex = u32;


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		/// The type of Kitty ID
		//type KittyIndex: Parameter+AtLeast32BitUnsigned + Copy + Parameter + Default + Bounded + MaxEncodedLen+CheckedAdd;
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded + CheckedAdd + MaxEncodedLen;
		/// The Currency handler for the Kitties pallet.
		//type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type Currency: ReservableCurrency<Self::AccountId>;
		#[pallet::constant]
		type MaxKittyIndex: Get<u32>;
		/// The staking balance when create_kitty
		#[pallet::constant]
		type ReserveForCreateKitty: Get<BalanceOf<Self>>;
	}

	#[pallet::type_value]
	pub fn GetDefaultValue<T: Config>() -> T::KittyIndex {
		0_u8.into()
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	//type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, T::KittyIndex,ValueQuery, GetDefaultValue<T>>;
	//pub type NextKittyId<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery, GetDefaultValue>;
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T:Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;
    //
	 #[pallet::storage]
	 #[pallet::getter(fn kitty_owner)]
	 pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;


	#[pallet::storage]
	#[pallet::getter(fn all_kitties)]
	pub type AllKitties<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::KittyIndex, T::MaxKittyIndex>,
		ValueQuery,
	>;

	// // Our pallet's genesis configuration.
	// #[pallet::genesis_config]
	// pub struct GenesisConfig<T: Config> {
	// 	pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>,
	// }
	//
	// // Required to implement default for GenesisConfig.
	// #[cfg(feature = "std")]
	// impl<T: Config> Default for GenesisConfig<T> {
	// 	fn default() -> GenesisConfig<T> {
	// 		GenesisConfig { kitties: vec![] }
	// 	}
	// }




	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex),
		KittyBred(T::AccountId, T::KittyIndex),
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		NotOwner,
		SameKittyId,
		KittyIdOverflow,
		KittyCntOverflow,
		ExceedMaxKittyOwned,
		InvalidReserveAmount,
		NotEnoughBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_price = T::ReserveForCreateKitty::get();
			ensure!(T::Currency::can_reserve(&who, kitty_price), Error::<T>::NotEnoughBalance);
			T::Currency::reserve(&who, kitty_price)?;
			 let dna = Self::random_value(&who);
			//let dna = Self::random_value();
			 let kitty_id=Self::mint(&who,dna)?;
			// // Logging to the console
			 log::info!("A kitty is born with ID: {:?}.", kitty_id);
			 Self::deposit_event(Event::KittyCreated(who, kitty_id));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_price = T::ReserveForCreateKitty::get();
			ensure!(T::Currency::can_reserve(&who, kitty_price), Error::<T>::NotEnoughBalance);
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
			let kitty_1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;
			//check owner
			ensure!(Self::kitty_owner(kitty_id_1) == Some(who.clone()), Error::<T>::NotOwner);
			ensure!(Self::kitty_owner(kitty_id_2) == Some(who.clone()), Error::<T>::NotOwner);
			let selector = Self::random_value(&who);
			//let selector = Self::random_value();
			let mut data = [0u8; 16];
			for i in 0..kitty_1.0.len() {
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			}
			let kitty_id=Self::mint(&who,data)?;
			// Logging to the console
			// log::info!("parent1 {:?} and parent2 {:?} breed a new kitty  with ID: {:?}.", kitty_id_1,kitty_id_2,kitty_id);
			Self::deposit_event(Event::KittyBred(who, kitty_id));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(origin: OriginFor<T>, kitty_id: T::KittyIndex, new_owner: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_price = T::ReserveForCreateKitty::get();
			ensure!(T::Currency::can_reserve(&who, kitty_price), Error::<T>::NotEnoughBalance);
			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);

			T::Currency::unreserve(&who, kitty_price);
			T::Currency::reserve(&new_owner, kitty_price)?;
			KittyOwner::<T>::insert(kitty_id, &new_owner);
			//移除原拥有者对应kitty数据
			AllKitties::<T>::try_mutate(&who, |ref mut kitties| {
				let index = kitties.iter().position(|&r| r == kitty_id).unwrap();
				kitties.remove(index);
				Ok::<(), DispatchError>(())
			})?;

			AllKitties::<T>::try_mutate(&new_owner, |ref mut kitties| {
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::KittyTransferred(who,new_owner, kitty_id));
			Ok(())
		}		
	}

	impl<T: Config> Pallet<T> {

		// fn random_value() -> [u8; 16] {
		// 	let payload = (
		// 		T::Randomness::random(&b"dna"[..]).0,
		// 		<frame_system::Pallet<T>>::extrinsic_index(),
		// 	);
		// 	payload.using_encoded(sp_io::hashing::blake2_128)
		// }

		// get a random 256.
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				//Self::random(&[][..]),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			payload.using_encoded(sp_io::hashing::blake2_128)
		}
		// get next id
		fn get_next_id() -> Result<T::KittyIndex, ()> {
			let kitty_id = Self::next_kitty_id();
			match kitty_id {
				_ if T::KittyIndex::max_value() <= kitty_id => Err(()),
				val => Ok(val),
			}
		}
		// mint kitty
		pub fn mint(
			owner: &T::AccountId,
			dna: [u8; 16],
		) -> Result<T::KittyIndex, Error<T>>{
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty = Kitty(dna);
			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &owner);
			//KittyIdOverflow
			let next_kitty_id = kitty_id
				.checked_add(&(T::KittyIndex::from(1_u8)))
				.ok_or(Error::<T>::KittyIdOverflow)
				.unwrap();
			NextKittyId::<T>::set(next_kitty_id);
			<AllKitties<T>>::try_mutate(&owner, |kitty_vec| kitty_vec.try_push(kitty_id))
				.map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;
			Ok(kitty_id)
		}
		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}

	}
}
