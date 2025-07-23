const fs = require('fs');
const LuckGame = artifacts.require("LuckGame");

module.exports = async function(deployer, network, accounts) {
  // 获取或生成私钥
  const privateKey = web3.eth.accounts.create().privateKey.substring(2); // 移除 "0x" 前缀
  
  // 部署合约
  await deployer.deploy(LuckGame);
  const instance = await LuckGame.deployed();
  
  // 保存部署信息
  const deployInfo = {
    contract: instance.address,
    owner: accounts[0],
    privateKey: privateKey,
    network,
    deployedAt: new Date().toISOString()
  };

  // 写入 deploy-info.json
  fs.writeFileSync(
    'deploy-info-development.json',
    JSON.stringify(deployInfo, null, 2)
  );
};