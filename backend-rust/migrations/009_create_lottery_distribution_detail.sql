-- DROP TABLE IF EXISTS lottery_distribution_detail;
CREATE TABLE IF NOT EXISTS lottery_distribution_detail (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(255) NOT NULL,
    prize_amount DECIMAL(36, 18) NOT NULL,
    prize_grade VARCHAR(50) NOT NULL,
    luck_number VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add index for faster queries
CREATE INDEX IF NOT EXISTS idx_lottery_distribution_user_address ON lottery_distribution_detail(user_address);
CREATE INDEX IF NOT EXISTS idx_lottery_distribution_created_at ON lottery_distribution_detail(created_at);
