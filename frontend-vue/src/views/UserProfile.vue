<template>
  <div class="profile-container">
    <el-card class="profile-card">
      <template #header>
        <div class="card-header">
          <span>用户设置</span>
        </div>
      </template>

      <el-form 
        :model="formData" 
        :rules="rules"
        ref="formRef"
        label-width="120px"
      >
        <el-form-item label="钱包地址">
          <el-input v-model="userStore.walletAddress" disabled />
        </el-form-item>

        <el-form-item 
          label="当前密码" 
          prop="currentPassword"
        >
          <el-input
            v-model="formData.currentPassword"
            :type="showCurrentPassword ? 'text' : 'password'"
          >
            <template #suffix>
              <el-icon 
                class="password-icon" 
                @click="showCurrentPassword = !showCurrentPassword"
              >
                <View v-if="!showCurrentPassword" />
                <Hide v-else />
              </el-icon>
            </template>
          </el-input>
        </el-form-item>

        <el-form-item 
          label="新密码" 
          prop="newPassword"
        >
          <el-input
            v-model="formData.newPassword"
            :type="showNewPassword ? 'text' : 'password'"
          >
            <template #suffix>
              <el-icon 
                class="password-icon" 
                @click="showNewPassword = !showNewPassword"
              >
                <View v-if="!showNewPassword" />
                <Hide v-else />
              </el-icon>
            </template>
          </el-input>
        </el-form-item>

        <el-form-item>
          <el-button 
            type="primary" 
            @click="handleChangePassword"
            :loading="loading"
          >
            修改密码
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { View, Hide } from '@element-plus/icons-vue'
import { useUserStore } from '@/stores/user'
import type { FormInstance } from 'element-plus'

const userStore = useUserStore()
const formRef = ref<FormInstance>()
const loading = ref(false)
const showCurrentPassword = ref(false)
const showNewPassword = ref(false)

interface FormData {
  currentPassword: string
  newPassword: string
}

const formData = ref<FormData>({
  currentPassword: '',
  newPassword: ''
})

const rules = {
  currentPassword: [
    { required: true, message: '请输入当前密码', trigger: 'blur' },
    { min: 6, message: '密码长度至少6个字符', trigger: 'blur' }
  ],
  newPassword: [
    { required: true, message: '请输入新密码', trigger: 'blur' },
    { min: 6, message: '密码长度至少6个字符', trigger: 'blur' }
  ]
}

const handleChangePassword = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
    loading.value = true

    await userStore.changePassword(
      formData.value.currentPassword,
      formData.value.newPassword
    )

    ElMessage.success('密码修改成功')
    formData.value.currentPassword = ''
    formData.value.newPassword = ''
  } catch (error) {
    ElMessage.error(`密码修改失败: ${error instanceof Error ? error.message : '未知错误'}`)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.profile-container {
  padding: 20px;
  max-width: 600px;
  margin: 0 auto;
}

.profile-card {
  background-color: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.password-icon {
  cursor: pointer;
}
</style> 