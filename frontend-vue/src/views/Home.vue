<template>
  <div class="home">
    <!-- 时间显示 -->
    <div class="time-display">
      <h2>{{ currentTime }}</h2>
      <h3>London: {{ currentWeekday }}</h3>
    </div>

    <!-- 奖金池显示 -->
    <el-card class="prize-pool">
      <template #header>
        <div class="card-header">
          <span>Prize Pool</span>
        </div>
      </template>
      <div class="prize-amount">
        {{ Number(prizeExpectation.prize_amount).toFixed(4) }} ETH
      </div>
    </el-card>

    <!-- 奖金等级预期显示 -->
    <el-card class="prize-expectation">
      <template #header>
        <div class="card-header">
          <span><i class="el-icon-trophy"></i> Prize Expectation</span>
        </div>
      </template>
      <div class="prize-expectation-details">
        <!-- 主要奖项 -->
        <div class="prize-section main-prizes">
          <div class="prize-expectation-item prize-first">
            <el-icon><TrophyBase /></el-icon>
            <div class="prize-content">
              <span class="prize-expectation-label">First Prize</span>
              <span class="prize-expectation-value">{{ Number(prizeExpectation.first_prize).toFixed(5) }} ETH</span>
            </div>
          </div>
          <div class="prize-expectation-item prize-second">
            <el-icon><Medal /></el-icon>
            <div class="prize-content">
              <span class="prize-expectation-label">Second Prize</span>
              <span class="prize-expectation-value">{{ Number(prizeExpectation.second_prize).toFixed(5) }} ETH</span>
            </div>
          </div>
          <div class="prize-expectation-item prize-third">
            <el-icon><Medal /></el-icon>
            <div class="prize-content">
              <span class="prize-expectation-label">Third Prize</span>
              <span class="prize-expectation-value">{{ Number(prizeExpectation.third_prize).toFixed(5) }} ETH</span>
            </div>
          </div>
        </div>
        
        <!-- 代理奖项 -->
        <div class="prize-section agent-prizes">
          <div class="section-title">Agent Rewards</div>
          <div class="prize-expectation-item agent-level-one">
            <el-icon><Star /></el-icon>
            <div class="prize-content">
              <span class="prize-expectation-label">Level One Agent</span>
              <span class="prize-expectation-value">{{ Number(prizeExpectation.level_one_agent_prize).toFixed(5) }} ETH</span>
            </div>
          </div>
          <div class="prize-expectation-item agent-level-two">
            <el-icon><Star /></el-icon>
            <div class="prize-content">
              <span class="prize-expectation-label">Level Two Agent</span>
              <span class="prize-expectation-value">{{ Number(prizeExpectation.level_two_agent_prize).toFixed(5) }} ETH</span>
            </div>
          </div>
          <div class="prize-expectation-item agent-level-common">
            <el-icon><Star /></el-icon>
            <div class="prize-content">
              <span class="prize-expectation-label">Level Common Agent</span>
              <span class="prize-expectation-value">{{ Number(prizeExpectation.common_agent_prize).toFixed(5) }} ETH</span>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 竞争信息显示 -->
    <el-card v-if="isLoggedIn" class="competition-info">
      <template #header>
        <div class="card-header">
          <span>Today's Competition</span>
        </div>
      </template>
      <div class="competition-details" v-if="homeInfo.competition">
        <div class="topic">{{ homeInfo.competition.topic }}</div>
        <div class="competition-count">
          Total Competitions: {{ homeInfo.competition.competition_count }}
        </div>
        <div class="rank" v-if="homeInfo.competition.rank > 0">
          Rank: {{ homeInfo.competition.rank }}
        </div>
        <div class="rank" v-else>
          Not Ranked
        </div>
      </div>
    </el-card>

    <!-- 中奖信息显示 -->
    <el-card v-if="isLoggedIn" class="lottery-info">
      <template #header>
        <div class="card-header">
          <span>Latest Lottery Results</span>
        </div>
      </template>
      <el-table :data="homeInfo.lottery" style="width: 100%">
        <el-table-column prop="prize_grade" label="Prize" width="120">
          <template #default="{ row }">
            <el-tag :type="getPrizeTagType(row.prize_grade)">
              {{ formatPrizeGrade(row.prize_grade) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="user_address" label="Winner">
          <template #default="{ row }">
            <el-tooltip
              class="box-item"
              effect="dark"
              :content="row.user_address"
              placement="top"
            >
              <span>{{ formatAddress(row.user_address) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column prop="luck_number" label="Luck Number" width="120">
          <template #default="{ row }">
            <el-tooltip
              class="box-item"
              effect="dark"
              :content="row.luck_number"
              placement="top"
            >
              <span>{{ formatLuckNumber(row.luck_number) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column prop="prize_amount" label="Amount" width="120">
          <template #default="{ row }">
            {{ Number(row.prize_amount).toFixed(5) }} ETH
          </template>
        </el-table-column>
        <el-table-column prop="time" label="Time" width="180">
          <template #default="{ row }">
            {{ formatTime(row.time) }}
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useUserStore } from '@/stores/user'
import { apiService } from '@/api/backend'
import type { HomeInfoResponse, PrizePoolPrizeExpectionResponse } from '@/api/backend'
import dayjs from 'dayjs'
import { Star, Medal, TrophyBase } from '@element-plus/icons-vue'
import utc from 'dayjs/plugin/utc'
import timezone from 'dayjs/plugin/timezone'

const userStore = useUserStore()
const isLoggedIn = computed(() => userStore.isLoggedIn)

const prizeExpectation = ref<PrizePoolPrizeExpectionResponse>({
  prize_amount: '0.000',
  first_prize: '0.00000',
  second_prize: '0.00000',
  third_prize: '0.00000',
  level_one_agent_prize: '0.00000',
  level_two_agent_prize: '0.00000',
  common_agent_prize: '0.00000'
})
const currentTime = ref('')
const currentWeekday = ref('')
const homeInfo = ref<HomeInfoResponse>({
  competition: {
    user_address: '',
    topic: '',
    competition_count: 0,
    rank: 0
  },
  lottery: []
})

let timer: ReturnType<typeof setInterval> | null = null

// 初始化 dayjs 插件
dayjs.extend(utc)
dayjs.extend(timezone)

// 更新时间显示
const updateTime = () => {
  const now = dayjs().tz('Europe/London')
  currentTime.value = now.format('YYYY-MM-DD HH:mm:ss')
  currentWeekday.value = now.format('dddd')
}

// 格式化地址显示
const formatAddress = (address: string) => {
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

// 格式化幸运号码显示
const formatLuckNumber = (luckNumber: string) => {
  return `${luckNumber.slice(0, 6)}...${luckNumber.slice(-4)}`
}

// 格式化奖项显示
const formatPrizeGrade = (grade: string) => {
  return grade.split('_').map(word => 
    word.charAt(0).toUpperCase() + word.slice(1)
  ).join(' ')
}

// 获取奖项标签类型
const getPrizeTagType = (grade: string): 'success' | 'warning' | 'info' | 'primary' | 'danger' => {
  const types: Record<string, 'success' | 'warning' | 'info'> = {
    first_prize: 'success',
    second_prize: 'warning',
    third_prize: 'info'
  }
  return types[grade as keyof typeof types] || 'info'
}

// 格式化时间显示
const formatTime = (time: string) => {
  return dayjs(time).format('YYYY-MM-DD HH:mm:ss')
}

// 获取奖金池信息
const fetchPrizePool = async () => {
  try {
    const response = await apiService.getPrizePool()
    console.log("prize pool response: ", response)
    const data = response.data as unknown as PrizePoolPrizeExpectionResponse
    console.log("prize pool data: ", data)
    if (data) {
      //prizePool.value = data.prize_amount
      prizeExpectation.value = data
    }
  } catch (error) {
    console.error('Failed to fetch prize pool:', error)
  }
}

// 获取首页信息
const fetchHomeInfo = async () => {
  try {
    const response = await apiService.getHomeInfo()
    console.log("home info response: ", response)
    const data = response.data as unknown as HomeInfoResponse
    if (data) {
      homeInfo.value = data
    }
  } catch (error) {
    console.error('Failed to fetch home info:', error)
  }
}

onMounted(() => {
  updateTime()
  timer = setInterval(updateTime, 1000)
  fetchPrizePool()
  if (isLoggedIn.value) {
    fetchHomeInfo()
  }
})

onUnmounted(() => {
  if (timer) {
    clearInterval(timer)
  }
})
</script>

<style scoped>
.home {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.time-display {
  text-align: center;
  margin-bottom: 20px;
}

.prize-pool {
  margin-bottom: 20px;
}

.prize-amount {
  font-size: 24px;
  font-weight: bold;
  color: #409EFF;
  text-align: center;
  padding: 10px 0;
}

.competition-info {
  margin-bottom: 20px;
}

.competition-details {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
}

.topic {
  font-weight: bold;
  color: #409EFF;
  font-size: 16px;
}

.rank {
  font-size: 18px;
  color: #67C23A;
  font-weight: bold;
}

.lottery-info {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

:deep(.el-card__header) {
  padding: 15px 20px;
  font-weight: bold;
}

:deep(.el-table) {
  margin-top: 10px;
}

.competition-count {
  font-size: 16px;
  color: #333;
  margin-top: 5px;
}

.prize-expectation {
  margin-bottom: 20px;
}

.prize-expectation-details {
  padding: 20px;
}

.prize-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: #606266;
  margin-bottom: 16px;
  padding-left: 8px;
  border-left: 3px solid #409EFF;
}

.prize-expectation-item {
  display: flex;
  align-items: center;
  padding: 16px;
  margin-bottom: 12px;
  border-radius: 8px;
  background: #f5f7fa;
  transition: all 0.3s ease;
}

.prize-expectation-item:hover {
  transform: translateX(5px);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
}

.prize-content {
  margin-left: 12px;
  flex: 1;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.prize-expectation-label {
  font-weight: 600;
  color: #303133;
}

.prize-expectation-value {
  font-weight: bold;
  color: #409EFF;
}

/* 主要奖项样式 */
.prize-first {
  background: linear-gradient(45deg, #fdf6ec, #fff);
  border-left: 4px solid #E6A23C;
}

.prize-second {
  background: linear-gradient(45deg, #f0f9eb, #fff);
  border-left: 4px solid #67C23A;
}

.prize-third {
  background: linear-gradient(45deg, #ecf5ff, #fff);
  border-left: 4px solid #409EFF;
}

/* 代理奖项样式 */
.agent-prizes .prize-expectation-item {
  background: linear-gradient(45deg, #f4f4f5, #fff);
  border-left: 4px solid #909399;
}

:deep(.el-icon) {
  font-size: 20px;
  color: #909399;
}

.prize-first :deep(.el-icon) {
  color: #E6A23C;
}

.prize-second :deep(.el-icon) {
  color: #67C23A;
}

.prize-third :deep(.el-icon) {
  color: #409EFF;
}
</style> 