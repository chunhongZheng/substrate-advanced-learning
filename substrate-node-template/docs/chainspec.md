创建chain spec json文件
./target/release/node-template build-spec>local-testnet-spec.json

加密chain spec json文件
./target/release/node-template build-spec --chain=local-testnet-spec.json --raw>local-testnet-spec-raw.json


部署公开测试网络

./target/release/node-template \
--base-path /tmp/bootnode1\
--chain local-testnet-spec-raw.json \
--name bootnode1