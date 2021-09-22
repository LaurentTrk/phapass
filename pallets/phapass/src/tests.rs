#![cfg(test)]

use super::mock::{
	assert_events, Event, new_test_ext, PhaPass, USER_A, Origin
};
use super::{pallet::Event as PhaPassEvent};

use frame_support::{assert_ok};

#[test]
fn vault_should_be_created() {
	new_test_ext().execute_with(|| {
		assert_ok!(PhaPass::create_vault(Origin::signed(USER_A)));
		assert_events(vec![Event::PhaPass(PhaPassEvent::VaultCreated(USER_A))]);
	})
}
