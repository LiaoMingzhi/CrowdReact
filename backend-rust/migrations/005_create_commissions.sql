CREATE TABLE IF NOT EXISTS commissions (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(255) NOT NULL,
    from_address VARCHAR(255) NOT NULL,
    commission DECIMAL(20, 8) NOT NULL,
    transaction_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_user_address ON commissions(user_address);
CREATE INDEX IF NOT EXISTS idx_from_address ON commissions(from_address);
CREATE INDEX IF NOT EXISTS idx_created_at ON commissions(created_at);
