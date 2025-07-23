// extract-abi.js
const fs = require('fs');
const path = require('path');

const artifactPath = path.join(__dirname, 'build/contracts/artifacts/LuckGame.json');
const contract = require(artifactPath);

// 提取 ABI
fs.writeFileSync(
  path.join(__dirname, 'build/contracts/LuckGame.abi'),
  JSON.stringify(contract.abi, null, 2)
);

// 提取 Bytecode
fs.writeFileSync(
  path.join(__dirname, 'build/contracts/LuckGame.bin'),
  contract.bytecode.slice(2) // 移除 '0x' 前缀
);