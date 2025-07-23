帮助命令：
truffle help

编译命令：
truffle compile -all

部署命令：
truffle migration --network development --verbose-rpc

运行提取脚本:
node extract-abi.js

主网部署
truffle migration --network mainnet --verbose-rpc


0. 编译：truffle compile -all
1. 启动 dashboard: `truffle dashboard`
2. 提取 abi: `node extract-abi.js`
3. 部署：truffle migration --network dashboard --verbose-rpc

npm install @truffle/hdwallet-provider dotenv

npm install @openzeppelin/upgrades-core @openzeppelin/truffle-upgrades

