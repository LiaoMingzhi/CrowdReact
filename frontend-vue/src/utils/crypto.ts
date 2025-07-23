import SHA256 from 'crypto-js/sha256'

export function sha256(data: string): string {
  return SHA256(data).toString()
} 