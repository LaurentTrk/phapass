// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, StorageVersion},
	};
	use frame_system::pallet_prelude::*;

	use pallet_mq::MessageOriginInfo;
	use phala_pallets::pallet_mq;
	use phala_types::messaging::{DecodedMessage, Vault};

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_mq::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [owner]
		VaultCreated(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NotOwner,
	}

	#[pallet::storage]
	#[pallet::getter(fn vaults)]
	pub type Vaults<T: Config> =
		StorageMap<_, Blake2_256, T::AccountId, u32, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new vault
		#[pallet::weight(195_000_000)]
		pub fn create_vault(
			origin: OriginFor<T>
		) -> DispatchResult {
			let source = ensure_signed(origin)?;
			Self::deposit_event(Event::VaultCreated(source));
			Ok(())
		}

	}

	impl<T: Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}

	impl<T: Config> Pallet<T> {
		pub fn on_message_received(_message: DecodedMessage<Vault<T::AccountId>>) -> DispatchResult {
			// TODO
			Ok(())
		}
	}
}
