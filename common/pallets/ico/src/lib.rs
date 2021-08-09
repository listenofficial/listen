#![cfg_attr(not(feature = "std"), no_std)]

pub use frame_system::{self as system, ensure_signed, ensure_none};
pub use frame_support::{traits::{Get, Currency, ReservableCurrency, LockIdentifier, EnsureOrigin, ExistenceRequirement::{KeepAlive, AllowDeath}, WithdrawReasons, OnUnbalanced, BalanceStatus as Status},
					PalletId,
					debug, ensure, decl_module, runtime_print, decl_storage, decl_error, decl_event, weights::{Weight}, StorageValue, StorageMap, StorageDoubleMap, IterableStorageDoubleMap, IterableStorageMap, Blake2_256};
use sp_std::{prelude::*, result, collections::btree_map::BTreeMap};
use sp_runtime::{traits::{AccountIdConversion, Saturating, CheckedDiv, Zero}, DispatchResult, Percent, RuntimeDebug, traits::CheckedMul, DispatchError, SaturatedConversion};
use codec::{Encode, Decode};
pub use primitive_types::U256;
pub use sp_std::convert::{TryInto,TryFrom, Into};
use sp_std::vec::Vec;
use orml_tokens::{self as tokens, Locks};
use orml_traits::{MultiCurrency, MultiReservableCurrency, BalanceStatus};
use orml_tokens::BalanceLock;
use sp_runtime::app_crypto::sp_core::sandbox::ERR_EXECUTION;
use frame_support::traits::ExistenceRequirement;
use crate::Error::{Icoterminated, CurrencyIdErr};
use sp_runtime::traits::CheckedAdd;

pub mod ico_trait;

pub mod mock;
pub mod tests;

use ico_trait::IcoHandler;
use currencies::{self, currencies_trait::CurrenciesHandler, DicoAssetMetadata};
use frame_system::ensure_root;

const ICO_ID: LockIdentifier = *b"ico     ";

const ChangeReward:  u128 = 200_000_000u128 * Usdt;

const Usdt: u128 = 1000_000_000_000_000_000u128;

pub(crate) type CurrencyIdOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
pub(crate) type MultiBalanceOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
type BalanceOf<T> = <<T as Config>::NativeCurrency as Currency<<T as system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Config>::NativeCurrency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;


#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone)]
pub enum Countries {
    China,
    USA,
}

impl Default for Countries {
    fn default() -> Self {
        Self::China
    }
}


#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone, Copy, Ord, PartialOrd, Eq)]
pub enum CurrencyIds {
    BTC,
    ETH,
    USDT,
    DOT,
    KSM,
    LT,
    Id(u32),
}

impl Default for CurrencyIds {
    fn default() -> Self {
        CurrencyIds::LT
    }
}


impl Into<u32> for CurrencyIds {
    fn into(self) -> u32 {
        match self {
            CurrencyIds::LT => 0,
            CurrencyIds::BTC => 1,
            CurrencyIds::ETH => 2,
            CurrencyIds::USDT => 3,
            CurrencyIds::DOT => 4,
            CurrencyIds::KSM => 5,
            CurrencyIds::Id(x) => x,
        }
    }
}


#[derive(PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, Clone)]
pub struct Release<AccountId, Block, CurrencyId, NativeBalance> {
    who: AccountId,  // 谁请求释放
    currency_id: CurrencyId,  // 资产id
    index: u32, // ico索引
    request_time: Block, // 请求释放的时间
    // amount: Balance,
    percent: Percent,  // 请求释放到多少比例
    pledge: NativeBalance,  // 请求时需要抵押的金额
}


#[derive(PartialEq, Encode, Decode, Default, RuntimeDebug, Clone)]
pub struct IcoLock<Balance, BlockNumber> {
    start_block: BlockNumber,  // 琐仓开始的时间
    index: u32,  // ico索引
    total_amount: Balance,  // 琐仓总金额
    unlock_amount: Balance,  // 已经解锁的金额
    unlock_duration: BlockNumber,  // 多久解锁一次
    per_duration_unlock_amount: Balance,  // 一次解锁多少

}


#[derive(PartialEq, Encode, Decode, Default, RuntimeDebug, Clone)]
pub struct IcoParameters<BlockNumber, Balance, CurrencyId, Countries> {
    currency_id: CurrencyId,  // 项目方代币的资产id
    logo_url: Vec<u8>, // logo地址
    official_website: Vec<u8>,  // 官网地址
    user_ico_max_count: u8, // 用户参与ico的最大次数
    is_must_kyc: bool,  // 用户是否必须kyc
    total_issuance: Balance,  // 项目方代币的总发行量
    total_circulation: Balance,  // 项目方代币的流通量
    ico_duration: BlockNumber,  // ico存活的时间
    total_ico_amount: Balance,  // 拿出来ico的币的数量（项目方的币)
    user_min_amount: Balance,  // 每个人最小的金额（用主流币作为计量单位)
    user_max_amount: Balance,  // 每个人最大的金额（用主流币作为计量单位)
    exchange_token: CurrencyId,  // 主流币的资产id
    total_exchange_amount: Balance,  // 兑换的主流币数量
    exclude_nation: Vec<Countries>,  // 不能参与ico的国家
    lock_proportion: Percent,  // 琐仓比例
    unlock_duration: BlockNumber, // 多久解锁一次
    per_duration_unlock_amount: Balance,  // 一次解锁多少
}



#[derive(PartialEq, Encode, Decode, Default, RuntimeDebug, Clone)]
pub struct UnRelease<MultiBalanceOf, CurrencyIdOf> {
    currency_id: CurrencyIdOf, //该ico的项目方资产id
    index: u32, // 该ico的索引
    unreleased_currency_id: CurrencyIdOf,  // 还未释放的资产id
    total_usdt: MultiBalanceOf,  // 这些代币换算成的等值usdt是多少
    tags: Vec<(MultiBalanceOf, MultiBalanceOf)>, // 参加ico时候的标记总金额(用于奖励)
    total: MultiBalanceOf,  // ico总共资产数量
    released: MultiBalanceOf, // ico已经释放的资产数量
    is_terminated: bool,  // 项目强制终止ico后， 时候已经领取了剩余的资产
    is_already_get_reward: Option<bool>,  // 是否已经领取奖励
}


#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone)]
pub struct PendingInfo<IcoInfo, Balance> {
    pub ico: IcoInfo,  // ico信息
    pub pledge: Balance,  // 发起ico需要抵押的金额

}


#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone)]
pub struct IcoMultiple {
    pub numerator: u32,  // 分子
    pub denominator: u32 // 分母
}

impl Default for IcoMultiple {
    fn default() -> Self {
        IcoMultiple {
            numerator: 20,
            denominator: 10,
        }
    }
}


#[derive(PartialEq, Encode, Decode, Default, RuntimeDebug, Clone)]
pub struct IcoInfo<BlockNumber, Balance, CurrencyId, Countries, AccountId> {
    start_time: Option<BlockNumber>,  // 发起的时间
    is_already_kyc: bool,  // 项目是否已经做了kyc
    initiator: AccountId, // 项目发起人
    total_usdt: Balance, // ico总金额折算成usdt
    is_terminated: bool,  // 是否被强制终止
    project_name: Vec<u8>, // 项目名称
    token_symbol: Vec<u8>,  // 代币名称
    decimals: u8, // 代币精度
    index: Option<u32>, // ico id
    already_released_proportion: Percent, // 已经允许释放的资产比例

    currency_id: CurrencyId,  // 项目方代币的资产id
    logo_url: Vec<u8>, // logo地址
    official_website: Vec<u8>,  // 官网地址
    user_ico_max_count: u8, // 用户参与ico的最大次数
    is_must_kyc: bool,  // 用户是否必须kyc
    total_issuance: Balance,  // 项目方代币的总发行量
    total_circulation: Balance,  // 项目方代币的流通量
    ico_duration: BlockNumber,  // ico存活的时间
    total_ico_amount: Balance,  // 拿出来ico的币的数量（项目方的币)
    user_min_amount: Balance,  // 每个人最小的金额（用主流币作为计量单位)
    user_max_amount: Balance,  // 每个人最大的金额（用主流币作为计量单位)
    exchange_token: CurrencyId,  // 主流币的资产id
    total_exchange_amount: Balance,  // 兑换的主流币数量
    exclude_nation: Vec<Countries>,  // 不能参与ico的国家
    lock_proportion: Percent,  // 琐仓比例
    unlock_duration: BlockNumber, // 多久解锁一次
    per_duration_unlock_amount: Balance,  // 一次解锁多少

}


pub trait Config: system::Config + tokens::Config {

    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    type PermitIcoOrigin: EnsureOrigin<Self::Origin>;

    type RejectIcoOrigin: EnsureOrigin<Self::Origin>;

    type PermitReleaseOrigin: EnsureOrigin<Self::Origin>;

    type TerminateIcoOrigin: EnsureOrigin<Self::Origin>;

    type MinProportion: Get<Percent>;

    type OnSlash: OnUnbalanced<NegativeImbalanceOf<Self>>;

    type MultiCurrency: MultiCurrency<Self::AccountId> + MultiReservableCurrency<Self::AccountId>;

    type NativeCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;

    type InitiatorPledge: Get<BalanceOf<Self>>;  // 发起ico需要抵押的金额

    type RequestPledge: Get<BalanceOf<Self>>;  // 释放资金申请需要抵押的金额

    type RequestExpire: Get<Self::BlockNumber>;  // 释放资金申请过期的时间

    type NativeMultiple: Get<IcoMultiple>;  // 如果用DICO来ico 奖励倍数是多少

    type CurrenciesHandler: CurrenciesHandler<CurrencyIdOf<Self>, DicoAssetMetadata, DispatchError>;

    type IcoTotalReward: Get<MultiBalanceOf<Self>>;

}


decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        /// 发起ico需要抵押的金额
        const InitiatorPledge: BalanceOf<T> = T::InitiatorPledge::get();
        /// 释放资金申请需要抵押的金额
        const RequestPledge: BalanceOf<T> = T::RequestPledge::get();
        /// ico的总奖励
        const IcoTotalReward: MultiBalanceOf<T> = T::IcoTotalReward::get();
        /// 释放资金申请过期的时间
        const RequestExpire: T::BlockNumber = T::RequestExpire::get();
        /// 用DICO参与ico奖励的倍数
        const NativeMultiple: IcoMultiple = T::NativeMultiple::get();
        /// 项目方筹集资金的最低比例要求
        const MinProportion: Percent = T:: MinProportion::get();


        type Error = Error<T>;
		fn deposit_event() = default;


        /// 项目方发起ico
		#[weight = 10_000]
		fn initiate_ico(origin, info: IcoParameters<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries>) {

		    let initiator = ensure_signed(origin)?;

            /// 是否可以发起ico?  对参数或是存储进行基本的检查
            Self::is_can_initiate_ico(&info, &initiator)?;

            /// metadata是否已经存在 存在才能够参与ico
            let exchange_token_metadata = T::CurrenciesHandler::get_metadata(info.exchange_token)?;
            let metadata = T::CurrenciesHandler::get_metadata(info.currency_id)?;

            ensure!(info.currency_id.clone() != T::GetNativeCurrencyId::get(), Error::<T>::TokenIsDICO);

            let total_num = <TotalNum>::get().checked_add(1u32).ok_or(Error::<T>::Overflow)?;

            /// 需要抵押
            T::NativeCurrency::reserve(&initiator, T::InitiatorPledge::get())?;

            /// 锁住项目方代币
            T::MultiCurrency::reserve(info.currency_id.clone(), &initiator, info.total_ico_amount.clone())?;

            <PendingIco<T>>::mutate(|h| h.push(
                PendingInfo {
                    ico: IcoInfo {
                    start_time: None,
                    is_already_kyc: Self::is_already_kyc(&initiator),
                    initiator: initiator.clone(),
                    total_usdt: MultiBalanceOf::<T>::from(0u32),
                    is_terminated: false,
                    project_name: metadata.name.clone(),
                    token_symbol: metadata.symbol.clone(),
                    decimals: metadata.decimals.clone(),
                    index: None,
                    already_released_proportion: Percent::from_percent(0u8),
                    currency_id: info.currency_id.clone(),
                    logo_url: info.logo_url.clone(),
                    official_website: info.official_website.clone(),
                    is_must_kyc: info.is_must_kyc.clone(),
                    user_ico_max_count: info.user_ico_max_count.clone(),
                    total_issuance: info.total_issuance.clone(),
                    total_circulation: info.total_circulation.clone(),
                    ico_duration: info.ico_duration.clone(),
                    total_ico_amount: info.total_ico_amount.clone(),
                    user_min_amount: info.user_min_amount.clone(),
                    user_max_amount: info.user_max_amount.clone(),
                    exchange_token: info.exchange_token.clone(),
                    total_exchange_amount: info.total_exchange_amount.clone(),
                    exclude_nation: info.exclude_nation.clone(),
                    lock_proportion: info.lock_proportion.clone(),
                    unlock_duration: info.unlock_duration.clone(),
                    per_duration_unlock_amount: info.per_duration_unlock_amount.clone(),
                },
                pledge: T::InitiatorPledge::get(),

                }));

            Self::deposit_event(RawEvent::InitiateIco(initiator, info.exchange_token, info.total_ico_amount));

		}


		/// 议会成员同意发起ico
		#[weight = 10_000]
		fn permit_ico(origin, currency_id: CurrencyIdOf<T>) {
            ensure_root(origin)?;
		    // T::PermitIcoOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
		    let mut pending_ico = <PendingIco<T>>::get();

		    let pos_opt = pending_ico.iter().position(|h| currency_id == h.ico.currency_id);
		    match pos_opt {
		        None => return Err(Error::<T>::PendingIcoNotExists)?,
		        Some(pos) => {
		            let mut pending_info = pending_ico.swap_remove(pos);

		            let total_num = <TotalNum>::get().checked_add(1u32).ok_or(Error::<T>::Overflow)?;

		            /// 释放抵押的金额
		            T::NativeCurrency::unreserve(&pending_info.ico.initiator, pending_info.pledge.clone());

                    <PendingIco<T>>::put(pending_ico);
                    pending_info.ico.start_time = Some(Self::now());
                    pending_info.ico.index = Some(total_num);
                    <Ico<T>>::insert(pending_info.ico.currency_id.clone(), total_num, pending_info.ico.clone());
                    <TotalNum>::put(total_num);
                    Indexs::<T>::mutate(currency_id, |h| h.push(total_num));
                    RemainAmountIsAlreadyUnserve::<T>::insert(currency_id, total_num, false);
                     Self::deposit_event(RawEvent::PermitIco(pending_info.ico.initiator.clone(), pending_info.ico.currency_id.clone()));
		        },
		    }

		}


		/// 议会成员反对ico
		#[weight = 10_000]
		fn reject_ico(origin, currency_id: CurrencyIdOf<T>) {

		    // T::RejectIcoOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
            ensure_root(origin)?;

		    let mut pending_ico = <PendingIco<T>>::get();

		    let pos_opt = pending_ico.iter().position(|h| currency_id == h.ico.currency_id);
		    match pos_opt {
		        None => return Err(Error::<T>::PendingIcoNotExists)?,
		        Some(pos) => {
		            let pending_info = pending_ico.swap_remove(pos);

		            /// 惩罚抵押的金额
		            T::NativeCurrency::slash_reserved(&pending_info.ico.initiator, pending_info.pledge.clone());

		            /// 释放抵押的自己项目的代币
                    T::MultiCurrency::unreserve(pending_info.ico.currency_id.clone(), &pending_info.ico.initiator.clone(), pending_info.ico.total_ico_amount);

                    <PendingIco<T>>::put(pending_ico);

                     Self::deposit_event(RawEvent::RejectIco(pending_info.ico.initiator.clone(), pending_info.ico.currency_id.clone()));
		        },
		    }

		}


		/// 用户参加ico
        #[weight = 10_000]
        fn join(origin, currency_id: CurrencyIdOf<T>, index: u32, amount: MultiBalanceOf<T>) {
            let user = ensure_signed(origin)?;

            /// 金额不能为0
            ensure!(amount != 0u128.saturated_into::<MultiBalanceOf<T>>(), Error::<T>::AmountIsZero);

            let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;

            /// 获取主流币子的精度
            let exchange_token_decimals = T::CurrenciesHandler::get_metadata(ico.exchange_token)?.decimals;
            /// 获取代币等值的u
            let mut total_usdt = Self::exchange_token_convert_usdt(ico.exchange_token, exchange_token_decimals, ico.total_exchange_amount)?;
            let total_usdt_cp = total_usdt.clone();
            let unreleased_info_opt = Self::get_unrelease_assets(user.clone(), currency_id, index);
            match unreleased_info_opt.clone() {
                None => {},
                Some(x) => {total_usdt = total_usdt.checked_add(&unreleased_info_opt.unwrap().total_usdt.clone()).ok_or(Error::<T>::Overflow)?;}
            }
            /// 满足项目方的最低和最高要求
            ensure!(total_usdt >= ico.user_min_amount && total_usdt <= ico.user_max_amount, Error::<T>::AmountNotMeetProjectRequirement);
            /// 满足系统的最低和最高要求
            ensure!(total_usdt >= IcoMinUsdtAmount::<T>::get() && total_usdt <= IcoMaxUsdtAmount::<T>::get(), Error::<T>::AmountNotMeetSystemRequirement);

            /// 不能过期
            ensure!(ico.ico_duration + ico.start_time.unwrap() >= Self::now(), Error::<T>::IcoExpire);

            /// 不能是已经终止的
            ensure!(!ico.is_terminated, Error::<T>::Icoterminated);

            /// 项目方不能参与ico
            ensure!(&ico.initiator != &user, Error::<T>::InitiatorYourself);

            /// 参与ico的次数不能大于上限值
            if let Some(info) =  Self::get_unrelease_assets(user.clone(), currency_id, index) {
                ensure!(info.tags.len().saturating_add(1) <= ico.user_ico_max_count.clone().into(), Error::<T>::IcoCountToMax);
            }

            /// 资产要对得上
            ensure!(ico.currency_id == currency_id, Error::<T>::CurrencyIdErr);

            ensure!(!Self::is_in_exclude_nation(&user, &currency_id, index)?, Error::<T>::InExcludeNation);

            ico.total_usdt = ico.total_usdt.checked_add(&total_usdt_cp).ok_or(Error::<T>::Overflow)?;

            /// 用户的资产要满足一定的条件
            // 项目方的代币不需要检查的原因是：发起ico的时候已经琐仓
            Self::is_user_amount_satisfy(ico.clone(), amount, ico.currency_id, index, user.clone())?;

            /// 资产交换
            let user_exchange_amount = Self::swap(&user, amount, ico.clone())?;

            Ico::<T>::insert(currency_id, index, ico.clone());

            /// 更新资产具体信息
            Self::insert_ico_assets_info(user.clone(), ico, amount, total_usdt_cp);

            Self::deposit_event(RawEvent::Join(user, currency_id, index, amount, user_exchange_amount));
        }


        /// 议会终止ico
        #[weight = 10_000]
        fn terminate_ico(origin, currency_id: CurrencyIdOf<T>, index: u32) {
            T::TerminateIcoOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
            let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;

            ensure!(!ico.is_terminated, Error::<T>::Icoterminated);

            ico.is_terminated = true;

            <Ico<T>>::insert(currency_id, index, ico);

            Self::deposit_event(RawEvent::TerminateIco(currency_id, index));
        }


        /// 项目方要求释放资产
        #[weight = 10_000]
        fn request_release(origin, currency_id: CurrencyIdOf<T>, index: u32, percent: Percent) {

            let initiator = ensure_signed(origin)?;

            /// ico存在
            let ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;

            /// 申请释放的资金比例要比历史的大
            ensure!(ico.already_released_proportion.clone() < percent, Error::<T>::ProportionTooLow);
            /// 是项目方
            ensure!(&initiator == &ico.initiator, Error::<T>::NotInitiator);
            /// ico还没有被终止
            ensure!(!ico.is_terminated.clone(), Error::<T>::Icoterminated);
            /// 还没有申请过 这个index的资产
            ensure!(Self::get_request_release_info(currency_id, index).is_none(), Error::<T>::AlreadyRequest);

            T::NativeCurrency::reserve(&initiator, T::RequestPledge::get())?;

            <RequestReleaseInfo<T>>::mutate(|h|h.push(Release{
                who: initiator.clone(),
                currency_id: currency_id,
                index: index,
                request_time: Self::now(),
                percent: percent,
                pledge: T::RequestPledge::get(),
            }));

            Self::deposit_event(RawEvent::RequestRelease(currency_id, index, percent));
        }


        /// 项目方取消释放资金的申请(扣掉一半的抵押金额)
        #[weight = 10_000]
        fn cancel_request(origin, currency_id: CurrencyIdOf<T>, index: u32) {
            let initiator = ensure_signed(origin)?;
            /// ico存在
            let ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;
            /// 是项目方
            ensure!(&initiator == &ico.initiator, Error::<T>::NotInitiator);
            /// 已经申请过
            let release_info_opt = Self::get_request_release_info(currency_id, index);
            if release_info_opt.is_none() {
                return Err(Error::<T>::RequestNotExists)?;
            }
            else {
                let release_info = release_info_opt.unwrap();
                let slash = release_info.pledge / <BalanceOf<T>>::from(2u32);
                let unreserve = release_info.pledge.saturating_sub(slash);
                T::OnSlash::on_unbalanced(T::NativeCurrency::slash_reserved(&initiator, slash).0);
                T::NativeCurrency::unreserve(&initiator, unreserve);
                Self::remove_request_release_info(Some(currency_id), index, false);

            }

            Self::deposit_event(RawEvent::CancelRequest(currency_id, index));

        }


        /// 允许资产释放 (过半议员同意)
        #[weight = 10_000]
        fn permit_release(origin, currency_id: CurrencyIdOf<T>, index: u32) {
            T::PermitReleaseOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
            /// ico存在
            let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;
            let initiator = ico.initiator.clone();

            let release_info_opt = Self::get_request_release_info(currency_id, index);
            if release_info_opt.is_none() {
                return Err(Error::<T>::RequestNotExists)?;
            }
            else {
                let release_info = release_info_opt.clone().unwrap();

                ensure!(!ico.is_terminated, Error::<T>::Icoterminated);

                ensure!(release_info.percent > ico.already_released_proportion, Error::<T>::ProportionTooLow);

                ico.already_released_proportion = release_info.percent;

                Self::remove_request_release_info(Some(currency_id), index, false);

                /// 归还抵押金额
                T::NativeCurrency::unreserve(&initiator, release_info.pledge);

                Ico::<T>::insert(currency_id, index, ico);

            }

            Self::deposit_event(RawEvent::PermitRelease(currency_id, index, release_info_opt.unwrap()));

        }


        /// 用户(或是项目方)手动释放资产
        #[weight = 10_000]
        fn user_release_ico_amount(origin, currency_id: CurrencyIdOf<T>, index: u32) {
            let user = ensure_signed(origin)?;
            
            /// ico存在
            let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;
            let asset_info = Self::get_unrelease_assets(user.clone(), currency_id, index).ok_or(Error::<T>::NotIcoMember)?;

            let total = asset_info.total;
            let released = asset_info.released;

            ensure!(total.saturating_sub(released) > MultiBalanceOf::<T>::from(0u32), Error::<T>::UnreleasedAmountIsZero);

            // 本来应该已经释放的金额
            let should_released = ico.already_released_proportion * total;

            let mut is_terminated_released_amount = false;

            /// 如果项目终止 那么检查项目方得到的资金是否达到要求
            if ico.is_terminated && user.clone() != ico.initiator.clone() && asset_info.is_terminated == false {

                Self::terminated_unreleased_user_amount(user.clone(), total, should_released, ico.clone());
                is_terminated_released_amount = true;
            }

            /// 处理项目方未筹集到主流币的那部分保留代币
            if Self::is_ico_expire(&ico) && !RemainAmountIsAlreadyUnserve::<T>::get(currency_id, index){
                Self::unreserved_initiator_not_ico_amount(ico.clone());
            }

            // 本次需要释放的金额
            let thistime_release_amount = should_released.saturating_sub(asset_info.released);

            // 本次释放的金额要大于0
            if thistime_release_amount == MultiBalanceOf::<T>::from(0u32) && is_terminated_released_amount == false {
                return Err(Error::<T>::UnreleaseAmountIsZero)?;
            } else if thistime_release_amount == MultiBalanceOf::<T>::from(0u32) && is_terminated_released_amount == true {
                return Ok(());
            }

            Self::common_unreleased_user_amount(user, ico, thistime_release_amount);

        }


        /// 用户解锁自己的代币
        #[weight = 10_000]
        fn unlock(origin, currency_id: CurrencyIdOf<T>) {
            let user = ensure_signed(origin)?;
            ensure!(<IcoLocks<T>>::contains_key(&user, &currency_id), Error::<T>::LockIsEmpty);

            <IcoLocks<T>>::try_mutate(&user, &currency_id, |h| {
                let (total, locks) = Self::unlock_asset(&user, &currency_id, h);
                if total == <MultiBalanceOf<T>>::from(0u32) {
                    return Err(Error::<T>::UnlockAmountIsZero);
                }
                else {
                    *h = locks;

                    Self::deposit_event(RawEvent::UnlockAsset(currency_id, user.clone(), total));
                    Ok(())
                }

                })?;
        }


        /// 公投设置系统级别的ico区间金额
        #[weight = 10_000]
        fn set_system_ico_amount_bound(origin, min_amount: MultiBalanceOf<T>, max_amount: MultiBalanceOf<T>) {
            ensure_root(origin)?;
            ensure!(min_amount <= max_amount && max_amount != MultiBalanceOf::<T>::from(0u32), Error::<T>::MinOrMaxIcoAmountSetErr);
            IcoMinUsdtAmount::<T>::put(min_amount);
            IcoMaxUsdtAmount::<T>::put(max_amount);
            Self::deposit_event(RawEvent::SetSystemIcoAmountBound(min_amount, max_amount));

        }


        /// 项目方设置用户ico金额区间
        #[weight = 10_000]
        fn initiator_set_ico_amount_bound(origin, currency_id: CurrencyIdOf<T>, index: u32, min_amount: MultiBalanceOf<T>, max_amount: MultiBalanceOf<T>) {
            let user = ensure_signed(origin)?;
            /// ico存在
            let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;
            /// 是项目方
            ensure!(ico.initiator.clone() == user.clone(), Error::<T>::NotInitiator);
            /// 参加ico的最小金额不能小于系统要求的最小值， 并且最大值不能大于系统要求的最大值
            ensure!(min_amount >= IcoMinUsdtAmount::<T>::get() && min_amount <= max_amount  && max_amount <= IcoMaxUsdtAmount::<T>::get(), Error::<T>::MinOrMaxIcoAmountSetErr);
            ico.user_min_amount = min_amount;
            ico.user_max_amount = max_amount;
            Ico::<T>::insert(currency_id, index, ico);

            Self::deposit_event(RawEvent::InitiatorSetIcoAmountBound(currency_id, index, min_amount, max_amount));

        }


        /// 项目方设置单个用户ico最大次数
         #[weight = 10_000]
         fn initiator_set_ico_max_count(origin, currency_id: CurrencyIdOf<T>, index: u32, max_count: u8) {
            let user = ensure_signed(origin)?;
            /// ico存在
            let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;
            /// 是项目方
            ensure!(ico.initiator.clone() == user.clone(), Error::<T>::NotInitiator);
            /// 不能与上次的次数相同， 并且不能是0
            ensure!(ico.user_ico_max_count != max_count && max_count != 0u8, Error::<T>::MaxCountErr);
            ico.user_ico_max_count = max_count;
            Ico::<T>::insert(currency_id, index, ico);

            Self::deposit_event(RawEvent::SetIcoMaxCount(currency_id, index, max_count));
         }



        /// 手动领取奖励
        #[weight = 10_000]
        fn get_reward(origin, currency_id: CurrencyIdOf<T>, index: u32) {

            let user = ensure_signed(origin)?;

            /// ico存在
            let ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;

            /// 项目已经过期或是被终止
            ensure!(Self::is_ico_expire(&ico), Error::<T>::IcoNotExpireOrTerminated);

            /// 计算这个项目的总算力  这个值已经是dico作为单位
            let total_reward = Self::calculate_total_reward(ico.clone());
            runtime_print!("以dico作为单位时候的项目总算力: {:#?}", total_reward);

            /// 获取用户资产信息
            let asset_info_opt = Self::get_unrelease_assets(user.clone(), currency_id, index);
            /// 必须是参与ico
            ensure!(asset_info_opt.is_some(), Error::<T>::NotIcoMember);
            /// 获取参与记录
            let tags = asset_info_opt.clone().unwrap().tags;

            /// 分类用户资金(以参与的主流币金额去换算)
            let classify = Self::classify_user_amount(ico.total_usdt, tags);
            runtime_print!("用户分类资产的具体信息(主流币作为单位): {:#?}", classify);

            /// 计算用户奖励
            let reward = Self::caculate_user_reward(classify, ico.total_usdt, total_reward);

            /// 奖励要大于0
            ensure!(reward > MultiBalanceOf::<T>::from(0u32), Error::<T>::RewardIsZero);

            T::NativeCurrency::deposit_creating(&user, reward.saturated_into::<u128>().saturated_into::<BalanceOf<T>>());

            Self::deposit_event(RawEvent::GetReward(currency_id, index, user.clone(), reward));

        }


        fn on_finalize(n: T::BlockNumber) {
            // 删除过期的释放资金的申请
            Self::remove_request_release_info(None, 0u32, true);
        }


    }
}


impl <T: Config> Module <T> {


    /// 获取现在的区块
    fn now() -> T::BlockNumber {
        <system::Pallet<T>>::block_number()
    }


    /// 获取用户的国籍  todo
    fn get_uesr_nation(who: &T::AccountId) -> Option<Countries> {
        Some(Countries::China)
    }


    /// 用户是否已经kyc todo
    fn is_already_kyc(who: &T::AccountId) -> bool {
        false
    }


    /// 是否是正在等待的ico
    fn is_pending_ico(currency_id: &CurrencyIdOf<T>) -> bool {
        let pending_ico = <PendingIco<T>>::get();
        let pos_opt = pending_ico.iter().position(|h| currency_id == &h.ico.currency_id);
        match pos_opt {
            None => { return false; },
            Some(x) => { return true; },
        }
    }


    /// 是否可以发起ico
    fn is_can_initiate_ico(info: &IcoParameters<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries>, who: &T::AccountId) -> result::Result<bool, DispatchError> {
        /// 用户参与ico的最大次数要大于0
        ensure!(info.user_ico_max_count > 0, Error::<T>::MaxCountIsZero);
        /// 不能是DICO项目方代币发起
        ensure!(info.currency_id != T::GetNativeCurrencyId::get(), Error::<T>::NativeCurrencyId);
        /// 代币id不能相同
        ensure!(info.exchange_token != info.currency_id, Error::<T>::TokenShouldBeDifferent);
        /// 用来兑换的币数量不能为0
        ensure!(info.total_exchange_amount != 0u128.saturated_into::<MultiBalanceOf<T>>() && info.total_ico_amount != 0u128.saturated_into::<MultiBalanceOf<T>>(), Error::<T>::AmountIsZero);
        /// 代币发行数量大于流通量 流通量大于用来ico的数量
        ensure!(info.total_issuance >= info.total_circulation && info.total_circulation >= info.total_ico_amount, Error::<T>::AmountErr);

        /// 参加ico的最小金额不能小于系统要求的最小值， 并且最大值不能大于系统要求的最大值
        ensure!(info.user_min_amount >= IcoMinUsdtAmount::<T>::get() && info.user_min_amount <= info.user_max_amount  && info.user_max_amount <= IcoMaxUsdtAmount::<T>::get(), Error::<T>::MinOrMaxIcoAmountSetErr);

        /// ico周期不能是0
        ensure!(info.ico_duration > T::BlockNumber::from(0u32), Error::<T>::DurationIsZero);
        /// 琐仓比例如果不是0 那么琐仓周期跟金额不能是0
        if info.lock_proportion > Percent::from_percent(0u8) {
            ensure!(info.unlock_duration > T::BlockNumber::from(0u32) && info.per_duration_unlock_amount > 0u128.saturated_into::<MultiBalanceOf<T>>(), Error::<T>::LockParaErr);
        }

        /// 没有发起过ico
        ensure!(!Self::is_pending_ico(&info.currency_id), Error::<T>::IsPendingIco);

        ensure!(T::MultiCurrency::can_reserve(info.currency_id.clone(), &who, info.total_ico_amount.clone()), Error::<T>::AmountTooLow);

        Ok(true)
    }


    /// 是否是项目成员(不包括项目方)
    fn is_member(who: &T::AccountId, currency_id: CurrencyIdOf<T>, index: u32) -> bool {
        let unrelease_info_vec = UnReleaseAssets::<T>::get(&who);
        if let Some(pos) = unrelease_info_vec.iter().position(|h| h.currency_id == currency_id && h.index == index && h.unreleased_currency_id == currency_id) {
            return true;
        }
        false
    }


    /// 获取释放的资产的信息
    fn get_unrelease_assets(who: T::AccountId, currency_id: CurrencyIdOf<T>, index: u32) -> Option<UnRelease<MultiBalanceOf<T>, CurrencyIdOf<T>>> {
        let mut unrelease_info_vec = UnReleaseAssets::<T>::get(&who);
        if let Some(pos) = unrelease_info_vec.iter().position(|h| h.currency_id == currency_id && h.index == index ) {
            return Some(unrelease_info_vec.swap_remove(pos));
        }
        None
    }


    /// 获取参与项目的金额
    fn get_user_ico_amount(currency_id: CurrencyIdOf<T>, index: u32, who: &T::AccountId) -> (MultiBalanceOf<T>, MultiBalanceOf<T>) {
        let mut unrelease_info_vec = UnReleaseAssets::<T>::get(&who);
        if let Some(pos) = unrelease_info_vec.iter().position(|h| h.currency_id == currency_id && h.index == index ) {
            let info = unrelease_info_vec.swap_remove(pos);
            return (info.total, info.released);
        }
        (MultiBalanceOf::<T>::from(0u32), MultiBalanceOf::<T>::from(0u32))
    }


    /// 是否是被排除在外的国家
    fn is_in_exclude_nation(who: &T::AccountId, currency_id: &CurrencyIdOf<T>, index: u32) -> result::Result<bool, DispatchError> {
        let ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;
        if &ico.is_must_kyc == &true {
            if Self::is_already_kyc(&who) {
                let nation_opt = Self::get_uesr_nation(&who);
                match nation_opt {
                    // 如果没有国籍 那么不允许参加
                    None => Ok(true),
                    Some(nation) => {
                        let nations = ico.exclude_nation.clone();
                        // 如果有国籍 并且国籍是被排除在外的国家 不允许参加
                        if let Some(pos) = nations.iter().position(|h| *h == nation) {
                            Ok(true)
                        }
                        else {
                            Ok(false)
                        }
                    },

                }
            }
                // 如果没有kyc 那么直接不允许参加
            else {
                Ok(true)
            }
        }
            // 如果项目不需要key 那么直接给参加
        else {
            Ok(false)
        }

    }


    /// 用户金额(主流币)是否足够参加ico
    fn is_user_amount_satisfy(ico: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>, amount: MultiBalanceOf<T>, currency_id: CurrencyIdOf<T>, index: u32, who: T::AccountId) -> result::Result<bool, DispatchError> {

        /// 金额要大于最小金额小于最大金额
        let user_total_amount = Self::get_user_total_amount(currency_id, index, &who);

        /// 项目方代币换主流
        let user_total_amount = Self::get_swap_token_amount(false, user_total_amount, ico.clone());

        ensure!((user_total_amount.saturating_add(amount) >= ico.user_min_amount) && (user_total_amount.saturating_add(amount) <= ico.user_max_amount), Error::<T>::AmountTooLowOrTooBig);

        let initiator_total_amount = Self::get_user_ico_amount(ico.exchange_token.clone(), index, &ico.initiator).0;

        /// 金额要小于等与项目方剩余的需要筹集的资金
        ensure!(ico.total_exchange_amount.saturating_sub(initiator_total_amount) >= amount, Error::<T>::AmountTooLow);

        Ok(true)
    }


    /// 项目方跟用户进行代币互换
    fn swap(who: &T::AccountId, amount: MultiBalanceOf<T>, ico: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>) -> result::Result<MultiBalanceOf<T>, DispatchError> {
        let project_token_id = ico.currency_id;
        let exchange_token_id = ico.exchange_token;
        let initiator = ico.initiator.clone();

        // 用户得到的代币
        let this_time_project_token_amount = Self::get_swap_token_amount(true, amount, ico.clone());

        // 转账给项目方之前做一个基本检查（是否能够进行下一步的抵押)
        ensure!(Self::is_can_reserve(ico.exchange_token.clone(), &initiator)?, Error::<T>::CanNotReserve);
        // 检查项目方剩余的项目代币是否足够
        ensure!(T::MultiCurrency::reserved_balance(project_token_id, &initiator) >= this_time_project_token_amount, Error::<T>::ReservedBalanceNotEnough);
        // 用户的代币直接转移给项目方
        Self::transfer(exchange_token_id, &who, &initiator, amount, ExistenceRequirement::KeepAlive)?;
        // 对项目方进行琐仓操作
        Self::reserve(ico.exchange_token.clone(), &initiator, amount)?;

        // 项目方的代币转到用户保留余额账户 todo 这个转账需要检查
        T::MultiCurrency::repatriate_reserved(project_token_id, &initiator, who, this_time_project_token_amount, BalanceStatus::Reserved)?;

        Ok(this_time_project_token_amount)

    }


    /// 获取代币价格 todo
    fn get_token_price(currency_id: CurrencyIdOf<T>) -> MultiBalanceOf<T>{
        (1u128 * Usdt).saturated_into::<MultiBalanceOf<T>>()

    }


    /// 主流币换算成usdt
    fn exchange_token_convert_usdt(currency_id: CurrencyIdOf<T>, decimals: u8, amount: MultiBalanceOf<T>) -> result::Result<MultiBalanceOf<T>, DispatchError>{

        /// 获取代币的价格 这个是18精度
        let price = Self::get_token_price(currency_id);

        let decimals_amount = 10u128.saturating_pow(decimals as u32).saturated_into::<MultiBalanceOf<T>>();

        let mut usdt = Self::u256_convert_to_balance(Self::balance_convert_to_u256(price) * Self::balance_convert_to_u256(amount) / Self::balance_convert_to_u256(decimals_amount));

        if currency_id == T::GetNativeCurrencyId::get() {

            usdt = (usdt * T::NativeMultiple::get().numerator.saturated_into::<MultiBalanceOf<T>>()).checked_div(&T::NativeMultiple::get().denominator.saturated_into::<MultiBalanceOf<T>>()).ok_or(Error::<T>::DenominatorIsZero)?;
        }

        Ok(usdt)
    }


    /// 金额类型转成u256
    fn balance_convert_to_u256(amount: MultiBalanceOf<T>) -> U256 {
        amount.saturated_into::<u128>().into()
    }


    /// u256转换成金额
    fn u256_convert_to_balance(num: U256) -> MultiBalanceOf<T> {
        <u128 as TryFrom<U256>>::try_from(num).unwrap()
            .saturated_into::<MultiBalanceOf<T>>()
    }

    //
    // /// usdt转换成算力  也就是usdt转换成dico
    // fn usdt_convert_power(amount: MultiBalanceOf<T>) -> MultiBalanceOf<T> {
    //     // usdt是以太坊精度 18
    //     // dico的精度 14
    //     amount / 10u32.pow(4).saturated_into::<MultiBalanceOf<T>>()
    // }


    /// 主流币兑换项目方代币 (返回项目方代币数量)
    fn get_swap_token_amount(is_main: bool, amount: MultiBalanceOf<T>, ico: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>) -> MultiBalanceOf<T> {
        let total_project_token_amount = ico.total_ico_amount;
        let total_exchange_token_amount = ico.total_exchange_amount;

        let mut result: U256;

        if is_main {

            result = Self:: balance_convert_to_u256(amount)  * Self:: balance_convert_to_u256(total_project_token_amount) / Self:: balance_convert_to_u256(total_exchange_token_amount)
        }
            
        else {
             result = Self:: balance_convert_to_u256(amount) * Self:: balance_convert_to_u256(total_exchange_token_amount) / Self:: balance_convert_to_u256(total_project_token_amount)
            
        }

        Self::u256_convert_to_balance(result)
    }


    /// 项目ico是否已经过期
    fn is_ico_expire(ico: &IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>) -> bool {
        if ico.is_terminated || (ico.start_time.unwrap() + ico.ico_duration < Self::now()) {
            return true;
        }
        false
    }


    /// 是否可以保留（现在的自由余额是否可以进行保留)
    fn is_can_reserve(currency_id: CurrencyIdOf<T>, who: &T::AccountId) -> result::Result<bool, DispatchError> {
        if currency_id == T::GetNativeCurrencyId::get() {
            let free_amount = T::NativeCurrency::free_balance(&who);

            T::NativeCurrency::ensure_can_withdraw(&who, BalanceOf::<T>::from(2u32), WithdrawReasons::RESERVE, free_amount)?;
        }
        else {
            let free_amount = T::MultiCurrency::free_balance(currency_id, &who);
            T::MultiCurrency::ensure_can_withdraw(currency_id, &who, free_amount)?;

        }

        Ok(true)
    }


    /// 是否已经申请资金释放
    fn get_request_release_info(currency_id: CurrencyIdOf<T>, index: u32) -> Option<Release<T::AccountId, T::BlockNumber, CurrencyIdOf<T>, BalanceOf<T>>> {
        let mut release_info = <RequestReleaseInfo<T>>::get();
        if let Some(pos) = release_info.iter().position(|h| h.currency_id == currency_id  && h.index == index) {
            return Some(release_info.swap_remove(pos));
        }
        None
    }


    /// 删除资金申请信息
    fn remove_request_release_info(currency_id: Option<CurrencyIdOf<T>>, index: u32, is_all: bool) {
        let mut release_info = <RequestReleaseInfo<T>>::get();
        if is_all {
            release_info.retain(|h| if h.request_time + T::RequestExpire::get() > Self::now() {true} else {
                T::OnSlash::on_unbalanced(T::NativeCurrency::slash_reserved(&h.who, h.pledge).0);
                false
            });

        }
        else {
            if currency_id.is_some() {
                if let Some(pos) = release_info.iter().position(|h| h.currency_id == currency_id.unwrap() && h.index == index) {
                    release_info.swap_remove(pos);
                }
            }

        }

        <RequestReleaseInfo<T>>::put(release_info);

    }


    /// 转账
    fn transfer(currency_id: CurrencyIdOf<T>, who: &T::AccountId, dest: &T::AccountId, amount: MultiBalanceOf<T>, requirement: ExistenceRequirement) -> DispatchResult{
        if currency_id == T::GetNativeCurrencyId::get() {
            // 用户的主流币转到项目方的账户
            T::NativeCurrency::transfer(who, dest, amount.saturated_into::<u128>().saturated_into::<BalanceOf<T>>(), ExistenceRequirement::KeepAlive)?;

        }
        else {
            T::MultiCurrency::transfer(currency_id, who, dest, amount)?;
        }
        Ok(())
    }


    /// 解锁资产
    fn unlock_asset(who: &T::AccountId, currency_id: &CurrencyIdOf<T>, locks: &mut [IcoLock<MultiBalanceOf<T>, T::BlockNumber>]) -> (MultiBalanceOf<T>, Vec<IcoLock<MultiBalanceOf<T>, T::BlockNumber>>) {

        let mut total = <MultiBalanceOf<T>>::from(0u32);
        for i in (0..locks.len()) {
            let time = Self::now().saturating_sub(locks[i].start_block);
            if locks[i].unlock_duration == T::BlockNumber::from(0u32) {
                // 全部释放
                let unlock_amount = locks[i].total_amount.saturating_sub(locks[i].unlock_amount);

                // Self::lock_sub_amount(ICO_ID, *currency_id, who, unlock_amount, WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
                Self::unreserve(*currency_id, who.clone(), unlock_amount);

                total += unlock_amount;
            }
            else {
                let num = time / locks[i].unlock_duration;
                let total_unlock_amount = (num.saturated_into::<u32>().saturated_into::<MultiBalanceOf<T>>().saturating_mul(locks[i].per_duration_unlock_amount)).min(locks[i].total_amount);
                let this_time_unlock_amount = total_unlock_amount.saturating_sub(locks[i].unlock_amount);
                // 改变未解锁的金额
                locks[i].unlock_amount = total_unlock_amount;
                // Self::lock_sub_amount(ICO_ID, *currency_id, who, this_time_unlock_amount, WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
                Self::unreserve(*currency_id, who.clone(), this_time_unlock_amount);
                total += this_time_unlock_amount;
            }
        }
        let mut locks = locks.to_vec();
        locks.retain(|h| h.total_amount != h.unlock_amount);

        (total, locks)

    }


    /// 计算总算力
    fn calculate_total_reward(ico: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>) -> MultiBalanceOf<T> {

        let actual_total_usdt = ico.total_usdt;
        runtime_print!("项目方实际筹集到的usdt数目是: {:#?}", actual_total_usdt);

        let total_usdt = TotalUsdt::<T>::get().saturating_add(actual_total_usdt);

        TotalUsdt::<T>::put(total_usdt);

        let num = total_usdt / ChangeReward.saturated_into::<MultiBalanceOf<T>>();
        runtime_print!("第{:#?}个奖励周期", num);

        /// 以usdt计量的算力
        let mut power_as_usdt = MultiBalanceOf::<T>::from(0u32);

        if total_usdt.saturating_sub(actual_total_usdt) < num * ChangeReward.saturated_into::<MultiBalanceOf<T>>() {
            let power1 = total_usdt.saturating_sub(num * ChangeReward.saturated_into::<MultiBalanceOf<T>>());
            let power2 = actual_total_usdt.saturating_sub(power1);
            power_as_usdt = power1 / 2u32.pow(num.saturated_into::<u32>()).saturated_into::<MultiBalanceOf<T>>() + power2 / 2u32.pow(num.saturated_into::<u32>() - 1u32).saturated_into::<MultiBalanceOf<T>>();

        }
        else {
            power_as_usdt = actual_total_usdt / 2u32.pow(num.saturated_into::<u32>()).saturated_into::<MultiBalanceOf<T>>();
        }
        runtime_print!("以usdt作为算力单位, 获得算力是: {:#?}", power_as_usdt);

        /// 第一个2亿usdt对应多少奖励
        let first_total = T::IcoTotalReward::get() / 2u32.saturated_into::<MultiBalanceOf<T>>();
        runtime_print!("第一年获取的奖励是: {:#?}", first_total);

        Self::u256_convert_to_balance(Self::balance_convert_to_u256(power_as_usdt) * Self::balance_convert_to_u256(first_total) / Self::balance_convert_to_u256(ChangeReward.saturated_into::<MultiBalanceOf<T>>()))

    }


    /// 插入UnReleaseAssets存储
    fn insert_ico_assets_info(who: T::AccountId, ico: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>, amount: MultiBalanceOf<T>, total_usdt: MultiBalanceOf<T>)  {
        // 更新用户的存储
        Self::update_user_unreleased_assets_info(who.clone(), ico.clone(), amount, total_usdt, true, None);
        // 更新项目方的存储
        Self::update_user_unreleased_assets_info(ico.initiator.clone(), ico.clone(), amount, total_usdt,true, None);

    }


    /// 更新具体用户的assets_info
    fn update_user_unreleased_assets_info(who: T::AccountId, ico: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>, amount: MultiBalanceOf<T>,
                                          total_usdt: MultiBalanceOf<T>,
                                          is_start_join: bool, is_terminated: Option<bool>) {

        let is_initiator = who == ico.initiator.clone();
        let mut unreleased_currency_id = ico.currency_id.clone();
        let mut total = amount;
        let mut user = who.clone();
        if is_initiator {
            user = ico.initiator.clone();
            unreleased_currency_id = ico.exchange_token.clone();
        }
        else {
            // 用户手里的币子是山寨
            total = Self::get_swap_token_amount(true, amount, ico.clone());
        }

        let mut assets_info = <UnReleaseAssets<T>>::get(user.clone());

        let mut new_info: UnRelease<MultiBalanceOf<T>, CurrencyIdOf<T>>;

        /// 如果本来就存在 那么就更新
        if let Some(pos) = assets_info.iter().position(|h| h.currency_id == ico.currency_id && h.index == ico.index.clone().unwrap()) {
            new_info = assets_info.swap_remove(pos);

            // 如果是更新总金额
            if is_start_join {
                if who != ico.initiator {

                    new_info.tags.push((total_usdt, new_info.total_usdt.saturating_add(total_usdt)));

                }
                    // 项目方不能参与 所以不能记录
                else {
                    new_info.tags = vec![];
                }
                new_info.total_usdt = new_info.total_usdt.saturating_add(total_usdt);
                new_info.total = new_info.total.saturating_add(total);
            }
                // 如果是更新已经释放的金额
            else {
                new_info.released = new_info.released.saturating_add(total);

            }

            if is_terminated.is_some() && is_terminated.unwrap() == true {
                new_info.is_terminated = is_terminated.unwrap();
            }

        }

        else {
            if who != ico.initiator {
                new_info =  UnRelease {
                    currency_id: ico.currency_id.clone(),
                    index: ico.index.clone().unwrap(),
                    tags: vec![(total_usdt, total_usdt)],
                    total_usdt: total_usdt,
                    unreleased_currency_id: unreleased_currency_id,
                    total: total,
                    is_terminated: false,
                    released: MultiBalanceOf::<T>::from(0u32),
                    is_already_get_reward: Some(false),
                };

            }
            else {
                new_info =  UnRelease {
                    currency_id: ico.currency_id.clone(),
                    index: ico.index.clone().unwrap(),
                    tags: vec![],
                    total_usdt: total_usdt,
                    unreleased_currency_id: unreleased_currency_id,
                    total: total,
                    is_terminated: false,
                    released: MultiBalanceOf::<T>::from(0u32),
                    is_already_get_reward: Some(false),
            };
            }

        };

        // 如果释放金额等与总金额 并且已经领取奖励 那就删除
        if (new_info.released == new_info.total) && (new_info.is_already_get_reward.is_some() && new_info.is_already_get_reward.unwrap() == true) {

        }
        else {
            assets_info.push(new_info);
        }

        UnReleaseAssets::<T>::insert(user, assets_info);

    }


    /// 普通释放
    fn common_unreleased_user_amount(user: T::AccountId, ico: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>, thistime_release_amount: MultiBalanceOf<T>) {
        // 获取主流币
        let project_currency_id = ico.exchange_token.clone();

        // 如果是项目方 给项目方解锁主流币
        if user.clone() == ico.initiator.clone() {
            Self::unreserve(project_currency_id, ico.initiator.clone(), thistime_release_amount);

        }
        // 如果是用户 则进行比较特殊的处理
        else {
            let user_keep_lock_amount = ico.lock_proportion * thistime_release_amount;
            // 本次解锁的金额
            let user_unlock_amount = thistime_release_amount.saturating_sub(user_keep_lock_amount);
            if user_unlock_amount > MultiBalanceOf::<T>::from(0u32) {
                // 解锁用户部分金额
                Self::unreserve(ico.currency_id, user.clone(), user_unlock_amount);

            }

            // 需要琐仓的部分需要琐仓
            if user_keep_lock_amount > MultiBalanceOf::<T>::from(0u32) {
                 <IcoLocks<T>>::mutate(user.clone(), ico.currency_id.clone(), |h| h.push(
                    IcoLock {
                        start_block: Self::now(),
                        index: ico.index.clone().unwrap(),
                        total_amount: user_keep_lock_amount,
                        unlock_amount: 0u32.saturated_into::<MultiBalanceOf<T>>(),
                        unlock_duration: ico.unlock_duration,
                        per_duration_unlock_amount: ico.per_duration_unlock_amount,
                    }
                ));
            }

        }

        // 更新用户的未释放资产
        Self::update_user_unreleased_assets_info(user.clone(), ico.clone(), thistime_release_amount, MultiBalanceOf::<T>::from(0u32), false, None);

        Self::deposit_event(RawEvent::UserReleaseIcoAmount(ico.currency_id, ico.index.unwrap(), thistime_release_amount));
    }


    /// 释放用户未释放的资产
    fn terminated_unreleased_user_amount(user: T::AccountId, total: MultiBalanceOf<T>, should_released: MultiBalanceOf<T>, ico: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>) {
        // 还剩下的未释放的金额
        let should_remain_amount = total.saturating_sub(should_released);
        // 这部分金额兑换成主流币
        let exchange_token_amount = Self::get_swap_token_amount(false, should_remain_amount, ico.clone());
        // 项目方把保留的主流币归还到用户的自由账户
        T::MultiCurrency::repatriate_reserved(ico.exchange_token.clone(), &ico.initiator, &user, exchange_token_amount, BalanceStatus::Free);

        let (project_total_amount, project_released_amount) = Self::get_user_ico_amount(ico.currency_id, ico.index.unwrap().clone(), &ico.initiator);
         // 如果项目方没有筹集到足够资金 那么销毁掉用户手里的项目方代币
         if T::MinProportion::get() * ico.total_exchange_amount > project_total_amount {
            T::MultiCurrency::slash_reserved(ico.currency_id, &user, should_remain_amount);
         }
         else {
            T::MultiCurrency::repatriate_reserved(ico.currency_id, &user, &ico.initiator, should_remain_amount, BalanceStatus::Free);
         }

        // 更新用户的未释放资产
        Self::update_user_unreleased_assets_info(user.clone(), ico.clone(), should_remain_amount, MultiBalanceOf::<T>::from(0u32),false, Some(true));
        Self::deposit_event(RawEvent::TerminatedGiveBackAmount(user.clone(), ico.currency_id, ico.index.unwrap(), should_remain_amount));

    }


    /// 处理项目方未筹集到主流币的那部分保留代币
    fn unreserved_initiator_not_ico_amount(ico: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>) {
        // 处理项目方还在保留的那部分资产
        let (project_total_amount, _) = Self::get_user_ico_amount(ico.currency_id, ico.index.clone().unwrap(), &ico.initiator);
        // 项目方手里未筹集到的资金
        let unico_amount = ico.total_exchange_amount.saturating_sub(project_total_amount);
        // 资金兑换成项目方币
        let should_slash_project_token = Self::get_swap_token_amount(true, unico_amount, ico.clone());
        // 如果项目方没有筹集到足够资金
        if T::MinProportion::get() * ico.total_exchange_amount > project_total_amount {

            // 惩罚掉项目方这部分的代币
            T::MultiCurrency::slash_reserved(ico.currency_id, &ico.initiator, should_slash_project_token);
        }
        else {
            // 释放项目方手里的代币
            T::MultiCurrency::unreserve(ico.currency_id, &ico.initiator, should_slash_project_token);
        }
        RemainAmountIsAlreadyUnserve::<T>::insert(ico.currency_id, ico.index.unwrap().clone(), true);
        Self::deposit_event(RawEvent::UnreservedInitiatorRemainPledgeAmount(ico.currency_id, ico.index.unwrap().clone(), should_slash_project_token));

    }


    /// 分类用户资金
    fn classify_user_amount(total_amount: MultiBalanceOf<T>, info: Vec<(MultiBalanceOf<T>, MultiBalanceOf<T>)>) -> Vec<(u32, MultiBalanceOf<T>)> {
        const NUM: u32 = 10;
        let av_amount = total_amount / NUM.saturated_into::<MultiBalanceOf<T>>();

        let mut result: Vec<(u32, MultiBalanceOf<T>)> = vec![];

        for (amount, tag_amount) in info.iter() {
            // 上一轮开始的金额
            let mut start_amount = tag_amount.saturating_sub(*amount);
            // 第一个奖励方案顺序
            let mut n = (start_amount / av_amount).saturated_into::<u32>() + 1;
            loop {
                // 下一个阶段开始的金额
                let next_amount = n.saturated_into::<MultiBalanceOf<T>>() * av_amount;
                if next_amount > *tag_amount {
                    if tag_amount.saturating_sub(start_amount) > MultiBalanceOf::<T>::from(0u32) {
                        result.push((n, tag_amount.saturating_sub(start_amount)));
                    }

                    break;
                }

                result.push((n, next_amount.saturating_sub(start_amount)));
                start_amount = next_amount;
                n += 1;

            }

        }

        result
    }


    /// 奖励用户
    fn caculate_user_reward(info: Vec<(u32, MultiBalanceOf<T>)>, total_amount: MultiBalanceOf<T>, total_reward: MultiBalanceOf<T>) -> MultiBalanceOf<T>{
        let mut total_reward = MultiBalanceOf::<T>::from(0u32);
        let tmpt = 50u32;
        for (n, amount) in info.iter() {
           if *n <= 5u32 {
               total_reward += (*amount + (Percent::from_percent((50 - (n - 1) * 10) as u8) * *amount))
           }
            else if *n > 5 && *n <= 10 {
                total_reward += (*amount - (Percent::from_percent(((n - 5) * 10) as u8) * *amount));
            }

        }
        // total_power 这个时候的单位已经是dico
        // total_amount 与 total_reward 是主流币作为单位
        Self::u256_convert_to_balance(Self::balance_convert_to_u256( total_reward) * Self::balance_convert_to_u256(total_reward) / Self::balance_convert_to_u256(total_amount))

    }


    /// 解锁保留
    fn unreserve(currency_id: CurrencyIdOf<T>, who: T::AccountId, amount: MultiBalanceOf<T>) {
        if currency_id == T::GetNativeCurrencyId::get() {
            T::NativeCurrency::unreserve(&who, amount.saturated_into::<u128>().saturated_into::<BalanceOf<T>>());
        }
        else {
            T::MultiCurrency::unreserve(currency_id, &who, amount);
        }
    }


    fn reserve(currency_id: CurrencyIdOf<T>, who: &T::AccountId, amount: MultiBalanceOf<T>) -> DispatchResult {
        if currency_id == T::GetNativeCurrencyId::get() {
            T::NativeCurrency::reserve(&who, amount.saturated_into::<u128>().saturated_into::<BalanceOf<T>>())?;
        }
        else {
            T::MultiCurrency::reserve(currency_id, &who, amount)?;
        }
        Ok(())
    }


}

decl_storage! {
    trait Store for Module<T: Config> as IcoModule {

        /// 正在等待ico
        pub PendingIco get(fn pending_ico): Vec<PendingInfo<IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>, BalanceOf<T>>>;//Vec<(IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>, BalanceOf<T>)>;

        /// 正在进行的ico
        pub Ico get(fn ico): double_map hasher(identity)  CurrencyIdOf<T>, hasher(identity) u32 => Option<IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>>;

        /// 参与ico还未释放的资产信息
        pub UnReleaseAssets get(fn ico_assets_info): map hasher(identity)  T::AccountId => Vec<UnRelease<MultiBalanceOf<T>, CurrencyIdOf<T>>>;

        /// 项目方申请释放资金的具体信息
        pub RequestReleaseInfo get(fn release_info): Vec<Release<T::AccountId, T::BlockNumber, CurrencyIdOf<T>, BalanceOf<T>>>;

        /// 已经释放的还未解锁的资产
        pub IcoLocks get(fn locks): double_map hasher(identity) T::AccountId, hasher(identity) CurrencyIdOf<T> => Vec<IcoLock<MultiBalanceOf<T>, T::BlockNumber>>;

        /// 参与ico的资产折算成usdt的历史总金额
        pub TotalUsdt get(fn total_usdt): MultiBalanceOf<T>;

        /// ico total num
        pub TotalNum get(fn total_num): u32;

        /// 资产参与ico的所有index
        pub Indexs get(fn indexes): map hasher(identity) CurrencyIdOf<T> => Vec<u32>;

        /// 项目方到期未筹集到资金 保留的代币剩下部分是否已经释放
        pub RemainAmountIsAlreadyUnserve: double_map hasher(identity) CurrencyIdOf<T>, hasher(identity) u32 => bool;

        /// 参与ico的最小usdt金额
        pub IcoMinUsdtAmount: MultiBalanceOf<T> = MultiBalanceOf::<T>::from(0u32);

        /// 参与ico的最大的usdt金额
        pub IcoMaxUsdtAmount: MultiBalanceOf<T> = (10_0000u128 * Usdt).saturated_into::<MultiBalanceOf<T>>();



    }
}

impl<T: Config> IcoHandler<CurrencyIdOf<T>, MultiBalanceOf<T>, T::AccountId, DispatchError, T::BlockNumber> for Module<T> {

    // 以下都是以项目方的代币作为单位

    // 是否存在项目中
    fn is_project_ico_member(currency_id: CurrencyIdOf<T>, index: u32, who: &T::AccountId) -> result::Result<bool, DispatchError> {
        let _ = Ico::<T>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;
        Ok(Self::is_member(who, currency_id, index))
    }

    // 获取参与的金额
    fn get_user_total_amount(currency_id: CurrencyIdOf<T>, index: u32, who: &T::AccountId) -> MultiBalanceOf<T> {

        Self::get_user_ico_amount(currency_id, index, &who).0
    }

    // 获取所有成员参与ico的金额
    fn get_total_ico_amount(currency_id: CurrencyIdOf<T>, index: u32) -> Result<MultiBalanceOf<T>, DispatchError> {
        let ico = Ico::<T>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists)?;
        let result = Self::balance_convert_to_u256(ico.total_ico_amount) * Self::balance_convert_to_u256(Self::get_user_ico_amount(currency_id, index, &ico.initiator).0) / Self::balance_convert_to_u256(ico.total_exchange_amount);
        let amount = Self::u256_convert_to_balance(result);
        Ok(amount)

    }

}
decl_event!(
    pub enum Event<T> where
    <T as system::Config>::AccountId,
    CurrencyId = CurrencyIdOf<T>,
    Amount = MultiBalanceOf<T>,
    Release = Release<<T as system::Config>::AccountId, <T as system::Config>::BlockNumber, CurrencyIdOf<T>, BalanceOf<T>>,
    {
        Test(AccountId),
        InitiateIco(AccountId, CurrencyId, Amount),
        PermitIco(AccountId, CurrencyId),
        RejectIco(AccountId, CurrencyId),
        Join(AccountId, CurrencyId, u32, Amount, Amount),
        TerminateIco(CurrencyId, u32),
        RequestRelease(CurrencyId, u32, Percent),
        CancelRequest(CurrencyId, u32),
        PermitRelease(CurrencyId, u32, Release),
        UnlockAsset(CurrencyId, AccountId, Amount),
        GetReward(CurrencyId, u32, AccountId, Amount),
        UserReleaseIcoAmount(CurrencyId, u32, Amount),
        InitiatorSetIcoAmountBound(CurrencyId, u32, Amount, Amount),
        SetSystemIcoAmountBound(Amount, Amount),
        SetIcoMaxCount(CurrencyId, u32, u8),
        TerminatedGiveBackAmount(AccountId, CurrencyId, u32, Amount),
        UnreservedInitiatorRemainPledgeAmount(CurrencyId, u32, Amount),

    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        Test,
        BeingIco,
        AmountTooLow,
        PendingIcoNotExists,
        IcoNotExists,
        InExcludeNation,
        BadOrigin,
        IsPendingIco,
        AmountTooLowOrTooBig,
        JoinIcoDuplicate,
        InitiatorYourself,
        CurrencyIdErr,
        AmountIsZero,
        TokenShouldBeDifferent,
        AmountErr,
        MaxAmountTooLow,
        AmountTooBig,
        MinAmountTooBig,
        RewardAmountIsZero,
        DurationIsZero,
        LockParaErr,
        IcoExpire,
        IcoNotExpireOrTerminated,
        Icoterminated,
        NotInitiator,
        AlreadyRequest,
        RequestNotExists,
        RequestExpire,
        AssetsInfoNotExists,
        LockIsEmpty,
        UnlockAmountIsZero,
        RewardPlanNotExists,
        CanNotReserve,
        ReservedBalanceNotEnough,
        TokenIsDICO,
        UnReleaseAssetsNotExists,
        NotIcoMember,
        DenominatorIsZero,
        Overflow,
        ProportionTooLow,
        UnreleaseAmountIsZero,
        RewardIsZero,
        MaxCountIsZero,
        IcoCountToMax,
        MaxCountErr,
        NativeCurrencyId,
        UnreleasedAmountIsZero,
        MinOrMaxIcoAmountSetErr,
        AmountNot,
        AmountNotMeetProjectRequirement,
        AmountNotMeetSystemRequirement,
    }
}