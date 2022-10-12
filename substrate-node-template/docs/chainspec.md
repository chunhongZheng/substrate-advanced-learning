创建chain spec json文件
./target/release/node-template build-spec>local-testnet-spec.json

加密chain spec json文件
./target/release/node-template build-spec --chain=local-testnet-spec.json --raw>local-testnet-spec-raw.json