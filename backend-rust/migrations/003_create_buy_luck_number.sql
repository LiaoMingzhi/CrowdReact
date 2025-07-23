CREATE TABLE IF NOT EXISTS buy_luck_number (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(255) NOT NULL,
    luck_number VARCHAR(36) NOT NULL,
    transaction_hash VARCHAR(66),
    is_winner BOOLEAN DEFAULT FALSE,
    prize_level VARCHAR(20),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_buy_luck_number_user_address ON buy_luck_number(user_address);
CREATE INDEX IF NOT EXISTS idx_buy_luck_number_luck_number ON buy_luck_number(luck_number);
CREATE INDEX IF NOT EXISTS idx_buy_luck_number_transaction_hash ON buy_luck_number(transaction_hash);
  