#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::traits::ExistenceRequirement;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*,traits::Randomness};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;

	use frame_support::traits::Currency;
    use frame_support::traits::ReservableCurrency;

	#[derive(Encode,Decode)]
	pub struct Kitty(pub [u8;16]);
	type KittyIndex =u32;

	type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash,Self::BlockNumber>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T> = StorageValue<_, u32>;
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_,Blake2_128Concat,KittyIndex,Option<Kitty>,ValueQuery>;
    
    #[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T:Config> = StorageMap<_,Blake2_128Concat,KittyIndex,Option<T::AccountId>,ValueQuery>;

	#[pallet::storage]
    #[pallet::getter(fn kitty_prices)]
    pub type KittyPrices<T: Config> =
        StorageMap<_, Blake2_128Concat, KittyIndex, Option<BalanceOf<T>>, ValueQuery>;



	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreate(T::AccountId, KittyIndex),
		KittyTransfer(T::AccountId, T::AccountId, KittyIndex),
		KittyForSale(T::AccountId, KittyIndex, Option<BalanceOf<T>>),
        KittySaleOut(T::AccountId, KittyIndex, Option<BalanceOf<T>>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
		AlreadyOwned,
		SameParentIndex,
		InvalidKittyIndex,
        NotForSale,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				},
				None => 0
			};

			let dna = Self::random_value(&who);

			Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));

			Owner::<T>::insert(kitty_id, Some(who.clone()));

			KittiesCount::<T>::put(kitty_id + 1);

			Self::deposit_event(Event::KittyCreate(who, kitty_id));

			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: KittyIndex) ->
			DispatchResultWithPostInfo
		{
			let who = ensure_signed(origin)?;
			ensure!(
                Some(who.clone()) != Some(new_owner.clone()),
                Error::<T>::AlreadyOwned
            );

			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

			Owner::<T>::insert(kitty_id, Some(new_owner.clone()));

			Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));

			Ok(().into())
		}
		#[pallet::weight(0)]
        pub fn bread(
            origin: OriginFor<T>,
            kitty_id_1: KittyIndex,
            kitty_id_2: KittyIndex,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				},
				None => 0
			};
            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

            let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
            let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;
            let dna_1 = kitty1.0;
            let dna_2 = kitty2.0;

            let selector = Self::random_value(&who);
            let mut new_dna = [0u8; 16];

            for i in 0..dna_1.len() {
                new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
            }

            Kitties::<T>::insert(kitty_id, Some(Kitty(new_dna)));
            Owner::<T>::insert(kitty_id, Some(who.clone()));
            KittiesCount::<T>::put(kitty_id + 1);
            Self::deposit_event(Event::KittyCreate(who, kitty_id));
            Ok(())
        }
		#[pallet::weight(0)]
        pub fn sale(
            origin: OriginFor<T>,
            kitty_id: KittyIndex,
            sale_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Some(who.clone()) == Owner::<T>::get(kitty_id),
                Error::<T>::NotOwner
            );

            KittyPrices::<T>::insert(kitty_id, sale_price);

            Self::deposit_event(Event::KittyForSale(who, kitty_id, sale_price));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn buy(origin: OriginFor<T>, kitty_id: KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let kitty_owner = Owner::<T>::get(kitty_id).ok_or(Error::<T>::NotOwner)?;
            let kitty_price = KittyPrices::<T>::get(kitty_id).ok_or(Error::<T>::NotForSale)?;
            ensure!(
                Some(who.clone()) != Some(kitty_owner.clone()),
                Error::<T>::AlreadyOwned
            );
            //转账（购买）
            T::Currency::transfer(
                &who,
                &kitty_owner,
                kitty_price,
                ExistenceRequirement::KeepAlive,
            )?;
            //更改拥有人
            Owner::<T>::insert(kitty_id, Some(who.clone()));
            //移除挂售
            KittyPrices::<T>::remove(kitty_id);
            Self::deposit_event(Event::KittySaleOut(who, kitty_id, Some(kitty_price)));
            Ok(())
        }



		
	}
	impl<T:Config> Pallet<T> {
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}
	}
}
