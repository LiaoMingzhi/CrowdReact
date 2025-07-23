CREATE TABLE IF NOT EXISTS platform_prize_pool (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(255) NOT NULL,
    from_address VARCHAR(255) NOT NULL,
    amount DECIMAL(20, 8) NOT NULL,
    transaction_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_user_address ON platform_prize_pool(user_address);
CREATE INDEX IF NOT EXISTS idx_from_address ON platform_prize_pool(from_address);
CREATE INDEX IF NOT EXISTS idx_created_at ON platform_prize_pool(created_at);