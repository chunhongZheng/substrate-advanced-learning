use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_support::dispatch::RawOrigin;
use frame_system::ensure_signed;
use frame_support::transactional;
use super::*;
#[test]
fn create_kitty_works() {

	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let kitty_id: u32 = NextKittyId::<Test>::get();  //此时该值应该为默认值0
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));  //创建成功
        //检查各项数据
		// check that account #1 owns 1 kitty
		assert_eq!(KittiesModule::all_kitties(1).len(), 1);
		assert_eq!(KittiesModule::next_kitty_id(),1);
		assert_eq!(KittiesModule::next_kitty_id(),kitty_id.checked_add(1).unwrap());
		assert_ne!(KittiesModule::kitties(kitty_id), None);
		let acc=KittiesModule::kitty_owner(kitty_id).unwrap();
		assert_eq!(acc, account_id);

		// check that some random account #5 does not own a kitty
		assert_eq!(KittiesModule::all_kitties(5).len(), 0);
		//account1 继续创建
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));  //创建成功
		// check that account #1 owns 2 kitty
		assert_eq!(KittiesModule::all_kitties(1).len(), 2);
		assert_eq!(KittiesModule::next_kitty_id(),2);
	})
}

#[test]
fn create_kitty_not_enough_balance_should_fail(){
	new_test_ext().execute_with(|| {
		// create a kitty with account #5.
		//初始化只有0, 100), (1, 98), (2, 50)，账户5余额为0
		assert_noop!(
			KittiesModule::create(Origin::signed(5)),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn create_kitty_exceed_max_kitty_owned_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		//assert_eq!(KittiesModule::all_kitties(1).len(),<Test as Config>::KittyIndex);
		//assert_ok!(KittiesModule::create(Origin::signed(1))); //此行会报错，超出设置数量4了。
		assert_noop!(
			KittiesModule::create(Origin::signed(1)),
			Error::<Test>::ExceedMaxKittyOwned
		);
	});
}
//
#[test]
//#[transactional]
fn create_kitty_count_overflow_should_fail() {
	new_test_ext().execute_with(|| {
		//let kitty_id: u32 = NextKittyId::<Test>::get();
		NextKittyId::<Test>::put(u32::max_value());
		// create a kitty with account #1.
		//assert_err!(KittiesModule::create(Origin::signed(1)), Error::<Test>::KittyIdOverflow);
		assert_noop!(
			KittiesModule::create(Origin::signed(1)),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn create_kitty_next_kitty_id_overflow_should_fail() {
	new_test_ext().execute_with(|| {
		NextKittyId::<Test>::put(u32::max_value());
		assert_noop!(
			KittiesModule::create(Origin::signed(1)),
			Error::<Test>::InvalidKittyId
		);
		// assert_noop!(
		// 	KittiesModule::create(Origin::signed(1)),
		// 	Error::<Test>::ExceedMaxKittyOwned
		// );

	});
}

#[test]
fn breed_kitty_works(){
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		KittiesModule::create(Origin::signed(account_id));
		KittiesModule::create(Origin::signed(account_id));
		let kitty_id1= KittiesModule::all_kitties(1)[0];
		let kitty_id2= KittiesModule::all_kitties(1)[1];
		assert_ok!(KittiesModule::breed(Origin::signed(account_id), kitty_id1, kitty_id2));
	});
}

#[test]
fn breed_kitty_not_owner_should_fail(){
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		KittiesModule::create(Origin::signed(account_id));
		KittiesModule::create(Origin::signed(account_id));
		let kitty_id1= KittiesModule::all_kitties(1)[0];
		let kitty_id2= KittiesModule::all_kitties(1)[1];
		let account_id2: u64 = 2;
		assert_noop!(
			KittiesModule::breed(Origin::signed(account_id2), kitty_id1, kitty_id2),
			Error::<Test>::NotOwner
		);

	});
}