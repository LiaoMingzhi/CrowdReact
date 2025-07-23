<template>
    <el-card>
      <template #header>
        <div class="card-header">
          <span>投注</span>
        </div>
      </template>
      <el-form>
        <el-form-item label="投注金额 (ETH)">
          <el-input 
            v-model="localBetAmount" 
            type="number"
            @input="updateBetAmount"
          ></el-input>
        </el-form-item>
        <el-button type="primary" @click="$emit('place-bet')">投注</el-button>
      </el-form>
    </el-card>
  </template>
  
  <script>
  import { computed } from 'vue'
  
  export default {
    name: 'BetPanel',
    props: {
      betAmount: {
        type: Number,
        required: true
      }
    },
    emits: ['update:bet-amount', 'place-bet'],
    setup(props, { emit }) {
      const localBetAmount = computed({
        get: () => props.betAmount,
        set: (value) => emit('update:bet-amount', value)
      })
  
      const updateBetAmount = (value) => {
        emit('update:bet-amount', Number(value))
      }
  
      return {
        localBetAmount,
        updateBetAmount
      }
    }
  }
  </script>