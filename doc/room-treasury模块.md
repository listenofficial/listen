## 说明
* 这个模块是群资金管理模块
* 如果对应的房间解散，那么就会清除掉所有房间数据（解散有优先权)
* Proposals这个存储必须通过投票清理掉(有结果的投票)
***
## 数据结构
* 国库提案

## 使用流程
1. 任意用户发起一个资金使用的要求
2. 任意群议会成员通过存储发现这个请求. 认为该资金申请值得赞成approve_proposal或是应该给予反对reject_proposal，基于此在room-collective模块发起赞成或是反对议案(只需要发起其中一个)
    > 注意： 这里的反对或是赞成，其实本质上是一样的，大部分时候只需要发起其中一个议案即可。 
                                                                                                                                                                                                  
3. 其他议会成员对2步骤的议案进行投票(通过room-collective模块的vote方法)

4. 投票通过或是失败
    >  1. 如果是反对资金使用的议案, 投票通过， 那么直接删除所有资金申请记录(从此将得不到资金), 如果不通过， 那就维持议案发起之前的状态。2. 如果发起的是一个赞成的议案， 大家投票通过， 那就会可能得到资金， 如果不通过，那就维持发起议案之前的状态. 3. 如果反对资金申请议案通过，是没有反悔的余地；如果赞成资金议案通过， 还可以通过提反对资金申请的议案来驳回
5. 如果赞成资金使用的议案通过, 期间没有反对资金申请的议案通过, 并且资金使用的时间已经到, 那么可以手动领取
 ```
/// A spending proposal.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct RoomTreasuryProposal<AccountId, Balance, BlockNumber> {
	/// The account proposing it.
	proposer: AccountId, /// 提出资金议案的人
	/// The (total) amount that should be paid if the proposal is accepted.
	value: Balance, /// 金额
	/// The account to whom the payment should be made if the proposal is accepted.
	beneficiary: AccountId, /// 受益者
	/// The amount held on deposit (reserved) for making this proposal.
	bond: Balance,  /// 提出议案的人需要抵押的金额
	/// 可以消费的时间
	start_spend_time: Option<BlockNumber>,  /// 议案通过后， 多少区块高度可以开始花费
}
  
  ```
***
* storage
    * ProposalCount 每个房间对应的议案数
    * Proposals 每个房间对应的议案
    * Approvals  每个房间已经通过的等待执行的议案
    
***
## 重要方法
* 提一个资金受益的要求
    * 代码: `pub fn propose_spend(
			origin,
			room_id: RoomIndex,
			#[compact] value: BalanceOf<T>,
			beneficiary: <T::Lookup as StaticLookup>::Source
		)`
	* 参数: 
	    * room_id: 房间号
	    * value: 金额（给受益者多少)
	    * beneficiary: 受益人
	* 逻辑:
	    * 可以是任何人发起
	    * 发起人需要抵押一定数额的金额
	> 注意：这里只是要求，不是议案。议案是议员发现要求之后主动去提的，就是拒绝或是赞成这个资金要求，其他议会去投票
***
* 拒绝资金申请
    * 代码: `pub fn reject_proposal(origin, room_id: RoomIndex, #[compact] proposal_id: ProposalIndex)`
    * 参数:
        * room_id: 房间id
        * proposal_id: 议案id
    * 逻辑:
        * 过半群议员权限
        * 惩罚掉发起人抵押的金额
        * 把议案删除
***
* 赞成资金申请
    * 代码: `pub fn approve_proposal(origin, room_id: RoomIndex, #[compact] proposal_id: ProposalIndex)`
    * 参数:
        * room_id: 房间号
        * proposal_id: 议案id
    * 逻辑:
        * 过半议员权限
        * 把议案加入待执行队列
***

* 手动获取议案资金
    * 代码: `pub fn spend_fund(origin, room_id: RoomIndex)`
    * 参数:
        * room_id: 房间id
    * 逻辑:
        * 任何人都可以操作
        * 到期的议案资金全部给受益人，并且返还发起人的抵押金额
        
	    