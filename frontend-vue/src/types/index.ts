export interface CommissionDetailResponse {
  date: string;
  amount: number;
  type: string;
}

export interface UplineAgent {
  id: string
  address: string
  name?: string
} 