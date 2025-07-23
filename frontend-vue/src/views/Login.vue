<template>
  <el-dialog
    v-model="dialogVisible"
    title="Login"
    width="30%"
    :before-close="handleClose"
  >
    <el-form :model="loginForm" :rules="rules" ref="loginFormRef">
      <el-form-item prop="username" label="Username">
        <el-input v-model="loginForm.username" autocomplete="off"></el-input>
      </el-form-item>
      <el-form-item prop="password" label="Password">
        <el-input
          v-model="loginForm.password"
          type="password"
          autocomplete="off"
        ></el-input>
      </el-form-item>
    </el-form>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="handleClose">Cancel</el-button>
        <el-button type="primary" @click="handleLogin" :loading="loading">
          Login
        </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { ElMessage } from 'element-plus'
import type { FormInstance } from 'element-plus'
import { useUserStore } from '@/stores/user'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits(['update:visible'])

const userStore = useUserStore()
const loading = ref(false)
const loginFormRef = ref<FormInstance>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

const loginForm = reactive({
  username: '',
  password: ''
})

const rules = {
  username: [
    { required: true, message: 'Please input username', trigger: 'blur' }
  ],
  password: [
    { required: true, message: 'Please input password', trigger: 'blur' }
  ]
}

const handleClose = () => {
  dialogVisible.value = false
  loginForm.username = ''
  loginForm.password = ''
}

const handleLogin = async () => {
  if (!loginFormRef.value) return

  try {
    await loginFormRef.value.validate()
    loading.value = true

    await userStore.login({
      username: loginForm.username,
      password: loginForm.password
    })

    ElMessage.success('Login successful')
    handleClose()
  } catch (error) {
    console.error('Login error:', error)
    ElMessage.error(error instanceof Error ? error.message : 'Login failed')
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style> 