# 存储
```buildoutcfg
decl_storage! {
    trait Store for Module<T: Config> as IcoModule {

        /// 正在等待ico
        pub PendingIco get(fn pending_ico): Vec<(IcoInfo<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries, T::AccountId>, BalanceOf<T>)>;

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

    }
}
```
# 常量
* `GetNativeCurrencyId` 本链资产id
* `InitiatorPledge` 发起ico的项目方需要抵押的DICO数量
* `RequestPledge` 项目方要求释放资金时候的
* `RequestExpire` 申请释放资金的提案的存活时间
* `MinProportion` 项目方筹集资金达到的最低比例要求
* `NativeMultiple` 如果用DICO来ico 奖励倍数是多少
# 重要数据结构
* ico信息
```buildoutcfg
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
```
***
* 参与ico的相关资产信息
```buildoutcfg
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
```
***
```buildoutcfg
pub struct PendingInfo<IcoInfo, Balance> {
    pub ico: IcoInfo,  // ico信息
    pub pledge: Balance,  // 发起ico需要抵押的金额

}
```
***
```buildoutcfg
pub struct IcoMultiple {
    pub numerator: u32,  // 分子
    pub denominator: u32 // 分母
}
```
***
```buildoutcfg
pub struct IcoLock<Balance, BlockNumber> {
    start_block: BlockNumber,  // 琐仓开始的时间
    index: u32,  // ico索引
    total_amount: Balance,  // 琐仓总金额
    unlock_amount: Balance,  // 已经解锁的金额
    unlock_duration: BlockNumber,  // 多久解锁一次
    per_duration_unlock_amount: Balance,  // 一次解锁多少

}
```
***
```buildoutcfg
pub struct Release<AccountId, Block, CurrencyId, NativeBalance> {
    who: AccountId,  // 谁请求释放
    currency_id: CurrencyId,  // 资产id
    index: u32, // ico索引
    request_time: Block, // 请求释放的时间
    // amount: Balance,
    percent: Percent,  // 请求释放到多少比例
    pledge: NativeBalance,  // 请求时需要抵押的金额
}
```
***


# 重要方法
1. 项目方发起ico
    * 代码 `fn initiate_ico(origin, info: IcoParameters<T::BlockNumber, MultiBalanceOf<T>, CurrencyIdOf<T>, Countries>)`
    * 参数说明
        * info: 关于ico的信息
    * 逻辑:
        * 任何人都可以发起
        * 已经设置过该资产的metadata（在currencies模块中设置)
        * 该用户资产存在并且足够
        * 填写的ico信息符合基本逻辑
        * 还没有在PendingIco队列
        * 抵押该用户用于ico的代币数量以及发起ico需要抵押的金额`InitiatorPledge`
        * 把该请求信息写入`PendingIco`中
    > 不需要在此时把代币换算成usdt 换算是用户加入的时候才做的
    ***    
2. 基金会成员过半同意项目方发起ico请求
    * 代码 `fn permit_ico(origin, currency_id: CurrencyIdOf<T>)`
    * 参数说明:
        * currency_id: 发起ico请求的项目方资产id
    * 逻辑:
        * 过半议员权限
        * ico请求在`PendingIco`中
        * 释放项目方抵押的金额`InitiatorPledge`
        * 在`PendingIco`中删除该信息
        * 添加信息到`Ico`中
    ***
3. 基金会成员过半拒绝项目方发起的ico请求
    * 代码 `fn reject_ico(origin, currency_id: CurrencyIdOf<T>)`
    * 参数:
        * currency_id: 项目方的资产id
    * 逻辑:
        * 过半议员权限
        * ico请求在`PendingIco`中
        * 惩罚抵押的金额`InitiatorPledge`
        * 在`PendingIco`中删除该信息
    ***
4. 用户参加ico
    * 代码`fn join(origin, currency_id: CurrencyIdOf<T>, index: u32, amount: MultiBalanceOf<T>)`
    * 参数:
        * currency_id: 项目方ico的资产id
        * amount: 主流币数量
        * index: ico的索引，也就是id
    * 逻辑：
        * `amount`不能为0
        * 该ico存在
        * ico还没有被终止或是过期
        * 项目方账号不能参与ico
        * 用户不能是被排除在外的国家
        * 用户的参与次数不能超过上限
        * 用户该主流币的资产足够
        * 用户的代币转给项目方并进行锁仓， 项目方的保留代币转移到用户的自由账户并琐仓
        * 筹集的主流币不能超过项目方要求的上限
        * 用户该主流币的usdt价值要在系统与项目方要求的区间
        * 更新`UnReleaseAssets`信息

    ***
5. 群DAO终止项目ico
    * 代码 `fn terminate_ico(origin, currency_id: CurrencyIdOf<T>, index: u32)`
    * 参数:
        * currency_id: 项目方ico的资产id
    * index: ico索引
    * 逻辑：
        * ico存在
        * ico被终止
        * 更新`Ico`中的`is_terminated`字段为true
    ***
6. 项目方要求释放资产  
    * 代码 `fn request_release(origin, currency_id: CurrencyIdOf<T>, index: u32, percent: Percent)`
    * 参数:
        * currency_id: 项目方ico的资产id
        * percent: 释放多少比例的资金
            > 不是本次释放的比例， 是总共释放的比例
    * index: ico索引
	
    * 逻辑：
        * percent比已经存在的释放比例要大
        * ico存在并且是项目方
        * ico还没有被终止
            > 如果只是过期， 是可以的
        * 抵押金额`RequestPledge`
        * 添加信息至`RequestReleaseInfo`
    ***
7. 项目方自动取消申请资金的提案
    * 代码 `cancel_request(origin, currency_id: CurrencyIdOf<T>, index: u32)`
    * 参数:
      * currency_id: 资产id
      * index: ico索引
    * 逻辑:
	
        * 项目方ico的资产id
        * ico存在并且自己是项目方
        * 有申请过的记录（申请但是还未通过)
        * 惩罚掉1/2抵押金额`RequestPledge`
        * 剩余的1/2抵押金额解除抵押
	
     ***
8. 项目DAO允许资金释放请求
    * 代码 `fn permit_release(origin, currency_id: CurrencyIdOf<T>)`
    * 参数:
        * currency_id: 项目方ico的资产id
    * 逻辑:
        * ico存在
        * 有申请资金释放的提案
        * 提案没有被终止
          > 过期可以  
        * 本次释放的比例参数比ico中已经存在的要大
     ***
9. 用户解锁自己资产(释放资金时候有锁仓要求的部分资产)
    * 代码 `fn unlock(origin, currency_id: CurrencyIdOf<T>)`
    * 参数:
        * currency_id: 项目方ico的资产id
    * 逻辑：
        * 该资产有被锁仓记录(在IcoLocks中)
        * 能解锁的资产不能为0
    ***
10. 获取奖励
    * 代码 `get_reward(origin, currency_id: CurrencyIdOf<T>, index: u32)`
    * 参数:
        * currency_id: 项目方ico的资产id
        * index: ico索引
    * 逻辑：
        * ico存在
        * 必须是参与ico的用户
        * 每个人都要手动去领自己的奖励
        * ico已经结束或是被终止
        * 该用户的奖励要大于0
    ***
11. 用户(或是项目方)手动释放资产
    * 代码 `fn user_release_ico_amount(origin, currency_id: CurrencyIdOf<T>, index: u32)`
    * 参数:
        * currency_id 资产id
        * index ico索引
    * 逻辑:
        * ico存在
        * 根据ico中可释放的比例来释放自己的金额(有还未释放的进行下面的操作)
        * 本次释放的资产要大于0
        * 如果是项目方则全部解锁
        * 如果是用户， 有锁仓要求的部分要锁仓， 剩余部分释放
        * 如果是项目已经过期或是已经被强制终止，则继续进行下面的操作
            1. 如果项目方申请ico时候抵押的保留代币还有未解抵押，也就是没有百分之百完成ico，那么就要解剩下的这部分保留代币；如果是没有满足60%， 这部分要被惩罚掉
            2. 如果操作的是用户，并且项目被强制终止， 那么执行下面操作
                * 项目方如果没有完成60%， 则销毁掉手中的代币；如果完成， 转给项目方
                * 项目方的等额主流币转给用户
    ***
12. 项目方设置用户ico金额区间
    * 代码: `fn initiator_set_ico_amount_bound(origin, currency_id: CurrencyIdOf<T>, index: u32, min_amount: MultiBalanceOf<T>, max_amount: MultiBalanceOf<T>)`
    * 参数:
        * currency_id 资产id
        * index ico索引
        * min_amount 最小金额
        * max_amount 最大金额
    * 逻辑：
        * 是项目方
        * 最大值要大于最小值
    ***
13. 项目方设置单个用户ico最大次数
    * 代码 `fn set_ico_max_count(origin, currency_id: CurrencyIdOf<T>, index: u32, max_count: u8)`
    * 参数:
        * currency_id 资产id
        * index ico索引
        * max_count 单个用户ico最大次数
    * 逻辑:
        * 是项目方
        * max_count不能与上次的次数相同， 并且不能是0
    ***
14. 公投设置系统级别的ico区间金额
    * 代码:`fn set_system_ico_amount_bound(origin, min_amount: MultiBalanceOf<T>, max_amount: MultiBalanceOf<T>)`
    * 参数:
        * min_amount
        * max_amount
    * 逻辑:  
        * 公投权限
        * 最大值大于等于最小值， 并且最大值不能是0
    
    
        
        
        
