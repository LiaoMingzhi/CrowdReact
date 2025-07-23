<template>
  <div>
    <form @submit.prevent="handleBet" class="betting-form">
      <div class="form-group">
        <label for="amount">下注金额 (ETH)</label>
        <input 
          id="amount"
          v-model="amount" 
          type="number" 
          step="0.01" 
          min="0.01"
          required
          placeholder="请输入下注金额" 
        />
      </div>
      
      <div class="form-group">
        <label for="number">下注数字</label>
        <input 
          id="number"
          v-model="number" 
          type="number" 
          min="1" 
          max="99"
          required
          placeholder="请输入1-99之间的数字" 
        />
      </div>

      <button type="submit" :disabled="!isValid">下注</button>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useBettingService } from '@/composables/useBettingService'

const amount = ref('')
const number = ref('')
const bettingService = useBettingService()

const isValid = computed(() => {
  const amountNum = parseFloat(amount.value)
  const numberNum = parseInt(number.value)
  return amountNum > 0 && numberNum >= 1 && numberNum <= 99
})

async function handleBet() {
  try {
    if (!isValid.value) {
      alert('请输入有效的下注金额和数字')
      return
    }
    
    const betAmount = amount.value
    const betNumber = parseInt(number.value)
    
    console.log('提交下注参数:', {
      amount: betAmount,
      number: betNumber,
      isNumberValid: !isNaN(betNumber),
      numberType: typeof betNumber
    })
    
    await bettingService.placeBet(betAmount, betNumber)
    
    alert('下注成功！')
    // 清空表单
    amount.value = ''
    number.value = ''
  } catch (error) {
    console.error('下注失败:', error)
    alert('下注失败：' + (error as Error).message)
  }
}
</script>

<style scoped>
.betting-form {
  max-width: 400px;
  margin: 0 auto;
  padding: 20px;
}

.form-group {
  margin-bottom: 15px;
}

label {
  display: block;
  margin-bottom: 5px;
}

input {
  width: 100%;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

button {
  width: 100%;
  padding: 10px;
  background-color: #4CAF50;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:disabled {
  background-color: #cccccc;
  cursor: not-allowed;
}
</style> 