<template>
  <el-menu mode="horizontal">
    <!-- 其他菜单项 -->
    
    <el-menu-item v-if="userStore.isLoggedIn" index="profile">
      <router-link to="/profile">个人设置</router-link>
    </el-menu-item>
    
    <el-menu-item v-if="userStore.isLoggedIn" index="logout">
      <span @click="handleLogout">退出登录</span>
    </el-menu-item>
  </el-menu>
</template>

<script setup lang="ts">
import { useUserStore } from '@/stores/user'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'

const userStore = useUserStore()
const router = useRouter()

const handleLogout = () => {
  userStore.logout()
  ElMessage.success('已退出登录')
  router.push('/login')
}
</script> 