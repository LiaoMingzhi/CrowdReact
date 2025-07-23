CREATE TABLE IF NOT EXISTS platform_transaction_gas (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(255) NOT NULL,
    from_address VARCHAR(255) NOT NULL,
    amount_wei DECIMAL NOT NULL,
    transaction_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_user_address ON platform_transaction_gas(user_address);
CREATE INDEX IF NOT EXISTS idx_from_address ON platform_transaction_gas(from_address);
CREATE INDEX IF NOT EXISTS idx_created_at ON platform_transaction_gas(created_at);