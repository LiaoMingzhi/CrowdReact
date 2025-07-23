<template>
  <div class="common-layout">
    <el-container>
      <el-header>
        <div class="header-content">
          <div class="nav-menu">
            <el-menu 
              mode="horizontal" 
              :router="true"
              :default-active="activeIndex"
            >
              <el-menu-item index="/">
                <router-link to="/">Home</router-link>
              </el-menu-item>
              <el-menu-item index="/agent">
                <router-link to="/agent">Agent</router-link>
              </el-menu-item>
              <el-menu-item index="/bet-history">
                <router-link to="/bet-history">Bet History</router-link>
              </el-menu-item>
              <el-menu-item index="/betting">
                <router-link to="/betting">Betting</router-link>
              </el-menu-item>
              <el-menu-item index="/commission">
                <router-link to="/commission">Commission</router-link>
              </el-menu-item>
            </el-menu>
          </div>
          <div class="user-info">
            <template v-if="isLoggedIn">
              <el-dropdown>
                <span class="welcome-text">
                  {{ formatAddress(username) }} <el-icon><ArrowDown /></el-icon>
                </span>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item @click="showChangePassword">Change Password</el-dropdown-item>
                    <el-dropdown-item @click="handleLogout">Logout</el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </template>
            <template v-else>
              <el-button type="primary" @click="handleLoginClick">Login</el-button>
            </template>
          </div>
        </div>
      </el-header>
      <el-main>
        <router-view></router-view>
      </el-main>
    </el-container>

    <!-- Login Dialog -->
    <el-dialog 
      v-model="loginDialogVisible" 
      title="Login" 
      width="600px"
      :close-on-click-modal="false"
      center
    >
      <el-form 
        :model="loginForm" 
        label-position="top" 
        class="login-form"
      >
        <el-form-item 
          label="Wallet Address" 
          prop="username"
        >
          <el-input 
            v-model="loginForm.username"
            placeholder="Enter your wallet address (0x...)"
            prefix-icon="Wallet"
          />
        </el-form-item>
        
        <el-form-item 
          label="Password" 
          prop="password"
        >
          <el-input 
            v-model="loginForm.password" 
            type="password"
            placeholder="Enter your password"
            prefix-icon="Lock"
            show-password
          />
        </el-form-item>

        <div class="info-text" v-if="isFirstLogin">
          <el-alert
            type="warning"
            :closable="false"
            show-icon
          >
            Please change your password for security reasons.
          </el-alert>
        </div>
      </el-form>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="loginDialogVisible = false">Cancel</el-button>
          <el-button 
            type="primary" 
            @click="handleLogin" 
            :loading="loading"
          >
            Login
          </el-button>
        </div>
      </template>
    </el-dialog>

    <!-- Change Password Dialog -->
    <el-dialog 
      v-model="changePasswordVisible" 
      title="Change Password" 
      width="600px"
      :close-on-click-modal="false"
      center
    >
      <el-form 
        :model="passwordForm" 
        label-position="top" 
        class="change-password-form"
      >
        <el-form-item label="Current Password">
          <el-input 
            v-model="passwordForm.oldPassword" 
            type="password" 
            show-password
          />
        </el-form-item>
        <el-form-item label="New Password">
          <el-input 
            v-model="passwordForm.newPassword" 
            type="password" 
            show-password
          />
        </el-form-item>
        <el-form-item label="Confirm New Password">
          <el-input 
            v-model="passwordForm.confirmPassword" 
            type="password" 
            show-password
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="changePasswordVisible = false">Cancel</el-button>
          <el-button type="primary" @click="handleChangePassword" :loading="loading">
            Confirm
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { ArrowDown, Wallet, Lock } from '@element-plus/icons-vue'
import { apiService } from '@/api/backend'
import { useUserStore } from '@/stores/user'
import { passwordService } from '@/services/passwordService'

const router = useRouter()
const route = useRoute()
const userStore = useUserStore()

const activeIndex = computed(() => route.path)
const isLoggedIn = computed(() => userStore.isLoggedIn)
const username = computed(() => userStore.walletAddress)

const loginDialogVisible = ref(false)
const changePasswordVisible = ref(false)
const loading = ref(false)
const isFirstLogin = ref(false)

const loginForm = ref({
  username: '',
  password: ''
})

const passwordForm = ref({
  oldPassword: '',
  newPassword: '',
  confirmPassword: ''
})

onMounted(() => {
  checkLoginStatus()
})

const formatAddress = (address: string) => {
  if (!address) return ''
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

const checkLoginStatus = () => {
  if (!userStore.isLoggedIn) {
    handleLogout()
  }
}

const handleLoginClick = async () => {
  try {
    // 获取钱包地址
    const accounts = await window.ethereum.request({ 
      method: 'eth_requestAccounts' 
    })
    
    loginForm.value = {
      username: '',  // 不预填用户名
      password: ''
    }
    
    loginDialogVisible.value = true
  } catch (error) {
    console.error('Login initialization error:', error)
    ElMessage.error('Failed to initialize login')
  }
}

interface ApiError {
  response?: {
    status: number;
    data?: any;
  };
}

const handleLogin = async () => {
  if (!loginForm.value.username) {
    ElMessage.error('Please enter your username')
    return
  }

  try {
    loading.value = true
    const username = loginForm.value.username.toLowerCase()
    let password = loginForm.value.password.trim()
    
    // 只有当用户没有输入密码时，才使用保存的密码
    if (!password) {
      password = passwordService.getPassword(username) || ''
      if (!password) {
        ElMessage.error('Please enter your password')
        return
      }
    }
    
    await userStore.login({
      username: username,
      password: password.trim()
    })
    
    loginDialogVisible.value = false
    ElMessage.success('Login successful')
    // 在登录成功后存储token
    localStorage.setItem('token', `Bearer ${userStore.token}`)
    // 检查是否是首次登录
    if (isFirstLogin.value) {
      ElMessage.warning('Please change your password for security')
    }
    // 添加页面刷新
    window.location.reload()
  } catch (error: unknown) {
    console.error('Login error:', error)
    if (
      error && 
      typeof error === 'object' && 
      (error as ApiError).response?.status === 401
    ) {
      ElMessage.error('Invalid username or password')
    } else {
      ElMessage.error('Login failed. Please try again later')
    }
  } finally {
    loading.value = false
    loginForm.value.password = '' // 清空密码
  }
}

const handleLogout = () => {
  localStorage.removeItem('token')
  userStore.logout()
  userStore.$reset() // 重置store状态
  router.push('/')
}

const showChangePassword = () => {
  changePasswordVisible.value = true
}

const handleChangePassword = async () => {
  if (passwordForm.value.newPassword !== passwordForm.value.confirmPassword) {
    ElMessage.error('New passwords do not match')
    return
  }
  
  try {
    loading.value = true
    console.log('Sending change password request with:', {
      currentPassword: passwordForm.value.oldPassword,
      newPassword: passwordForm.value.newPassword
    })

    const token = localStorage.getItem('token')
    if (token) {
      console.log('Token exists, length:', token.length)
    } else {
      console.warn('No token found')
    }

    await userStore.changePassword(
      passwordForm.value.oldPassword,
      passwordForm.value.newPassword
    )
    
    ElMessage.success('Password changed successfully')
    changePasswordVisible.value = false
    
    // 清空表单
    passwordForm.value = {
      oldPassword: '',
      newPassword: '',
      confirmPassword: ''
    }
    
  } catch (error) {
    console.error('Change password error:', error)
    // 增加更详细的错误处理
    if (error && typeof error === 'object' && 'response' in error) {
        const apiError = error as ApiError
        if (apiError.response?.status === 401) {
            const errorMessage = apiError.response.data?.error || ''
            if (errorMessage.includes('Invalid token')) {
                ElMessage.error('会话已过期，请重新登录')
                userStore.logout()
                router.push('/login')
            } else {
                ElMessage.error('当前密码不正确')
            }
        } else if (apiError.response?.status === 500) {
            ElMessage.error('服务器内部错误，请稍后重试')
            console.error('Server error details:', apiError.response.data)
        } else {
            ElMessage.error('密码修改失败，请重试')
        }
    } else {
        ElMessage.error('密码修改失败，请重试')
    }
  } finally {
    loading.value = false
  }
}

// 添加路由变化监听
watch(
  () => route.path,
  (newPath) => {
    console.log('Route changed to:', newPath)
    const requiresAuth = router.currentRoute.value.meta.requiresAuth
    if (requiresAuth && !userStore.isLoggedIn) {
      ElMessage.warning('Please login first')
    }
  }
)

// 添加菜单选择处理函数
const handleSelect = (key: string) => {
  console.log('Selected menu:', key)
  router.push(key)
}
</script>

<style scoped>
.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 100%;
}

.nav-menu {
  flex: 1;
}

.user-info {
  margin-left: 20px;
}

:deep(.el-menu--horizontal) {
  border-bottom: none;
}

:deep(.el-menu-item) {
  font-size: 16px;
}

:deep(.el-menu--horizontal .el-menu-item.is-active) {
  border-bottom: 2px solid var(--el-menu-active-color);
  color: var(--el-menu-active-color);
}

/* 添加路由链接样式 */
:deep(.el-menu-item a) {
  text-decoration: none;
  color: inherit;
}

:deep(.el-menu-item.is-active a) {
  color: var(--el-menu-active-color);
}

/* 确保链接填满菜单项 */
:deep(.el-menu-item) {
  padding: 0;
}

:deep(.el-menu-item a) {
  display: block;
  padding: 0 20px;
  line-height: 60px;
  height: 100%;
}

.login-form {
  padding: 20px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 0 20px 20px;
}

:deep(.el-form-item__label) {
  font-weight: 500;
  color: #606266;
}

:deep(.el-input__wrapper) {
  box-shadow: 0 0 0 1px #dcdfe6;
}

:deep(.el-input__wrapper:hover) {
  box-shadow: 0 0 0 1px #409eff;
}

.info-text {
  margin-top: 16px;
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
