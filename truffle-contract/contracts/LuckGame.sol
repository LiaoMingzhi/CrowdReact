// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../node_modules/@openzeppelin/contracts/access/Ownable.sol";
import "../node_modules/@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "../node_modules/@openzeppelin/contracts/security/Pausable.sol";
import "./interfaces/ILuckGame.sol";

contract LuckGame is ILuckGame, Ownable, ReentrancyGuard, Pausable {
    // 添加投注记录结构
//    struct BetRecord {
//        uint256 amount;
//        uint256 number;
//        uint256 timestamp;
//    }

    // 添加用户投注记录映射
    //mapping(address => BetRecord[]) public betRecords;
    
    // 状态变量
    mapping(address => uint256) private _balances;
//    mapping(address => uint256) private _commissions;
//    mapping(address => bool) private _agents;
//    mapping(address => address) private _agentRelations;
    
    uint256 public constant MIN_BET = 0.001 ether;
//    uint256 public constant MAX_BET = 10 ether;
//    uint256 public constant COMMISSION_RATE = 500; // 5% = 500 basis points
//    uint256 private constant BASIS_POINTS = 10000;
    
    constructor() Ownable() Pausable() {
        _transferOwnership(msg.sender);  // 显式设置所有者
    }

    // Add this event with the other events at the top of the contract
    event FundsTransferred(address indexed recipient, uint256 amount);
    
    // 下注函数 - 删除重复的 placeBet 函数，保留这个实现
    function placeBet() external payable override nonReentrant whenNotPaused {
        if (msg.value < MIN_BET) {
            revert InvalidBetAmount(msg.value);
        }
//        if (number < 1 || number > 99) {
//            revert InvalidNumber(number);
//        }
        
        _balances[msg.sender] += msg.value;
        
        // 处理代理佣金
//        address agent = _agentRelations[msg.sender];
//        if (agent != address(0) && _agents[agent]) {
//            uint256 commission = (msg.value * COMMISSION_RATE) / BASIS_POINTS;
//            _commissions[agent] += commission;
//        }
        
        // 记录投注信息
//        betRecords[msg.sender].push(BetRecord({
//            amount: msg.value,
//            number: number,
//            timestamp: block.timestamp
//        }));
        
        emit Bet(msg.sender, msg.value);  // 使用接口定义的事件
    }
    
    // 提现函数
//    function withdraw() external override nonReentrant {
//        uint256 amount = _balances[msg.sender];
//        if (amount == 0) revert NoBalance();
//
//        _balances[msg.sender] = 0;
//
//        (bool success, ) = payable(msg.sender).call{value: amount}("");
//        if (!success) revert TransferFailed();
//
//        emit Win(msg.sender, amount);
//    }

    // Add this function after the withdraw() function and before _processBet()
    function transferFunds(address to, uint256 amount) external onlyOwner nonReentrant {
        require(to != address(0), "Invalid recipient address");
        require(amount > 0, "Amount must be greater than 0");
        require(address(this).balance >= amount, "Insufficient contract balance");

        // Transfer the funds
        (bool success, ) = to.call{value: amount}("");
        require(success, "Transfer failed");

        emit FundsTransferred(to, amount);
    }
    
    // 提取佣金
//    function withdrawCommission() external override nonReentrant {
//        if (!_agents[msg.sender]) revert NotAgent(msg.sender);
//
//        uint256 amount = _commissions[msg.sender];
//        if (amount == 0) revert NoBalance();
//
//        _commissions[msg.sender] = 0;
//
//        (bool success, ) = payable(msg.sender).call{value: amount}("");
//        if (!success) revert TransferFailed();
//
//        emit AgentCommission(msg.sender, amount);
//    }
    
    // 管理函数
//    function addAgent(address agent) external override onlyOwner {
//        _agents[agent] = true;
//        emit AgentAdded(agent);
//    }
//
//    function removeAgent(address agent) external override onlyOwner {
//        _agents[agent] = false;
//        emit AgentRemoved(agent);
//    }
//
//    function setAgentRelation(address user, address agent) external override onlyOwner {
//        if (!_agents[agent]) revert NotAgent(agent);
//        _agentRelations[user] = agent;
//        emit AgentRelationSet(user, agent);
//    }
    
    // 查询函数
    function getBalance(address user) external view override returns (uint256) {
        return _balances[user];
    }
    
//    function getAgentCommission(address agent) external view override returns (uint256) {
//        return _commissions[agent];
//    }
//
//    function isAgent(address user) external view override returns (bool) {
//        return _agents[user];
//    }
//
//    function getAgentForUser(address user) external view override returns (address) {
//        return _agentRelations[user];
//    }
    
    // 紧急功能
    function pause() external onlyOwner {
        _pause();
    }
    
    function unpause() external onlyOwner {
        _unpause();
    }
    
    // 修改 receive 函数
    receive() external payable {
        // 检查转账金额是否在允许范围内
        if (msg.value < MIN_BET) {
            revert InvalidBetAmount(msg.value);
        }
        
        // 调用内部下注处理函数
        _processBet(msg.sender, msg.value); // 默认下注数字为 1
    }

    // 修改内部处理函数
    function _processBet(address bettor, uint256 amount) internal {
        require(amount > 0, "Bet amount must be greater than 0");
        //require(number >= 1 && number <= 99, "Bet number must be between 1 and 99");
        
        _balances[bettor] += amount;
        
        // 处理代理佣金
//        address agent = _agentRelations[bettor];
//        if (agent != address(0) && _agents[agent]) {
//            uint256 commission = (amount * COMMISSION_RATE) / BASIS_POINTS;
//            _commissions[agent] += commission;
//        }
        
        // 记录投注信息
//        betRecords[bettor].push(BetRecord({
//            amount: amount,
//            number: number,
//            timestamp: block.timestamp
//        }));
        
        emit Bet(bettor, amount);  // 使用接口定义的事件
    }




}