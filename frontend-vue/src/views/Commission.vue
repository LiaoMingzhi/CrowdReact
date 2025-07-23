<template>
  <div class="page-container">
    <template v-if="userStore.isLoggedIn">
      <div class="commission-container">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>Commission History</span>
            </div>
          </template>

          <!-- 用户地址和总金额显示 -->
          <div class="user-info">
            <div class="info-row">
              <div class="info-item">
                <span class="label">ETH Account Address:</span>
                <span class="address">{{ formatAddress(userAddress) }}</span>
              </div>
              <div class="info-item">
                <span class="label">Total Amount:</span>
                <span class="amount">{{ total_amount }} ETH</span>
              </div>
            </div>
          </div>

          <!-- 佣金历史表格 -->
          <el-table :data="commissionDetails" style="width: 100%" v-loading="loading">
            <el-table-column prop="time" label="Time" width="180">
              <template #default="scope">
                {{ formatTime(scope.row.time) }}
              </template>
            </el-table-column>
            
            <el-table-column prop="commission" label="Commission" width="150">
              <template #default="scope">
                {{ parseFloat(scope.row.commission).toFixed(4) }} ETH
              </template>
            </el-table-column>

            <el-table-column prop="from_address" label="From Address" width="220">
              <template #default="scope">
                <el-tooltip :content="scope.row.from_address" placement="top">
                  <span>{{ formatAddress(scope.row.from_address) }}</span>
                </el-tooltip>
              </template>
            </el-table-column>

            <el-table-column prop="transaction_hash" label="Transaction">
              <template #default="scope">
                <el-link 
                  type="primary" 
                  :href="`${EXPLORER_URL}/tx/${scope.row.transaction_hash}`"
                  target="_blank"
                >
                  {{ formatAddress(scope.row.transaction_hash) }}
                </el-link>
              </template>
            </el-table-column>
          </el-table>

          <!-- 分页组件 -->
          <div class="pagination-container">
            <el-pagination
              v-model:current-page="currentPage"
              v-model:page-size="pageSize"
              :page-sizes="[10, 20, 50, 100]"
              :total="total"
              :background="true"
              layout="total, sizes, prev, pager, next, jumper"
              @size-change="handleSizeChange"
              @current-change="handleCurrentChange"
            />
          </div>
        </el-card>
      </div>
    </template>
    <template v-else>
      <el-empty description="Please login first">
        <el-button type="primary" @click="handleLogin">Login</el-button>
      </el-empty>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { apiService } from '@/api/backend'
import { useUserStore } from '@/stores/user'
import { useRouter } from 'vue-router'
import type { CommissionDetail, CommissionDetailParams, CommissionDetailResponse } from '@/api/backend'
import dayjs from 'dayjs'

// 添加环境变量
const EXPLORER_URL = import.meta.env.VITE_EXPLORER_URL || 'https://sepolia.etherscan.io'

const userStore = useUserStore()
const router = useRouter()
const commissionDetails = ref<CommissionDetail[]>([])
const userAddress = ref('')
const currentPage = ref(1)
const pageSize = ref(10)
const total = ref(0)
const total_amount = ref('0')
const loading = ref(false)

// 格式化地址显示
const formatAddress = (address: string) => {
  if (!address) return ''
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

// 格式化时间显示
const formatTime = (timestamp: string) => {
  return dayjs(timestamp).format('YYYY-MM-DD HH:mm:ss')
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
  fetchCommissionDetails()
}

const handleCurrentChange = (page: number) => {
  currentPage.value = page
  fetchCommissionDetails()
}

const handleLogin = () => {
  router.push('/')
}

async function fetchCommissionDetails(params?: CommissionDetailParams) {
  if (!userStore.isLoggedIn) {
    ElMessage.warning('Please login first')
    return
  }

  loading.value = true
  try {
    const response = await apiService.getCommissionDetails({
      page: currentPage.value,
      limit: pageSize.value,
      ...params
    })
    const data = response.data as unknown as CommissionDetailResponse
    console.log("data: ", data)
    if (data && data.user_address && Array.isArray(data.commission_details)) {
      userAddress.value = data.user_address
      commissionDetails.value = data.commission_details
      total.value = data.total ?? 0
      total_amount.value = data.total_amount ?? '0'
    } else {
      console.error('Invalid data structure:', data)
      throw new Error('Invalid response format')
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || 'Network error occurred')
    console.error('Commission details error:', error)
  } finally {
    loading.value = false
  }
}

// 添加计算总金额的函数
// const formatTotalAmount = (details: any[]) => {
//   const total = details.reduce((sum, commission) => sum + Number(commission.commission), 0)
//   return total.toFixed(4) // 保留4位小数
// }

onMounted(() => {
  if (userStore.isLoggedIn) {
    fetchCommissionDetails()
  }
})
</script>

<style scoped>
.commission-container {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.user-info {
  margin-bottom: 20px;
  padding: 10px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.label {
  font-weight: bold;
  margin-right: 10px;
  color: #606266;
}

.address {
  font-family: monospace;
  color: #409EFF;
}

:deep(.el-table) {
  margin-top: 20px;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: center;
}

:deep(.el-pagination) {
  padding: 15px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.info-item {
  display: flex;
  align-items: center;
}

.amount {
  font-family: monospace;
  color: #409EFF;
  margin-left: 8px;
}
</style> 