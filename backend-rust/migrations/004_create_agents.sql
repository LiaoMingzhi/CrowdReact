CREATE TABLE IF NOT EXISTS agents (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(255) NOT NULL UNIQUE,
    level_agent VARCHAR(20) NOT NULL DEFAULT 'not_agent' 
        CHECK (level_agent IN ('one', 'two', 'common', 'not_agent')),
    superior_address VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_frozen BOOLEAN NOT NULL DEFAULT FALSE
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_agents_user_address ON agents(user_address);
CREATE INDEX IF NOT EXISTS idx_agents_level_agent ON agents(level_agent);
CREATE INDEX IF NOT EXISTS idx_agents_superior_address ON agents(superior_address);

ALTER TABLE agents DROP CONSTRAINT IF EXISTS fk_superior_address;

ALTER TABLE agents 
ADD CONSTRAINT fk_superior_address
FOREIGN KEY (superior_address) 
REFERENCES agents(user_address)
ON DELETE SET NULL;
