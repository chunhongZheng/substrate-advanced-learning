Substrate benchmark 备注

查看帮助
./target/production/node-template benchmark --help

./target/production/node-template benchmark pallet --help

查看列表
./target/production/node-template benchmark pallet --list


单元测试命令

cargo test --package pallet-poe --features runtime-benchmarks
cargo test --package pallet-mycustom --features runtime-benchmarks


编译
cd bin/node/cli
cargo build --profile=production --features runtime-benchmarks

cargo build --package node-template --release --features runtime-benchmarks


运行：
./target/production/node-template benchmark pallet \
--chain dev \
--execution=wasm \
--wasm-execution=compiled \
--pallet pallet_balances \
--extrinsic transfer \
--steps 50 \
--repeat 20 \
--output pallets/transfer-weight.rs

运行命令

./target/release/node-template benchmark pallet --template .maintain/frame-weight-template.hbs --pallet pallet_poe --extrinsic "*" --output ./pallets/poe/src/we.rs --steps 20 --repeat 10 --json-file=raw.json








