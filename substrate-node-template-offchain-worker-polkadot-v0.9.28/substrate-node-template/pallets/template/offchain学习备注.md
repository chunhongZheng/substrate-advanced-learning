#### offchain相关函数

fn offchain_worker(block_number: T::BlockNumber){}
fn on_initialize(_n: T::BlockNumber) -> Weight {}
fn on_finalize(_n: T::BlockNumber)()
fn on_idle(_n: T::BlockNumber, _remaining_weight: Weight) -> Weight{}


#### 运行结果顺序：

2022-09-20 10:30:06 ✨ Imported #3 (0x1c59…9761)    
2022-09-20 10:30:06 Hello World from offchain workers!: 3    
2022-09-20 10:30:09 💤 Idle (0 peers), best: #3 (0x1c59…9761), finalized #1 (0xa5cb…abe0), ⬇ 0 ⬆ 0    
2022-09-20 10:30:12 🙌 Starting consensus session on top of parent 0x1c59453140856d7dc7f8b6af509b51a637973c970e52b797bc5dd53bb2e39761    
2022-09-20 10:30:12 in on_initialize!    
2022-09-20 10:30:12 in on_idle!    
2022-09-20 10:30:12 in on_finalize!  

#### 执行cargo build --release出现以下报错信息：

Blocking waiting for file lock on build directory
解决办法为：先control + c 终止当前界面，然后切换到根目录，删除掉~/.cargo/.package-cache


#### * 使用mutate方法对数据进行原子更改，适用多线程环境。

mutate使用compare-and-set模式。它会对比存储位置的内容与给定值是否一致，只有相同时，该存储位置才会被修改为新的内容。
这是一个原子操作，它保证了根据最新信息计算新的数值；如果该值在同一时间被另一个线程更新，则写入操作会失败。



#### 向链上发送交易例子
需要修改多处地方，node,runtime及pallet代码。

###### 一: 调整node/service.rs
1： 在cargo.toml的dependencies添加依赖
sp-keystore= { version = "0.12.0", git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.28" }

在node/service.rs中将下面这段代码
在let client = Arc::new(client); 之后
新增以下配置
`if config.offchain_worker.enabled {
let keystore = keystore_container.sync_keystore();
sp_keystore::SyncCryptoStore::sr25519_generate_new(
&*keystore,
node_template_runtime::pallet_template::KEY_TYPE,
Some("//Alice"),
).expect("Creating key with account Alice should succeed.");
}`

#### 二：修改runtime/lib.rs

//添加offchain runtime 发送交易相关配置开始

`impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
Call: From<LocalCall>,
{
fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
call: Call,
public: <Signature as sp_runtime::traits::Verify>::Signer,
account: AccountId,
nonce: Index,
) -> Option<(Call, <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload)> {
let tip = 0;

		let period =
			BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			.saturating_sub(1);
		let era = generic::Era::mortal(period, current_block);
		let extra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(era),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			//pallet_asset_tx_payment::ChargeAssetTxPayment::<Runtime>::from(tip, None),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|_| {
				//log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = account;
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (sp_runtime::MultiAddress::Id(address), signature.into(), extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
type Public = <Signature as sp_runtime::traits::Verify>::Signer;
type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
Call: From<C>,
{
type Extrinsic = UncheckedExtrinsic;
type OverarchingCall = Call;
}

//添加offchain runtime 发送交易相关配置结束
`

#### 二：修改template/lib.rs
1：增加crypto包
`
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");
pub mod crypto {
use super::KEY_TYPE;
use sp_core::sr25519::Signature as Sr25519Signature;
use sp_runtime::{
app_crypto::{app_crypto, sr25519},
traits::Verify,
MultiSignature, MultiSigner,
};
app_crypto!(sr25519, KEY_TYPE);

    pub struct OcwAuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
        type RuntimeAppPublic = Public;
        type GenericPublic = sp_core::sr25519::Public;
        type GenericSignature = sp_core::sr25519::Signature;
    }

    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
    for OcwAuthId
    {
        type RuntimeAppPublic = Public;
        type GenericPublic = sp_core::sr25519::Public;
        type GenericSignature = sp_core::sr25519::Signature;
    }
}
`
2：修改pallet的config配置，如下
#[pallet::config]
pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
/// Because this pallet emits events, it depends on the runtime's definition of an event.
type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
}



3：发签名交易的函数
        fn send_signed_tx(payload: Vec<u8>) -> Result<(), &'static str> {
            let signer = Signer::<T, T::AuthorityId>::all_accounts();
            if !signer.can_sign() {
                return Err(
                    "No local accounts available. Consider adding one via `author_insertKey` RPC.",
                )
            }

            let results = signer.send_signed_transaction(|_account| {

                Call::submit_data { payload: payload.clone() }
            });

            for (acc, res) in &results {
                match res {
                    Ok(()) => log::info!("[{:?}] Submitted data:{:?}", acc.id, payload),
                    Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
                }
            }

            Ok(())
        }