<template>
    <el-card>
      <template #header>
        <div class="card-header">
          <span>代理信息</span>
        </div>
      </template>
      <div>
        <p>代理角色: {{ agentInfo.role }}</p>
        <p>上级代理: {{ agentInfo.upline }}</p>
        <p>下级代理: {{ agentInfo.downline }}</p>
        <p>总佣金: {{ agentInfo.totalCommission }} ETH</p>
        <p>佣金排名: {{ agentInfo.ranking }}</p>
      </div>
    </el-card>
  </template>
  
  <script lang="ts">
  import { defineComponent, computed } from 'vue'
  import { useStore } from 'vuex'
  
  export default defineComponent({
    name: 'AgentInfo',
    setup() {
      const store = useStore()
      return {
        agentInfo: computed(() => store.state.agentInfo)
      }
    }
  })
  </script>