use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;
use frame_support::traits::{OnFinalize, OnInitialize}; 
pub const ALICE: u64 = 1;
pub const BOB:u64 =2;


fn run_to_block( n: u64) {
	while System::block_number() < n {
		KittiesModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number()+1);
		System::on_initialize(System::block_number());
		KittiesModule::on_initialize(System::block_number());
	}
}
#[test]
fn  create_kitty_success(){
    new_test_ext().execute_with(||{
		   run_to_block(2);
           assert_ok!(KittiesModule::create(Origin::signed(ALICE)));
		   //check kitty count
           assert_eq!( KittiesCount::<Test>::get(),Some(1));
		   //check KittyCreate event
		   System::assert_last_event(mock::Event::KittiesModule(crate::Event::KittyCreate(ALICE,0)));
    })
}
#[test]
fn create_kitty_failed_when_kittyindex_exceed_maxvalue(){
	new_test_ext().execute_with(||{
		type KittyIndex = u32;
		KittiesCount::<Test>::put(KittyIndex::max_value());
		assert_noop!(
			KittiesModule::create(Origin::signed(ALICE)),
			Error::<Test>::KittiesCountOverflow
	    );     	
 })
}
#[test]
fn tranfer_kitty_success(){
	new_test_ext().execute_with(||{
		run_to_block(2);
		assert_ok!(KittiesModule::create(Origin::signed(ALICE)));
		assert_eq!(KittiesCount::<Test>::get(), Some(1));
		assert_ok!(KittiesModule::transfer(Origin::signed(ALICE), BOB, 0));
		System::assert_last_event(mock::Event::KittiesModule(crate::Event::KittyTransfer(
			ALICE, BOB, 0,
		)));
 })		
}
#[test]
fn tranfer_kitty_failed_when_not_owner(){
	new_test_ext().execute_with(|| {
        assert_noop! {
            KittiesModule::transfer(Origin::signed(ALICE),BOB,99),
            Error::<Test>::NotOwner
        }
    });
}
#[test]
fn tranfer_kitty_failed_when_already_owned(){
	new_test_ext().execute_with(|| {
        assert_noop! {
            KittiesModule::transfer(Origin::signed(ALICE),ALICE,0),
            Error::<Test>::AlreadyOwned
        }
    });
}
#[test]
fn bread_kitty_success(){
	new_test_ext().execute_with(|| {
		run_to_block(2);
        assert_ok!(KittiesModule::create(Origin::signed(ALICE)));
        assert_ok!(KittiesModule::create(Origin::signed(ALICE)));
        assert_eq!(KittiesCount::<Test>::get(), Some(2));
        assert_eq!(Owner::<Test>::get(0), Some(ALICE));
        assert_eq!(Owner::<Test>::get(1), Some(ALICE));
        assert_ok!(KittiesModule::bread(Origin::signed(ALICE), 0, 1));
        System::assert_last_event(mock::Event::KittiesModule(crate::Event::KittyCreate(
            ALICE, 2,
        )));
        assert_eq!(KittiesCount::<Test>::get(), Some(3));
        assert_eq!(Owner::<Test>::get(2), Some(ALICE));
    });	
}
#[test]
fn bread_kitty_failed_when_same_parent_index(){
	new_test_ext().execute_with(|| {
		assert_noop! {
            KittiesModule::bread(Origin::signed(ALICE),1,1),
            Error::<Test>::SameParentIndex
        }
    });
}
#[test]
fn bread_kitty_failed_when_invalid_kittyindex(){
	new_test_ext().execute_with(|| {
		assert_noop! {
			KittiesModule::bread(Origin::signed(ALICE),0,1),
			Error::<Test>::InvalidKittyIndex
		}
    });
}
#[test]
fn sale_kitty_success(){
	new_test_ext().execute_with(|| {
		run_to_block(2);
        assert_ok!(KittiesModule::create(Origin::signed(ALICE)));
        assert_ok!(KittiesModule::sale(Origin::signed(ALICE), 0, Some(5_000)));
        System::assert_last_event(mock::Event::KittiesModule(crate::Event::KittyForSale(
            ALICE,
            0,
            Some(5_000),
        )));
    });
}
#[test]
fn sale_kitty_failed_when_not_owner(){
    new_test_ext().execute_with(|| {
		assert_noop! {
			KittiesModule::sale(Origin::signed(ALICE),0,Some(5_000)),
			Error::<Test>::NotOwner
		}
    });
}
#[test]
fn buy_kitty_success(){
    new_test_ext().execute_with(|| {
        //创建Kittiy
        assert_ok!(KittiesModule::create(Origin::signed(ALICE)));
        //挂售Kittiy
        assert_ok!(KittiesModule::sale(Origin::signed(ALICE), 0, Some(8_000)));
        //检查拥有者
        assert_eq!(Owner::<Test>::get(0), Some(ALICE));
        //检查挂单
        assert_eq!(KittyPrices::<Test>::get(0), Some(8_000));
        //购买Kittiy
        assert_ok!(KittiesModule::buy(Origin::signed(BOB), 0));
        //检查事件
        System::assert_last_event(mock::Event::KittiesModule(crate::Event::KittySaleOut(
            BOB,
            0,
            Some(8_000),
        )));

        //检查是否已经收到转账
        assert_eq!(Balances::free_balance(ALICE), 10_000 + 8_000);
        //检查是否已经转出
        assert_eq!(
            Balances::free_balance(BOB),
            20_000 - 8_000
        );
        //检查拥有者
        assert_eq!(Owner::<Test>::get(0), Some(BOB));
        //检查挂单
        assert_eq!(KittyPrices::<Test>::get(0), None);
    });
}

