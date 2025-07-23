import { IContractService } from './contract'
import { ethers } from 'ethers'
import axios from 'axios'

export class BettingService {
  private accountChangeHandler: ((accounts: string[]) => void) | null = null;

  constructor(private contractService: IContractService) {
    // 添加账号变更监听
    if (window.ethereum) {
      this.accountChangeHandler = (accounts: string[]) => {
        console.log('MetaMask account changed to:', accounts[0]);
        // 可以在这里添加账号变更后的处理逻辑
        window.location.reload(); // 刷新页面以更新状态
      };
      
      window.ethereum.on('accountsChanged', this.accountChangeHandler);
    }
  }

  // 在组件销毁时清理监听器
  dispose() {
    if (window.ethereum && this.accountChangeHandler) {
      window.ethereum.removeListener('accountsChanged', this.accountChangeHandler);
    }
  }

  async placeBet(amount: string, betNumber: number) {
    try {
      // 1. 检查 MetaMask
      const ethereum = window.ethereum
      if (!ethereum) {
        throw new Error('请先安装 MetaMask 钱包')
      }

      // 2. 请求连接钱包
      const accounts = await ethereum.request({ 
        method: 'eth_requestAccounts' 
      })
      const userAddress = accounts[0]
      console.log('Using account for betting:', userAddress);

      // 3. 获取交易参数
      const params = new URLSearchParams({
        from: userAddress,
        amount: amount.toString(),
      })

      const response = await fetch(`/bet/transaction-params?${params.toString()}`)
      
      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || 'Failed to get transaction params')
      }
      
      const txParams = await response.json()

      // 4. 发送交易
      const signer = await this.contractService.getSigner()
      const tx = await signer.sendTransaction({
        to: txParams.to,
        value: ethers.parseEther(amount),
        data: txParams.data,
        gasLimit: 300000n
      })
      
      // 等待交易确认
      const receipt = await tx.wait()

      // 5. 发送投注记录到后端
      const betRecord = {
        account_address: userAddress,
        amount: amount,
        transaction_hash: receipt?.hash,
        block_number: receipt?.blockNumber,
        block_timestamp: Math.floor(Date.now() / 1000)
      }

      const recordResponse = await axios.post('/bet/record', betRecord)
      
      if (!recordResponse.data.success) {
        throw new Error(recordResponse.data.message || '投注记录保存失败')
      }

      return receipt
    } catch (error) {
      console.error('下注失败:', error)
      throw error
    }
  }
} 