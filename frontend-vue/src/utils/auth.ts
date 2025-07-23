import { ethers } from 'ethers'
import { sha256 } from '@/utils/crypto'

export function generatePasswordFromAddress(address: string): string {

  // 使用 crypto-js 的 sha256 而不是 ethers.js 的 sha256
  // 直接使用原始地址
  return sha256(address)
} 