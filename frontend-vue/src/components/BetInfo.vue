<template>
    <el-card>
      <template #header>
        <div class="card-header">
          <span>查看投注</span>
        </div>
      </template>
      <div>
        <p>购买金额: {{ betInfo.amount }} ETH</p>
        <p>账户地址: {{ betInfo.address }}</p>
        <p>购买号码: {{ betInfo.number }}</p>
        <p>购买时间: {{ betInfo.time }}</p>
      </div>
    </el-card>
  </template>
  
  <script>
  import { computed } from 'vue'
  import { useStore } from 'vuex'
  
  export default {
    name: 'BetInfo',
    setup() {
      const store = useStore()
      return {
        betInfo: computed(() => store.state.betInfo)
      }
    }
  }
  </script>