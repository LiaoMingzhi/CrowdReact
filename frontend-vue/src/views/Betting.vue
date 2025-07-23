<template>
  <div class="betting-container">
    <el-card class="betting-card">
      <template #header>
        <div class="card-header">
          <span>Place Your Bet</span>
        </div>
      </template>
      
      <el-form label-position="top" :model="formData" ref="formRef">
        <el-form-item 
          label="Wallet Address" 
          prop="walletAddress"
          :rules="rules.walletAddress"
        >
          <el-input
            v-model="formData.walletAddress"
            :disabled="userStore.isLoggedIn"
            placeholder="Enter your wallet address (0x...)"
          />
        </el-form-item>

        <el-form-item label="Upline Agent">
          <el-select 
            v-model="selectedAgent" 
            class="agent-select"
            placeholder="Select an agent">
            <el-option value="" label="Random Agent" />
            <el-option
              v-for="agent in uplineAgents"
              :key="agent.id"
              :value="agent.name || ''"
              :label="agent.name || ''"
            />
          </el-select>
        </el-form-item>

        <el-form-item 
          label="Custom Agent Address (Optional)"
          :rules="rules.customAgentAddress"
        >
          <el-input
            v-model="customAgentAddress"
            placeholder="Enter specific agent address"
            @blur="validateCustomAgentAddress"
          />
        </el-form-item>

        <el-form-item label="Bet Amount (ETH)">
          <el-input-number
            v-model="betAmount"
            :min="0.01"
            :max="10"
            :step="0.01"
            :precision="2"
            class="bet-amount-input"
          />
        </el-form-item>

        <el-button 
          type="primary" 
          class="place-bet-btn"
          @click="handlePlaceBet"
          :disabled="isSunday"
          :title="isSunday ? 'Sunday is closed, see you on Monday' : ''">
          {{ isSunday ? 'Sunday is closed, see you on Monday' : 'Place Bet' }}
        </el-button>
      </el-form>
    </el-card>

    <!-- Confirmation Dialog -->
    <el-dialog
      v-model="dialogVisible"
      title="Confirm Bet"
      width="30%"
    >
      <span>Are you sure you want to place this bet?</span>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="dialogVisible = false">Cancel</el-button>
          <el-button type="primary" @click="confirmBet">Confirm</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- Login Dialog -->
    <el-dialog v-model="loginDialogVisible" title="Login" width="30%">
      <el-form :model="loginForm" label-width="80px">
        <el-form-item label="Username">
          <el-input v-model="loginForm.username" disabled />
        </el-form-item>
        <el-form-item label="Password">
          <el-input v-model="loginForm.password" type="password" />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="loginDialogVisible = false">Cancel</el-button>
          <el-button type="primary" @click="handleLogin">Login</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- Register Dialog -->
    <el-dialog 
      v-model="registerDialogVisible" 
      title="Register" 
      width="600px"
      :close-on-click-modal="false"
      center
    >
      <el-form 
        :model="registerForm" 
        label-position="top" 
        class="register-form"
      >
        <el-form-item label="Wallet Address">
          <el-input 
            v-model="registerForm.username"
            disabled
          />
        </el-form-item>

        <el-form-item label="Generated Password">
          <el-input 
            v-model="generatedPassword"
            readonly
            :type="showGeneratedPassword ? 'text' : 'password'"
          >
            <template #suffix>
              <el-icon 
                class="password-icon" 
                @click="showGeneratedPassword = !showGeneratedPassword"
              >
                <View v-if="!showGeneratedPassword" />
                <Hide v-else />
              </el-icon>
            </template>
            <template #append>
              <el-button @click="copyGeneratedPassword">
                <el-icon><DocumentCopy /></el-icon>
              </el-button>
            </template>
          </el-input>
          <div class="password-hint">Please save this password carefully</div>
        </el-form-item>

        <el-form-item label="Password">
          <el-input 
            v-model="registerForm.password"
            :type="showPassword ? 'text' : 'password'"
          >
            <template #suffix>
              <el-icon 
                class="password-icon" 
                @click="showPassword = !showPassword"
              >
                <View v-if="!showPassword" />
                <Hide v-else />
              </el-icon>
            </template>
          </el-input>
        </el-form-item>

        <el-form-item label="Confirm Password">
          <el-input 
            v-model="registerForm.confirmPassword"
            :type="showConfirmPassword ? 'text' : 'password'"
          >
            <template #suffix>
              <el-icon 
                class="password-icon" 
                @click="showConfirmPassword = !showConfirmPassword"
              >
                <View v-if="!showConfirmPassword" />
                <Hide v-else />
              </el-icon>
            </template>
          </el-input>
        </el-form-item>
      </el-form>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="registerDialogVisible = false">Cancel</el-button>
          <el-button type="primary" @click="handleRegister">Register</el-button>
        </div>
      </template>
    </el-dialog>

    <!-- Change Password Dialog -->
    <el-dialog 
      v-model="changePasswordDialogVisible" 
      title="Change Password" 
      width="800px"
      :close-on-click-modal="false"
      center
    >
      <el-form 
        :model="changePasswordForm" 
        label-position="top" 
        class="change-password-form"
      >
        <el-form-item label="Current Password">
          <el-input 
            v-model="changePasswordForm.currentPassword"
            :type="showCurrentPassword ? 'text' : 'password'"
          >
            <template #suffix>
              <el-icon 
                class="password-icon" 
                @click="showCurrentPassword = !showCurrentPassword"
              >
                <View v-if="!showCurrentPassword" />
                <Hide v-else />
              </el-icon>
            </template>
          </el-input>
        </el-form-item>

        <el-form-item label="New Password">
          <el-input 
            v-model="changePasswordForm.newPassword"
            :type="showNewPassword ? 'text' : 'password'"
          >
            <template #suffix>
              <el-icon 
                class="password-icon" 
                @click="showNewPassword = !showNewPassword"
              >
                <View v-if="!showNewPassword" />
                <Hide v-else />
              </el-icon>
            </template>
          </el-input>
        </el-form-item>

        <el-form-item label="Confirm New Password">
          <el-input 
            v-model="changePasswordForm.confirmPassword"
            :type="showConfirmNewPassword ? 'text' : 'password'"
          >
            <template #suffix>
              <el-icon 
                class="password-icon" 
                @click="showConfirmNewPassword = !showConfirmNewPassword"
              >
                <View v-if="!showConfirmNewPassword" />
                <Hide v-else />
              </el-icon>
            </template>
          </el-input>
        </el-form-item>
      </el-form>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="changePasswordDialogVisible = false">Cancel</el-button>
          <el-button type="primary" @click="handleChangePassword">Confirm</el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ethers } from 'ethers'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useUserStore } from '@/stores/user'
import type { UplineAgent } from '@/types'
import { generatePasswordFromAddress } from '@/utils/auth'
import contractService from '@/api/contract'
import axios from 'axios'
import { useRouter } from 'vue-router'
import { DocumentCopy, View, Hide } from '@element-plus/icons-vue'
import { useClipboard } from '@vueuse/core'
import { passwordService } from '@/services/passwordService'
import dayjs from 'dayjs'
import utc from 'dayjs/plugin/utc'
import timezone from 'dayjs/plugin/timezone'
//import apiService, { type AgentInfo } from '@/api/backend'
const router = useRouter()
const userStore = useUserStore()
const formRef = ref()
const dialogVisible = ref(false)
const uplineAgents = ref<UplineAgent[]>([])
const selectedAgent = ref('')
const customAgentAddress = ref('')
const betAmount = ref(0.01)

const formData = ref({
  walletAddress: userStore.isLoggedIn ? userStore.walletAddress : '',
  mnemonic: '',
  signature: ''
})

const rules = ref({
  walletAddress: [
    { required: true, message: 'Wallet address is required' },
    { pattern: /^0x[a-fA-F0-9]{40}$/, message: 'Invalid wallet address format' }
  ],
  customAgentAddress: [
    {
      validator: (rule: any, value: string, callback: Function) => {
        if (!value) {
          callback()
          return
        }

        // First check if it's a valid ETH address format
        if (!/^0x[a-fA-F0-9]{40}$/.test(value)) {
          callback(new Error('Please enter a valid ETH address format (0x...)'))
          return
        }

        // Case-insensitive comparison
        const normalizedInput = normalizeAddress(value)
        const isValid = uplineAgents.value.some(agent => 
          normalizeAddress(agent.address) === normalizedInput
        )

        if (!isValid) {
          callback(new Error('This address is not in the Upline Agents list. Please select from the available agents.'))
          return
        }

        callback()
      },
      trigger: ['blur', 'change']
    }
  ]
})

const loginForm = ref({
  username: '',
  password: ''
})

const registerForm = ref({
  username: '',
  password: '',
  confirmPassword: ''
})

const loginDialogVisible = ref(false)
const registerDialogVisible = ref(false)

const { copy } = useClipboard()
const generatedPassword = ref('')
const showGeneratedPassword = ref(false)
const showPassword = ref(false)
const showConfirmPassword = ref(false)

const changePasswordDialogVisible = ref(false)
const showCurrentPassword = ref(false)
const showNewPassword = ref(false)
const showConfirmNewPassword = ref(false)

const copyGeneratedPassword = async () => {
  await copy(generatedPassword.value)
  ElMessage.success('Password copied to clipboard')
}

const fetchUplineAgents = async () => {
  try {
    // Only fetch agents if user is logged in and it's not Sunday
    if (userStore.isLoggedIn && !isSunday.value) {
      console.log('Fetching upline agents...')
      const response = await axios.get('/api/bet/agents')
      console.log('Raw API response:', response)
      
      if (response.data.status === 'success') {
        uplineAgents.value = response.data.data.map((agent: any) => ({
          id: agent.id.toString(),
          name: agent.userAddress,
          address: agent.userAddress
        }))
        console.log('Processed upline agents:', uplineAgents.value)
      } else {
        console.warn('Invalid response format:', response.data)
        ElMessage.warning('Invalid agent data format')
      }
    }
  } catch (error) {
    console.error('Error details:', error)
    ElMessage.error('Failed to fetch upline agents')
  }
}

const handlePlaceBet = async () => {
  if (!formRef.value) return
  
  await formRef.value.validate(async (valid: boolean) => {
    if (valid) {
      dialogVisible.value = true
    }
  })
}

const confirmBet = async () => {
  try {
    // 继续下注流程
    await placeBet();
  } catch (error) {
    console.error('Bet failed:', error);
    ElMessage.error(`Bet failed: ${error instanceof Error ? error.message : 'unknown error'}`);
    dialogVisible.value = false;
  }
}

// 定义错误类型接口
interface EthereumError {
  code: string;
  message: string;
}

// 类型卫函数
function isEthereumError(error: unknown): error is EthereumError {
  return (
    typeof error === 'object' && 
    error !== null && 
    'code' in error &&
    typeof (error as EthereumError).code === 'string'
  );
}

// 在 script 部分添加随机选择代理的方法
const getRandomAgent = () => {
  if (customAgentAddress.value) {
    if (validateCustomAgentAddress()) {
      return customAgentAddress.value
    }
    // If validation fails, clear the custom address
    customAgentAddress.value = ''
  }
  
  if (selectedAgent.value) {
    return selectedAgent.value
  }
  
  if (!uplineAgents.value || uplineAgents.value.length === 0) return ''
  const randomIndex = Math.floor(Math.random() * uplineAgents.value.length)
  return uplineAgents.value[randomIndex].address
}

async function placeBet() {
  try {
    // 1. 检查是否安装了 MetaMask
    const ethereum = window.ethereum;
    if (!ethereum) {
      await ElMessageBox.alert(
        `To use this application, please install MetaMask wallet first:
        <br><br>
        1. Visit <a href="https://metamask.io/download/" target="_blank">MetaMask official website</a>
        <br>
        2. Click "Install MetaMask for Chrome" (or other browsers)
        <br>
        3. Follow the instructions to complete the installation
        <br>
        4. Refresh this page after installation
        `,
        'Install MetaMask',
        {
          confirmButtonText: 'I know',
          dangerouslyUseHTMLString: true,
          type: 'warning'
        }
      );
      return;
    }

    // 2. 请求用户连接钱包
    const accounts = await ethereum.request({ 
      method: 'eth_requestAccounts' 
    });
    const currentAccount = accounts[0];
    console.log('Current betting account:', currentAccount);

    // 验证当前账号
    if (!currentAccount) {
      throw new Error('could not get current account');
    }

    // 3. 如果用户未登录，检查用户是否存在
    if (!userStore.isLoggedIn) {
      try {
        // 检查用户是否存在
        const checkUserResponse = await axios.get(`/api/auth/check-user/${currentAccount}`);
        const userExists = checkUserResponse.data.exists;

        if (userExists) {
          // 用户存在，显示登录对话框
          loginForm.value = {
            username: currentAccount,
            password: ''
          };
          loginDialogVisible.value = true;
          return;
        } else {
          // 用户不存在，显示注册对话框
          const genPassword = generatePasswordFromAddress(currentAccount)
          generatedPassword.value = genPassword
          registerForm.value = {
            username: currentAccount, // 保持原始格式，不转换为小写
            password: genPassword,
            confirmPassword: genPassword
          }
          registerDialogVisible.value = true
          return
        }
      } catch (error) {
        console.error('Authentication check failed:', error);
        ElMessage.error('Failed to check user status');
        return;
      }
    } else {
      // 如果登录的账户与连接的钱包账号不一致，则提示用户切换钱包账号
      if (userStore.walletAddress !== currentAccount) {
        ElMessage.warning('Please connect with the correct wallet address')
        return
      }
    }

    // 显示交易处理中的加载状态
    const loading = ElMessage({
      message: '交易处理中...',
      type: 'info',
      duration: 0
    });

    // 获取合约实例
    const contract = await contractService.getContract();
    if (!contract) {
      throw new Error('Failed to get contract instance');
    }

    // 发送交易
    const tx = await contract.placeBet({
      value: ethers.parseEther(betAmount.value.toString())
    });
    
    // 显示交易已发送
    loading.close();
    ElMessage({
      message: 'Transaction sent, waiting for confirmation...',
      type: 'info',
      duration: 0
    });

    // 等待交易确认
    const receipt = await tx.wait();

    // 交易成功
    ElMessage.closeAll();
    ElMessage({
      message: `Betting successful!\nTransaction hash: ${receipt.hash}`,
      type: 'success',
      duration: 5000
    });
    
    // 关闭确认对话框
    dialogVisible.value = false;

    // 记录交易到后端
    try {
      // 如果用户没有选择代理，随机选择一个
      const agentAddress = selectedAgent.value || getRandomAgent();
      
      // 创建一个普通对象，避免响应式引用
      const betData = {
        account_address: currentAccount,
        amount: betAmount.value.toString(),
        transaction_hash: receipt.hash,
        block_number: Number(receipt.blockNumber), // 转换为数字
        block_timestamp: Math.floor(Date.now() / 1000),
        agent_address: agentAddress
      };

      console.log('Sending bet data:', betData); // 调试日志
      // 发送投注记录到后端
      const response = await axios.post('/api/bet/place_bet', betData);

      if (!response.data.success) {
        console.warn('Failed to record bet:', response.data.error);
      } else {
        console.log('Bet recorded successfully');
      }
    } catch (error) {
      console.error('Failed to record bet:', error);
    }

    // 可选：重置表单
    betAmount.value = 0.01;

  } catch (err: unknown) {
    // 关闭所有消息
    ElMessage.closeAll();
    
    console.error('Betting failed:', err);
    
    // 使用类型守卫检查错误类型
    if (isEthereumError(err)) {
      if (err.code === 'ACTION_REJECTED') {
        ElMessage.error('User canceled the transaction');
      } else if (err.code === 'INSUFFICIENT_FUNDS') {
        ElMessage.error('Insufficient funds');
      } else {
        ElMessage.error(err.message || 'Betting failed');
      }
    } else if (err instanceof Error) {
      ElMessage.error(err.message);
    } else {
      ElMessage.error('Betting failed');
    }
  }
}

// 添加登录和注册处理函数
const handleLogin = async () => {
  try {
    // 使用之前保存的密码
    const password = tempPassword.value || loginForm.value.password.trim()
    console.log('Using password for login:', password)
    
    await userStore.login({
      username: loginForm.value.username.toLowerCase(),
      password: password
    })
    loginDialogVisible.value = false
    await placeBet()
    // 添加页面刷新
    window.location.reload()
  } catch (error) {
    console.error('Login failed:', error)
    
    // 定义类型保护函数
    const isAxiosError = (error: unknown): error is { response?: { status: number } } => {
      return typeof error === 'object' && 
             error !== null && 
             'response' in error && 
             typeof (error as any).response?.status === 'number'
    }
    
    // 使用类型保护进行错误处理
    if (isAxiosError(error) && error.response?.status === 401) {
      ElMessage.error('Invalid username or password')
    } else {
      ElMessage.error('Login failed. Please try again later')
    }
  }
}

const handleRegister = async () => {
  if (!registerForm.value.password) {
    ElMessage.error('Please enter your password')
    return
  }
  
  const loadingRegisterMsg = ElMessage({
    message: 'Registering...',
    type: 'info',
    duration: 0
  })

  try {
    const username = registerForm.value.username.toLowerCase()
    // 使用用户输入的密码，而不是生成的密码
    const password = registerForm.value.password.trim()
    
    // 保存用户输入的密码
    passwordService.savePassword(username, password)
    
    await userStore.register(username, password)
    await userStore.login({
      username: username,
      password: password
    })
    
    registerDialogVisible.value = false
    ElMessage.success('Registration and login successful')
  } catch (error) {
    console.error('Registration error:', error)
    ElMessage.error(error instanceof Error ? error.message : 'Registration failed')
  } finally {
    loadingRegisterMsg.close()
  }
  window.location.reload()
}

// 在组件挂载时执行
onMounted(async () => {
  // 获取上级代理
  fetchUplineAgents()
  
  // 如果用户已登录，自动连接钱包
  if (userStore.isLoggedIn) {
    try {
      const ethereum = window.ethereum
      if (ethereum) {
        const accounts = await ethereum.request({ 
          method: 'eth_requestAccounts' 
        })
        const walletAddress = accounts[0]
        
        if (walletAddress !== userStore.walletAddress) {
          ElMessage.warning('Please connect with the correct wallet address')
          userStore.logout() // 使用 store 的 logout 方法
          router.push('/') // 重定向到首页
        }
      }
    } catch (error) {
      console.error('Failed to connect wallet:', error)
      ElMessage.error('Failed to connect wallet')
    }
  }
})

interface ChangePasswordForm {
  currentPassword: string
  newPassword: string
  confirmPassword: string
}

const changePasswordForm = ref<ChangePasswordForm>({
  currentPassword: '',
  newPassword: '',
  confirmPassword: ''
})

const handleChangePassword = async () => {
  try {
    // 验证新密码
    if (changePasswordForm.value.newPassword !== changePasswordForm.value.confirmPassword) {
      ElMessage.error('New passwords do not match')
      return
    }

    // 调用 store 中的修改密码方法
    await userStore.changePassword(
      changePasswordForm.value.currentPassword,
      changePasswordForm.value.newPassword
    )

    ElMessage.success('Password changed successfully')
    changePasswordDialogVisible.value = false
    
    // 清空表单
    changePasswordForm.value = {
      currentPassword: '',
      newPassword: '',
      confirmPassword: ''
    }
  } catch (error) {
    console.error('Failed to change password:', error)
    ElMessage.error(error instanceof Error ? error.message : 'Failed to change password')
  }
}

// 添加一个临时存储密码的变量
const tempPassword = ref('')

// Add a helper function to normalize ethereum addresses
const normalizeAddress = (address: string): string => {
  return address.toLowerCase()
}

// Update validation function with case-insensitive comparison
const validateCustomAgentAddress = () => {
  if (!customAgentAddress.value) return true

  if (!/^0x[a-fA-F0-9]{40}$/.test(customAgentAddress.value)) {
    ElMessage.warning('Please enter a valid ETH address format (0x...)')
    return false
  }

  const normalizedInput = normalizeAddress(customAgentAddress.value)
  const isValid = uplineAgents.value.some(agent => 
    normalizeAddress(agent.address) === normalizedInput
  )

  if (!isValid) {
    ElMessage.warning('This address is not in the Upline Agents list. Please select from the available agents.')
    customAgentAddress.value = '' // Clear invalid input
    return false
  }

  return true
}

// Add these dayjs plugins
dayjs.extend(utc)
dayjs.extend(timezone)

// Add this computed property
const isSunday = computed(() => {
  const tz = 'Europe/London' // Using Singapore timezone
  return dayjs().tz(tz).day() === 0 // 0 represents Sunday
  // return false;
})
</script>

<style scoped>
.betting-container {
  padding: 20px;
  max-width: 800px;
  margin: 0 auto;
}

.betting-card {
  background-color: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header span {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.agent-select {
  width: 100%;
}

.bet-amount-input {
  width: 100%;
}

.place-bet-btn {
  width: 100%;
  margin-top: 20px;
  height: 40px;
  font-size: 16px;
}

:deep(.el-form-item__label) {
  font-weight: 500;
  color: #606266;
}

:deep(.el-input-number) {
  width: 100%;
}

:deep(.el-form-item) {
  margin-bottom: 20px;
}

.password-hint {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

:deep(.el-input-group__append) {
  padding: 0;
}

:deep(.el-input-group__append .el-button) {
  border: none;
  padding: 8px 15px;
}

.register-form {
  padding: 20px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 0 20px 20px;
}

:deep(.password-icon) {
  cursor: pointer;
  color: #909399;
}

:deep(.password-icon:hover) {
  color: #409EFF;
}

:deep(.el-input-group__append) {
  padding: 0;
}

:deep(.el-input-group__append .el-button) {
  border: none;
  padding: 8px 15px;
}

:deep(.el-dialog__header) {
  margin-right: 0;
  padding: 20px;
  border-bottom: 1px solid #dcdfe6;
}

:deep(.el-dialog__title) {
  font-size: 18px;
  font-weight: 600;
}

:deep(.el-dialog__body) {
  padding: 0;
}
</style> 