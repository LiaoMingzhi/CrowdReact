<template>
  <el-card>
    <template #header>
      <div class="card-header">
        <span>佣金收入详情</span>
      </div>
    </template>
    <el-table :data="commissionDetails" style="width: 100%">
      <el-table-column prop="amount" label="入账金额">
        <template #default="scope">
          {{ scope.row.amount }} ETH
        </template>
      </el-table-column>
      <el-table-column prop="date" label="入账时间" />
      <el-table-column prop="type" label="类型" />
    </el-table>
  </el-card>
</template>

<script lang="ts">
import { defineComponent, computed } from 'vue'
import { useStore } from 'vuex'
import type { CommissionDetailResponse } from '@/api'

export default defineComponent({
  name: 'CommissionDetails',
  setup() {
    const store = useStore()
    return {
      commissionDetails: computed(() => store.state.commissionDetails as CommissionDetailResponse[])
    }
  }
})
</script>