# 存储
```buildoutcfg
decl_storage! {
	trait Store for Module<T: Config> as Dao {
		/// The hashes of the active proposals. 资产（项目）对应的所有提案hash
		pub Proposals get(fn proposals): map hasher(identity) CurrencyIdOf<T> => Vec<T::Hash>;
        
		/// Actual proposal for a given hash, if it's current. 具体提案信息
		pub ProposalOf get(fn proposal_of):
			double_map hasher(identity) CurrencyIdOf<T>, hasher(identity) T::Hash => Option<<T as Config>::Proposal>;
		/// Votes on a given proposal, if it is ongoing.提案对应的具体投票
		pub Voting get(fn voting):
			double_map hasher(identity) CurrencyIdOf<T>, hasher(identity) T::Hash => Option<RoomCollectiveVotes<T::AccountId, T::BlockNumber, MultiBalanceOf<T>>>;
		/// Proposals so far. 提案个数（历史累计)
		pub ProposalCount get(fn proposal_count): map hasher(identity) CurrencyIdOf<T> => u32;

	}
	}
```
***
# 重要数据结构
1. 投票
```buildoutcfg
pub struct RoomCollectiveVotes<AccountId, BlockNumber, MulBalance> {
	/// The proposal's unique index. 提案id
	index: ProposalIndex,
	/// The proposal's reason,  提案理由
	reason: Option<Vec<u8>>,
	/// The number of approval RoomCollectiveVotes that are needed to pass the motion. 提案的资产阀值
	threshold: MulBalance, 
	/// The current set of voters that approved it. 同意的票
	ayes: Vec<(AccountId, MulBalance)>,  
	/// The current set of voters that rejected it. 反对的票
	nays: Vec<(AccountId, MulBalance)>,  
	/// The hard end time of this vote. 提案结束的时间
	end: BlockNumber,  
}

```
***
# 常量
* ` MotionDuration` 提案存活时间
* `MaxProposals` 一种资产对应的最多提案数
# 重要方法
1. 提议案
    * 代码: `fn propose(origin,
			currency_id: CurrencyIdOf<T>,
			ico_index: u32,
			#[compact] threshold: MultiBalanceOf<T>,
			proposal: Box<<T as Config>::Proposal>,
			reason: Option<Vec<u8>>,
			#[compact] length_bound: u32
		)`
	* 参数:
	    * currency_id: 资产id（项目id)
	    * threshold: 同意票的资产阀值（多少金额后会触发)
	    * proposal: 提案对应的方法
	    * reason: 提案理由
	    * length_bound: 提案的字节长度上限
		* ico_index: ico索引
	* 逻辑:
	    * 是参与项目ico的成员
	    * 提案的字节长度不能超过最大限制
	    * 提案不存在于`ProposalOf`中
	    * 本人参与ico的资产如果大于等于阀值， 那么直接执行, 否则加入投票队列中
	    * 更新存储
	> index是自增的`ProposalCount`来生生成的
    ***
2. 投票
    * 代码: `fn vote(origin,
			currency_id: CurrencyIdOf<T>,
			ico_index: u32,
			proposal: T::Hash,
			#[compact] index: ProposalIndex,
			approve: bool,
		)`   
	* 参数:
	    * `currency_id`资产id
	    *  `proposal` 提案hash
	    *  `index` 投票id
	    * `approve` 同意或是赞成
	* 逻辑:
	    * 是参与该项目ico的成员
	    * 提案投票信息存在
	    * 提案还没过期
	    * 不能多次投一方的票
	    * 不会去检查投票是否通过(只更新投票信息)
	    * Event中有该提案的反对票总金额、赞成票总金额、以及参加ico的总金额
		* ico_index: ico索引
	***
3. 关闭提案
    * 代码: `fn close(origin,
			currency_id: CurrencyIdOf<T>,
			ico_index: u32,
			proposal_hash: T::Hash,
			#[compact] index: ProposalIndex,
			#[compact] proposal_weight_bound: Weight,
			#[compact] length_bound: u32
		)`
	* 参数:
	    * `currency_id` 资产id
	    * `proposal_hash` 提案hash
	    * `index` 提案索引
	    * `proposal_weight_bound` 提案weight上限
	    * `length_bound` 提案字节上限
		* ico_index: ico索引
	* 逻辑:
	    * 投票还存在
	    * 同意票的总资产大于等于阀值， 那就说明同意通过; ico的总资产 - 反对票的总资产 > 阀值， 那就说明反对通过; 其他情况说明投票还没有结果
	    * 如果同意通过， 就会去执行提案。 然后删除提案所有信息（关闭提案)
	    * 如果反对通过，删除提案所有信息(关闭提案)
	    * 如果该提案过期， 那么删除掉提案信息(关闭提案)
	***
4. root权限否决提案
    * 代码:
        * `fn disapprove_proposal(origin, currency_id: CurrencyIdOf<T>, proposal_hash: T::Hash)`
        * 参数:
            * `currency_id` 资产id
            * `proposal_hash` 提案hash
        * 逻辑:
            * root权限
            * 删除掉所有提案相关信息
            
	    
	    
	                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        
