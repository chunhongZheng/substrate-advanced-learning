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

`	// if config.offchain_worker.enabled {
	// 	sc_service::build_offchain_workers(
	// 		&config,
	// 		task_manager.spawn_handle(),
	// 		client.clone(),
	// 		network.clone(),
	// 	);
	// }`
调整为
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





        //example2
        // 在奇数块向Local Storage写数据，偶数块读取数据，并检查。
        fn example2(block_number: T::BlockNumber) {
            log::info!("Hello World from offchain workers!: {:?}", block_number);
            if block_number % 2u32.into() != Zero::zero(){
                //odd 奇数
                let key = Self::derive_key(block_number);
                let val_ref = StorageValueRef::persistent(&key);

                //  get a local random value  链下随机数
                //pub fn random_seed() -> [u8; 32]
                let random_slice = sp_io::offchain::random_seed();
                log::info!("in odd block, value to  local random_slice write: {:?}", random_slice);
                //  get a local timestamp
                let timestamp_u64 = sp_io::offchain::timestamp().unix_millis();
                log::info!("in odd block, value to  local timestamp write: {:?}", timestamp_u64);
                // combine to a tuple and print it
                let value = (random_slice, timestamp_u64);
                log::info!("in odd block, value to write: {:?}", value);

                //  write or mutate tuple content to key
                val_ref.set(&value);
            }else{
                //偶数 even
                let key = Self::derive_key(block_number - 1u32.into());
                let mut val_ref = StorageValueRef::persistent(&key);

                // get from db by key
                //pub fn get<T: Decode>(&self) -> Result<Option<T>, StorageRetrievalError>
                //val_ref.get();


                if let Ok(Some(value)) = val_ref.get::<([u8;32], u64)>() {
                    // print values
                    log::info!("in even block, value read: {:?}", value);
                    // delete that key
                    val_ref.clear();
                }
            }
            log::info!("Leave from offchain workers!: {:?}", block_number);
        }

        //example3
        fn example3(block_number: T::BlockNumber){
            log::info!("Hello World from offchain workers!: {:?}", block_number);
            if block_number % 2u32.into() != Zero::zero(){
                // odd
                let key = Self::derive_key(block_number);
                let val_ref = StorageValueRef::persistent(&key);
                //  get a local random value
                let random_slice = sp_io::offchain::random_seed();
                //  get a local timestamp
                let timestamp_u64 = sp_io::offchain::timestamp().unix_millis();
                // combine to a tuple and print it
                let value = (random_slice, timestamp_u64);
                log::info!("in odd block, value to write: {:?}", value);
                struct StateError;
                //  write or mutate tuple content to key
                //source
                // pub fn mutate<T, E, F>(
                //     &self,
                //     mutate_val: F
                // ) -> Result<T, MutateStorageError<T, E>>
                // where
                //     T: Codec,
                //     F: FnOnce(Result<Option<T>, StorageRetrievalError>) -> Result<T, E>
                //mutate使用compare-and-set模式。它会对比存储位置的内容与给定值是否一致，只有相同时，该存储位置才会被修改为新的内容。这是一个原子操作，它保证了根据最新信息计算新的数值；如果该值在同一时间被另一个线程更新，则写入操作会失败。
                let res = val_ref.mutate(|val: Result<Option<([u8;32], u64)>, StorageRetrievalError>| -> Result<_, StateError> {
                    match val {
                        Ok(Some(_)) => Ok(value),
                        _ => Ok(value),
                    }
                });
                match res {
                    Ok(value) => {
                        log::info!("in odd block, mutate successfully: {:?}", value);
                    },
                    Err(MutateStorageError::ValueFunctionFailed(_)) => (log::info!("this is valueFunctionFailed error")),
                    Err(MutateStorageError::ConcurrentModification(_)) => (log::info!("this is concurrentModfication error")),
                }
            }else{
                // even
                let key = Self::derive_key(block_number - 1u32.into());
                let mut val_ref = StorageValueRef::persistent(&key);

                // get from db by key
                if let Ok(Some(value)) = val_ref.get::<([u8;32], u64)>() {
                    // print values
                    log::info!("in even block, value read: {:?}", value);
                    // delete that key
                    val_ref.clear();
                }
            }

            log::info!("Leave from offchain workers!: {:?}", block_number);
        }

        //发送http请求 获取数据
        fn example4(block_number: T::BlockNumber){
            log::info!("Hello World from offchain workers!: {:?}", block_number);
            if let Ok(info) = Self::fetch_github_info() {

                log::info!("Github Info: {:?}", info);

            } else {

                log::info!("Error while fetch github info!");

            }
            log::info!("Leave from offchain workers!: {:?}", block_number);
     





        fn example1(block_number: T::BlockNumber) {
            log::info!("Hello World from offchain workers!: {:?}", block_number);

            let timeout = sp_io::offchain::timestamp()
                .add(sp_runtime::offchain::Duration::from_millis(8000));

            sp_io::offchain::sleep_until(timeout);

            log::info!("Leave from offchain workers!: {:?}", block_number);
        }
