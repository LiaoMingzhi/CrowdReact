<template>
  <div class="page-container">
    <template v-if="userStore.isLoggedIn">
      <div class="bet-history-container">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>Bet History</span>
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

          <!-- 投注历史表格 -->
          <el-table :data="betHistory" style="width: 100%" v-loading="loading">
            <!-- 时间列 -->
            <el-table-column prop="time" label="Time" width="180">
              <template #default="scope">
                {{ formatTime(scope.row.time) }}
              </template>
            </el-table-column>

            <!-- 金额列 -->
            <el-table-column prop="amount" label="Amount" width="120">
              <template #default="scope">
                {{ scope.row.amount }} ETH
              </template>
            </el-table-column>

            <!-- 幸运数字列 -->
            <el-table-column prop="luck_number" label="Lucky Numbers">
              <template #default="scope">
                <el-tooltip
                  v-for="(number, index) in scope.row.luck_number"
                  :key="number"
                  :content="number"
                  placement="top"
                  effect="light"
                >
                  <el-tag
                    :type="getTagType(index)"
                    class="number-tag"
                  >
                    {{ formatLuckNumber(number) }}
                  </el-tag>
                </el-tooltip>
              </template>
            </el-table-column>

            <!-- 交易哈希列 -->
            <el-table-column prop="transaction_hash" label="Transaction" width="160">
              <template #default="scope">
                <el-link 
                  type="primary" 
                  :href="`https://etherscan.io/tx/${scope.row.transaction_hash}`"
                  target="_blank"
                >
                  {{ formatTxHash(scope.row.transaction_hash) }}
                </el-link>
              </template>
            </el-table-column>
          </el-table>

          <!-- 添加分页组件 -->
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
import { useRouter } from 'vue-router'
import { apiService } from '@/api/backend'
import { ElMessage } from 'element-plus'
import { useUserStore } from '@/stores/user'
import type {  BetHistoryParams } from '@/api/backend'
import dayjs from 'dayjs'
const userStore = useUserStore()
const router = useRouter()
const betHistory = ref<BetHistoryResponse['bet_history']>([])
const userAddress = ref('')
const currentPage = ref(1)
const pageSize = ref(10)
const total = ref(0)
const loading = ref(false)
const total_amount = ref('0')

// 格式化地址显示
const formatAddress = (address: string) => {
  if (!address) return ''
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

// 格式化时间显示
const formatTime = (timestamp: string) => {
  return dayjs(timestamp).format('YYYY-MM-DD HH:mm:ss')
}

// 格式化交易哈希显示
const formatTxHash = (hash: string) => {
  return `${hash.slice(0, 6)}...${hash.slice(-4)}`
}

// 格式化幸运数字显示（从UUID中提取数字）
const formatLuckNumber = (uuid: string) => {
  // 从UUID中提取前两位数字
  return uuid.slice(0, 2)
}

// 获取标签类型
const getTagType = (index: number): 'success' | 'warning' | 'danger' | 'info' => {
  const types = ['success', 'warning', 'danger'] as const
  return types[index] || 'info'
}

const handleLogin = () => {
  router.push('/')
}

// First, make sure your types are properly defined (add these if they don't exist)
interface BetHistoryResponse {
  user_address: string;
  bet_history: Array<{
    time: string;
    amount: number;
    luck_number: string[];
    transaction_hash: string;
  }>;
  total?: number;
  total_amount?: string;
}

// 处理页码变化
const handleCurrentChange = (page: number) => {
  currentPage.value = page
  fetchBetHistory()
}

// 处理每页条数变化
const handleSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1 // 重置到第一页
  fetchBetHistory()
}

async function fetchBetHistory(params?: BetHistoryParams) {
  if (!userStore.isLoggedIn) {
    ElMessage.warning('Please login first')
    return
  }

  loading.value = true
  try {
    const response = await apiService.getBetHistory({
      page: currentPage.value,
      limit: pageSize.value,
      ...params
    })
    
    // Type assertion to help TypeScript understand the response structure
    const data = response.data as unknown as BetHistoryResponse
    console.log("data: ", data)
    if (data && data.user_address && Array.isArray(data.bet_history)) {
      userAddress.value = data.user_address
      betHistory.value = data.bet_history
      total.value = data.total ?? 0
      total_amount.value = data.total_amount ?? '0'
    } else {
      console.error('Invalid data structure:', data)
      throw new Error('Invalid response format')
    }
  } catch (error) {
    console.error('Failed to fetch betting history:', error)
    ElMessage.error('Failed to fetch betting history')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  if (userStore.isLoggedIn) {
    fetchBetHistory()
  }
})
</script>


<style scoped>
.bet-history-container {
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

.number-tag {
  margin-right: 5px;
}

:deep(.el-table) {
  margin-top: 20px;
}

:deep(.el-tag) {
  margin: 2px;
}


/* 响应式布局 */
@media screen and (max-width: 768px) {
  :deep(.el-pagination) {
    padding: 10px 5px;
  }
}

/* 添加加载状态的样式 */
:deep(.el-loading-mask) {
  background-color: rgba(255, 255, 255, 0.8);
}

/* 添加悬浮提示相关样式 */
:deep(.el-tooltip__trigger) {
  display: inline-block;
  margin: 2px;
}

.number-tag {
  cursor: pointer; /* 添加指针样式表明可互动 */
}



.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: center;
}

:deep(.el-pagination) {
  padding: 15px;

  .el-pagination__sizes {
    width: 125px;

    .el-select {
      width: 125px;
      .el-select__wrapper {
        width: 125px;

        .el-select__selected-item,
        .el-select__placeholder {
          width: 125px;
          margin-top: 15px;
          align-items: center;
          height: 32px;
        }

        .el-select__suffix {
          margin-top: 5px;
          .el-icon {
            margin-left: 90px;
          }
        }
      }
    }
    
  }
}



.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;

  .info-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    white-space: nowrap;
  }

  .address,.amount {
    font-family: monospace;
    color: #409EFF;
    margin-left: 8px;
  }
}



</style>