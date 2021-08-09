#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, result};

pub trait IcoHandler<CurrencyId, MulBalanceOf, AccountId, DispathErr, BlockNumber> {
    // 是否存在项目中
    fn is_project_ico_member(currency_id: CurrencyId, index: u32, who: &AccountId) -> result::Result<bool, DispathErr>;
    // 获取参与的金额(换取多少项目方代币)
    fn get_user_total_amount(currency_id: CurrencyId, index: u32, who: &AccountId) -> MulBalanceOf;
    // 获取所有成员参与ico的金额（换取项目方多少代币)
    fn get_total_ico_amount(currency_id: CurrencyId, index: u32) -> result::Result<MulBalanceOf, DispathErr>;
    // // 获取项目方开始ico的时间
    // fn get_ico_start_time(currency_id: CurrencyId) -> result::Result<BlockNumber, DispathErr>;

}