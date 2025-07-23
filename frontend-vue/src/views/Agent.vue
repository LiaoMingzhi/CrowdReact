<template>
  <div class="page-container">
    <template v-if="userStore.isLoggedIn">
      <div class="agent-container">
        <el-card v-loading="loading">
          <template #header>
            <div class="card-header">
              <span>Agent Information</span>
            </div>
          </template>

          <!-- User info display -->
          <div class="user-info">
            <div class="info-row">
              <div class="info-item">
                <span class="label">ETH Account Address:</span>
                <span class="address">{{ formatAddress(agentInfo.user_address) }}</span>
              </div>
              <div class="info-item">
                <span class="label">Agent Level:</span>
                <span class="level">{{ formatLevel(agentInfo.level_agent) }}</span>
              </div>
            </div>
          </div>

          <!-- Agent details -->
          <div class="agent-details" v-if="agentInfo.agent_details">
            <!-- One Agent -->
            <div class="agent-section" v-if="agentInfo.agent_details.one_agent">
              <h3>One Agent</h3>
              <div class="address-list">
                <el-tooltip
                  :content="agentInfo.agent_details.one_agent.user_address"
                  placement="top"
                  effect="light"
                >
                  <el-tag type="success">
                    {{ formatAddress(agentInfo.agent_details.one_agent.user_address) }}
                  </el-tag>
                </el-tooltip>
              </div>
            </div>

            <!-- Two Agents -->
            <div class="agent-section" v-if="agentInfo.agent_details?.two_agents?.length">
              <h3>Two Agents</h3>
              <div class="address-list">
                <el-tooltip
                  v-for="agent in agentInfo.agent_details.two_agents"
                  :key="agent.user_address"
                  :content="agent.user_address"
                  placement="top"
                  effect="light"
                >
                  <el-tag 
                    type="warning"
                    class="address-tag"
                  >
                    {{ formatAddress(agent.user_address) }}
                  </el-tag>
                </el-tooltip>
              </div>
            </div>

            <!-- Common Agents -->
            <div class="agent-section" v-if="agentInfo.agent_details?.common_agents?.length">
              <h3>Common Agents</h3>
              <div class="address-list">
                <el-tooltip
                  v-for="agent in agentInfo.agent_details.common_agents"
                  :key="agent.user_address"
                  :content="agent.user_address"
                  placement="top"
                  effect="light"
                >
                  <el-tag 
                    type="info"
                    class="address-tag"
                  >
                    {{ formatAddress(agent.user_address) }}
                  </el-tag>
                </el-tooltip>
              </div>
            </div>
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
import { useUserStore } from '@/stores/user'
import { apiService } from '@/api/backend'
import { ElMessage } from 'element-plus'
import { AgentInfoResponse } from '@/api/backend'




const userStore = useUserStore()
const router = useRouter()
const loading = ref(false)
const agentInfo = ref<AgentInfoResponse>({
  user_address: '',
  level_agent: '',
  agent_details: {
    one_agent: null,
    two_agents: [],
    common_agents: []
  }
})

// 格式化地址显示
const formatAddress = (address: string) => {
  if (!address) return ''
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

// 格式化等级显示
const formatLevel = (level: string | undefined | null) => {
  if (!level) return 'Not Agent'
  return level.charAt(0).toUpperCase() + level.slice(1)
}

const handleLogin = () => {
  router.push('/')
}

// 获取代理信息
async function fetchAgentInfo() {
  if (!userStore.isLoggedIn) {
    ElMessage.warning('Please login first')
    return
  }

  loading.value = true
  try {
    const response = await apiService.getAgentDetails()
    console.log("Agent details response:", response)
    
    if (response.data.status === 'success') {
      console.log("Agent details data:", response.data.data)
      console.log("Two agents:", response.data.data.agent_details.two_agents)
      console.log("Common agents:", response.data.data.agent_details.common_agents)
      
      agentInfo.value = {
        user_address: response.data.data.user_address,
        level_agent: response.data.data.level_agent,
        agent_details: {
          one_agent: response.data.data.agent_details.one_agent || null,
          two_agents: response.data.data.agent_details.two_agents || [],
          common_agents: response.data.data.agent_details.common_agents || []
        }
      }
      
      console.log("Processed agent info:", agentInfo.value)
    } else {
      throw new Error('Invalid response format')
    }
  } catch (error) {
    console.error('Failed to fetch agent information:', error)
    ElMessage.error('Failed to fetch agent information')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  if (userStore.isLoggedIn) {
    fetchAgentInfo()
  }
})
</script>

<style scoped>
.agent-container {
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

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.info-item {
  display: flex;
  align-items: center;
  white-space: nowrap;
}

.label {
  font-weight: bold;
  margin-right: 10px;
  color: #606266;
}

.address, .level {
  font-family: monospace;
  color: #409EFF;
  margin-left: 8px;
}

.agent-details {
  margin-top: 20px;
}

.agent-section {
  margin-bottom: 20px;
}

.agent-section h3 {
  margin-bottom: 10px;
  color: #606266;
}

.address-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.address-tag {
  cursor: pointer;
  transition: all 0.3s;
}

.address-tag:hover {
  transform: translateY(-2px);
}

:deep(.el-tooltip__popper) {
  font-family: monospace;
  max-width: 300px;
  word-break: break-all;
}

:deep(.el-loading-mask) {
  background-color: rgba(255, 255, 255, 0.8);
}
</style> 