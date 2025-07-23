<template>
  <div class="header">
    <!-- ... existing header content ... -->
    <div class="user-status">
      <template v-if="isLoggedIn">
        <span class="username">{{ username }}</span>
        <el-button size="small" @click="handleLogout">Logout</el-button>
      </template>
      <template v-else>
        <el-button size="small" @click="handleLogin">Login</el-button>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'

const router = useRouter()
const isLoggedIn = ref(false)
const username = ref('')

onMounted(() => {
  checkLoginStatus()
})

const checkLoginStatus = () => {
  const token = localStorage.getItem('token')
  const storedUsername = localStorage.getItem('username')
  isLoggedIn.value = !!token
  username.value = storedUsername || ''
}

const handleLogout = () => {
  localStorage.removeItem('token')
  localStorage.removeItem('username')
  isLoggedIn.value = false
  username.value = ''
  ElMessage.success('Logout successful')
  router.push('/login')
}

const handleLogin = () => {
  router.push('/login')
}
</script>

<style scoped>
.header {
  /* ... existing styles ... */
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 20px;
}

.user-status {
  display: flex;
  align-items: center;
  gap: 10px;
}

.username {
  color: #fff;
  margin-right: 10px;
}
</style> 