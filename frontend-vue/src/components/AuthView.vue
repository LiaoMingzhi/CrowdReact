<template>
    <el-card class="auth-card">
      <el-tabs v-model="activeTab">
        <el-tab-pane label="登录" name="login">
          <el-form :model="loginForm" :rules="loginRules" ref="loginFormRef">
            <el-form-item prop="email" label="邮箱">
              <el-input v-model="loginForm.email" type="email"/>
            </el-form-item>
            <el-form-item prop="password" label="密码">
              <el-input v-model="loginForm.password" type="password"/>
            </el-form-item>
            <el-button type="primary" @click="handleLogin">登录</el-button>
          </el-form>
        </el-tab-pane>
        
        <el-tab-pane label="注册" name="register">
          <el-form :model="registerForm" :rules="registerRules" ref="registerFormRef">
            <el-form-item prop="firstName" label="名">
              <el-input v-model="registerForm.firstName"/>
            </el-form-item>
            <el-form-item prop="lastName" label="姓">
              <el-input v-model="registerForm.lastName"/>
            </el-form-item>
            <el-form-item prop="email" label="邮箱">
              <el-input v-model="registerForm.email" type="email"/>
            </el-form-item>
            <el-form-item prop="password" label="密码">
              <el-input v-model="registerForm.password" type="password"/>
            </el-form-item>
            <el-button type="primary" @click="handleRegister">注册</el-button>
          </el-form>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import { useStore } from 'vuex'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import type { FormInstance } from 'element-plus'

interface LoginForm {
  email: string
  password: string
}

interface RegisterForm extends LoginForm {
  firstName: string
  lastName: string
}

export default defineComponent({
  name: 'AuthView',
  setup() {
    const store = useStore()
    const router = useRouter()
    const activeTab = ref('login')
    const loginFormRef = ref<FormInstance>()
    const registerFormRef = ref<FormInstance>()

    const loginForm = ref<LoginForm>({
      email: '',
      password: ''
    })

    const registerForm = ref<RegisterForm>({
      firstName: '',
      lastName: '',
      email: '',
      password: ''
    })

    const loginRules = {
      email: [
        { required: true, message: '请输入邮箱', trigger: 'blur' },
        { type: 'email', message: '请输入正确的邮箱格式', trigger: 'blur' }
      ],
      password: [
        { required: true, message: '请输入密码', trigger: 'blur' },
        { min: 6, message: '密码长度不能小于6位', trigger: 'blur' }
      ]
    }

    const registerRules = {
      ...loginRules,
      firstName: [
        { required: true, message: '请输入名', trigger: 'blur' }
      ],
      lastName: [
        { required: true, message: '请输入姓', trigger: 'blur' }
      ]
    }

    const handleLogin = async () => {
      if (!loginFormRef.value) return
      await loginFormRef.value.validate(async (valid) => {
        if (valid) {
          try {
            await store.dispatch('login', loginForm.value)
            ElMessage.success('登录成功')
            router.push('/betting')
          } catch (error) {
            ElMessage.error('登录失败')
          }
        }
      })
    }

    const handleRegister = async () => {
      if (!registerFormRef.value) return
      await registerFormRef.value.validate(async (valid) => {
        if (valid) {
          try {
            await store.dispatch('register', registerForm.value)
            ElMessage.success('注册成功')
            activeTab.value = 'login'
          } catch (error) {
            ElMessage.error('注册失败')
          }
        }
      })
    }

    return {
      activeTab,
      loginForm,
      registerForm,
      loginRules,
      registerRules,
      loginFormRef,
      registerFormRef,
      handleLogin,
      handleRegister
    }
  }
})
</script>

<style scoped>
.auth-card {
  max-width: 500px;
  margin: 100px auto;
}
</style>