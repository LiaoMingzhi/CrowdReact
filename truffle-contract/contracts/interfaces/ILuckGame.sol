// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title ILuckGame
 * @dev 幸运游戏合约的接口定义
 */
interface ILuckGame {
    /**
     * @dev 事件定义
     * 记录所有重要的状态变更
     */
    
    /// @notice 用户下注时触发
    /// @param user 下注用户地址
    /// @param amount 下注金额
    event Bet(address indexed user, uint256 amount);

    /// @notice 用户提现时触发
    /// @param user 提现用户地址
    /// @param amount 提现金额
    // event Win(address indexed user, uint256 amount);

    /// @notice 代理提取佣金时触发
    /// @param agent 代理地址
    /// @param amount 佣金金额
    // event AgentCommission(address indexed agent, uint256 amount);

    /// @notice 添加新代理时触发
    /// @param agent 新代理地址
    // event AgentAdded(address indexed agent);

    /// @notice 移除代理时触发
    /// @param agent 被移除的代理地址
    // event AgentRemoved(address indexed agent);

    /// @notice 设置用户代理关系时触发
    /// @param user 用户地址
    /// @param agent 代理地址
    // event AgentRelationSet(address indexed user, address indexed agent);

    /**
     * @dev 错误定义
     * 使用自定义错误来节省 gas 并提供更好的错误信息
     */
    
    /// @notice 下注金额无效时抛出
    error InvalidBetAmount(uint256 amount);

    /// @notice 下注数字无效时抛出
    // error InvalidNumber(uint256 number);

    /// @notice 非代理调用代理功能时抛出
    // error NotAgent(address caller);

    /// @notice 尝试提取零余额时抛出
    // error NoBalance();

    /// @notice 转账失败时抛出
    // error TransferFailed();

    /**
     * @dev 函数定义
     */
    
    /// @notice 下注函数
    /// @dev 必须附带适当的 ETH 金额
    function placeBet() external payable;

    /// @notice 提现函数
    /// @dev 提取用户的所有可用余额
    // function withdraw() external;

    /// @notice 提取代理佣金
    /// @dev 只能由代理调用
    // function withdrawCommission() external;

    /// @notice 添加新代理
    /// @param agent 要添加的代理地址
    /// @dev 只能由合约所有者调用
    // function addAgent(address agent) external;

    /// @notice 移除代理
    /// @param agent 要移除的代理地址
    /// @dev 只能由合约所有者调用
    // function removeAgent(address agent) external;

    /// @notice 设置用户的代理关系
    /// @param user 用户地址
    /// @param agent 代理地址
    /// @dev 只能由合约所有者调用
    // function setAgentRelation(address user, address agent) external;

    /// @notice 查询用户余额
    /// @param user 用户地址
    /// @return 用户的当前余额
    function getBalance(address user) external view returns (uint256);

    /// @notice 查询代理佣金
    /// @param agent 代理地址
    /// @return 代理的当前佣金余额
    // function getAgentCommission(address agent) external view returns (uint256);

    /// @notice 检查地址是否为代理
    /// @param user 要检查的地址
    /// @return 是否为代理
    // function isAgent(address user) external view returns (bool);

    /// @notice 获取用户的代理
    /// @param user 用户地址
    /// @return 用户的代理地址
    // function getAgentForUser(address user) external view returns (address);
}