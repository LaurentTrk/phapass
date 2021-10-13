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
		traits::{StorageVersion},
	};
	use frame_system::pallet_prelude::*;

	use pallet_mq::MessageOriginInfo;
	use phala_pallets::pallet_mq;
	use phala_types::messaging::{DecodedMessage, PhaPassCommandEvent};
	use frame_support::log::info;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_mq::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CommandExecuted(PhaPassCommandEvent),
	}

	#[pallet::error]
	pub enum Error<T> {
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
	}

	impl<T: Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}

	impl<T: Config> Pallet<T> {
		pub fn on_message_received(message: DecodedMessage<PhaPassCommandEvent>) -> DispatchResult {
			let data = message.payload;
			info!("[PhaPass] *******************************************************************************************");
			info!("[PhaPass] Message received: {:?}", &data);
			Self::deposit_event(Event::CommandExecuted(data));
			
			Ok(())
		}
	}
}
