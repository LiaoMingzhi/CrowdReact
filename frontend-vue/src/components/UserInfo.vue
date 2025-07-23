<template>
  <el-card>
    <template #header>
      <div class="card-header">
        <span>用户信息</span>
      </div>
    </template>
    <div>
      <p>用户邮箱: {{ userInfo.email }}</p>
      <p>用户姓名: {{ userInfo.firstName }} {{ userInfo.lastName }}</p>
      <p>账户余额: {{ userInfo.balance }} ETH</p>
      <p>注册时间: {{ userInfo.registerTime }}</p>
    </div>
  </el-card>
</template>

<script lang="ts">
import { defineComponent, computed } from 'vue'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'UserInfo',
  setup() {
    const store = useStore()
    return {
      userInfo: computed(() => store.state.user.userInfo)
    }
  }
})
</script>