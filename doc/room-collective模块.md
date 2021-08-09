## 说明
* 该模块是群组的议会模块
* 每个群组的议员通过群里消费排名产生(前15名)
## 数据结构
* 议会投票
```
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
/// Info for keeping track of a motion being voted on.
pub struct RoomCollectiveVotes<AccountId, BlockNumber> {
	/// The proposal's unique index.
	index: ProposalIndex,
	/// The proposal's reason,
	reason: Option<Vec<u8>>, /// 议案原因
	/// The number of approval RoomCollectiveVotes that are needed to pass the motion.
	threshold: MemberCount,
	/// The current set of voters that approved it.
	ayes: Vec<AccountId>,
	/// The current set of voters that rejected it.
	nays: Vec<AccountId>,
	/// The hard end time of this vote.
	end: BlockNumber,
}
```
***
## 重要方法
* 直接执行(不需要通过投票， 一半是群里root权限)
    * 代码: `fn execute(origin,
			room_id: RoomIndex,
			proposal: Box<<T as Config<I>>::Proposal>,
			#[compact] length_bound: u32,
		)`
    * 参数: 
        * room_id: 房间id
        * proposal: 执行的方法
        * length_bound: 方法的字节数上限
    * 逻辑:
        * 群主或是群员
        * 方法的字节数不能太大
        * 执行提案里的方法
    > 这个方法是直接用某个身份去执行，不需要走投票流程。比如说我说我需要议会成员权限来执行某个操作, 这时候没有说明多少个议会成员赞成才能通过， 那么就用这个方法
***
* 提议案
    * 代码: `fn propose(origin,
			room_id: RoomIndex,
			#[compact] threshold: MemberCount,
			proposal: Box<<T as Config<I>>::Proposal>,
			reason: Option<Vec<u8>>,
			#[compact] length_bound: u32
		)`
	* 参数: 
	    * room_id: 房间号
	    * threshold: 多少票通过才会执行
	    * proposal: 要执行的方法
	    * reason: 议案的理由
	    * length_bound: 方法的字节数上限
	* 逻辑:
	    * 议员权限
	    * 字节数不能过大
	    * 房间里还没有该议案
	    * threshold小于2直接执行
	    * 加入房间投票队列
	    * 房间议案id累加1
***
* 给议案投票
    * 代码: `fn vote(origin,
			room_id: RoomIndex,
			proposal: T::Hash,
			#[compact] index: ProposalIndex,
			approve: bool,
		)`
	* 参数:
	    * room_id: 房间号
	    * proposal: 议案hash
	    * index: 议案id
	    * approve: 赞成或是反对
	* 逻辑:
	    * 群组议员权限
	    * 投票存在
	    * 议案id对得上
	    * 不能重复投但是可以悔投
	    * 通过投票的话立即执行
	
***
* ~~关闭议案(最新版本已经取消掉这个方法)~~
    * 代码: `fn close(origin,
			room_id: RoomIndex,
			proposal_hash: T::Hash,
			#[compact] index: ProposalIndex,
			#[compact] proposal_weight_bound: Weight,
			#[compact] length_bound: u32
		)`       
	* 参数: 
	    * room_id: 房间id
	    * proposal_hash: 议案hash
	    * index: 议案id
	    * proposal_weight_bound: 议案权重最大值
	    * length_bound: 议案字节最大值
	* 逻辑:
	    * 投票存在
	    * 议案id对得上
	    * 用赞成票来决定是否通过(赞成票大于或是小于等于阀值)
	    * 投票有结果(明确知道反对或是赞成), 执行相应操作然后关闭
	    * 投票结束时间还没到并且投票还没有有结果， 不能关闭
***
* listen root权限执行议案
    * 代码: `fn disapprove_proposal(origin, room_id: RoomIndex, proposal_hash: T::Hash)`
    * 参数: 
        * room_id: 房间号
***
	                                                                                                                                                                                                                                                                             
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                
        
