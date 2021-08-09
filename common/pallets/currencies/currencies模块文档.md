# 存储
* 资产信息
```buildoutcfg
#[pallet::storage]
	/// Metadata of an asset.
	pub(super) type DicoAssetsInfo<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		CurrencyIdOf<T>,
		DicoAssetInfo<T::AccountId, DicoAssetMetadata>,
	>;
```
# 数据结构
```buildoutcfg
pub struct DicoAssetInfo<AccountId, DicoAssetMetadata> {
	owner: AccountId,  /// 资产所有者
	metadata: Option<DicoAssetMetadata>,  /// 资产metadata
}
```
***
```buildoutcfg
pub struct DicoAssetMetadata {
	/// project name
	pub name: Vec<u8>,  /// 项目名称
	/// The ticker symbol for this asset. Limited in length by `StringLimit`.
	pub symbol: Vec<u8>, /// 代币名称
	/// The number of decimals this asset uses to represent one unit.
	pub decimals: u8, /// 精度

}
```
# 重要方法
1. 创建资产
    * 代码: `fn create_asset(origin: OriginFor<T>, currency_id: CurrencyIdOf<T>, amount: BalanceOf<T>)`
    * 参数:
        * `currency_id` 资产id
        * `amount` 金额
    * 逻辑：
        * 资产信息还没有存在并且该资产的总金额是0
        * 给该账户铸币， 并且把该账户设置成该资产所有者
    ***
2. 设置资产元数据
    * 代码: `fn set_metadata(origin: OriginFor<T>, currency_id: CurrencyIdOf<T>, metadata: DicoAssetMetadata)`
    * 参数:
        * `currency_id` 资产id
        * `metadata` 元数据
    * 逻辑:
        * metadata要符合基本要求
             > name长度大于2， symbol长度大于1， 精度不能是0
        * 资产要存在(之前创建过)metadata
        * 资产所有者才能设置
        * 如果之前的metadata存在， 那么新的metadata不能与旧的相同, 并且精度不能修改
            > 修改精度会导致数据失误， 所以DICO不能修改精度
    ***
3. 自己销毁自己的代币
    * 代码: `fn burn(origin: OriginFor<T>, currency_id: CurrencyIdOf<T>, amount: BalanceOf<T>)`
    * 参数:
        * `currency_id`资产id
        * `amount` 销毁金额
    * 逻辑:
        * metadata存在
        * 销毁
    ***
4. 转账
    * 代码: `pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			currency_id: CurrencyIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
		)`
	* 参数:
	    * `dest`目标地址
	    * `currency_id` 资产id
	    * `amount` 金额
	* 逻辑：
	    * metadata存在
	> 该方法用于所有代币的转账， 包括本链原生资产
	***
5. 原生代币转账
    * 代码: `pub fn transfer_native_currency(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] amount: BalanceOf<T>,
		)`
	* 参数: 略
    ***
6. 更新资产
    * 代码: `pub fn update_balance(
			origin: OriginFor<T>,
			who: <T::Lookup as StaticLookup>::Source,
			currency_id: CurrencyIdOf<T>,
			amount: AmountOf<T>,
		)`
	* 参数:
	    * who: 目标账户
	    * currency_id: 资产id
	    * amount: 金额
	* 逻辑:
	    * root权限
	    * 设置账户金额为amount
	***
	
                                    