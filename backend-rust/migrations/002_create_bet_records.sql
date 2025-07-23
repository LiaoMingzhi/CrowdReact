CREATE TABLE IF NOT EXISTS bet_records (
    id SERIAL PRIMARY KEY,
    account_address VARCHAR(42) NOT NULL,
    amount VARCHAR(255) NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    block_number BIGINT NOT NULL,
    block_timestamp BIGINT NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 为常用查询添加复合索引
CREATE INDEX IF NOT EXISTS idx_bet_records_user_status ON bet_records(account_address, status);
CREATE INDEX IF NOT EXISTS idx_bet_records_timestamp ON bet_records(block_timestamp DESC);

-- 添加部分索引以提高特定查询性能
CREATE INDEX IF NOT EXISTS idx_active_bets ON bet_records(account_address) WHERE status = 'pending';
