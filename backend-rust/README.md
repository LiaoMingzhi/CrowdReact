cargo check

cargo clean

cargo build

$env:RUST_LOG="debug"

cargo run


生成jwt_secret的值：

# 在 Git Bash 终端中运行, 获取jwt_secret
openssl rand -base64 32

openssl rand -base64 32


# 安装 OpenSSL 开发包和其他必要的构建工具
sudo apt install -y pkg-config libssl-dev build-essential



cargo build --release

# 1.系统准备
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装必要的系统依赖
sudo apt install -y build-essential pkg-config libssl-dev curl git postgresql postgresql-contrib redis-server
# 2. 安装 Rust
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 配置环境变量
source $HOME/.cargo/env

# 安装 nightly 版本并设置为默认
rustup default nightly

# 3. 配置 PostgreSQL
# 启动 PostgreSQL 服务
sudo systemctl start postgresql
sudo systemctl enable postgresql

# 创建数据库和用户
sudo -u postgres psql

postgres=# CREATE DATABASE luck_game;
postgres=# CREATE USER your_user WITH ENCRYPTED PASSWORD 'your_password';
postgres=# GRANT ALL PRIVILEGES ON DATABASE luck_game TO your_user;
postgres=# \q


# 4. 配置 Redis
# 启动 Redis 服务
sudo systemctl start redis-server
sudo systemctl enable redis-server

# 编辑 Redis 配置
sudo vim /etc/redis/redis.conf

# 修改以下配置
bind 127.0.0.1
requirepass your_redis_password  # 如果需要密码

# 5. 项目部署
# 创建部署目录
sudo mkdir -p /opt/backend-rust
sudo chown $USER:$USER /opt/backend-rust

# 克隆项目
git clone your_repository_url /opt/backend-rust

# 进入项目目录
cd /opt/backend-rust

# 创建生产环境配置
cp config/production.toml config/local.toml


#6. 配置环境变量
export RUST_LOG="debug"
sudo vim /opt/backend-rust/.env

ETHEREUM_PRIVATE_KEY=your_private_key
CONTRACT_ADDRESS=your_contract_address
OWNER_ADDRESS=your_owner_address
JWT_SECRET=your_jwt_secret
TOKEN_DURATION_HOURS=168
DATABASE_URL=postgres://your_user:your_password@localhost:5432/luck_game
REDIS_URL=redis://127.0.0.1:6379

7. 编译项目
cd /opt/backend-rust
cargo build --release

# 启动项目
cargo run --release


# 8. 创建系统服务
创建服务文件
vim /etc/systemd/system/backend-rust.service
添加以下内容：
[Unit]
Description=Backend Rust Service
After=network.target postgresql.service redis-server.service

[Service]
Type=simple
User=your_user
Group=your_user
WorkingDirectory=/opt/backend-rust
Environment="RUST_ENV=production"
EnvironmentFile=/opt/backend-rust/.env
ExecStart=/opt/backend-rust/target/release/backend-rust
Restart=on-failure  # 只在失败时重启， 默认是 always
RestartSec=10   # 重启前等待 10 秒
StartLimitInterval=600   # 10 分钟内
StartLimitBurst=5        # 最多重启 5 次

[Install]
WantedBy=multi-user.target


# 9. 启动服务
# 重新加载 systemd
sudo systemctl daemon-reload

# 启动服务
sudo systemctl start backend-rust

# 设置开机自启
sudo systemctl enable backend-rust

结果如下：
Created symlink '/etc/systemd/system/multi-user.target.wants/backend-rust.service' → '/etc/systemd/system/backend-rust.service'.

# 查看服务状态
sudo systemctl status backend-rust

结果如下：
backend-rust.service - Backend Rust Service
     Loaded: loaded (/etc/systemd/system/backend-rust.service; enabled; preset: enabled)
     Active: active (running) since Sat 2024-12-28 12:55:35 UTC; 57s ago
 Invocation: 969606f3aacc4650ba33fb50e34ead84
   Main PID: 301435 (backend-rust)
      Tasks: 4 (limit: 2318)
     Memory: 14.5M (peak: 14.8M)
        CPU: 141ms
     CGroup: /system.slice/backend-rust.service
             └─301435 /opt/backend-rust/target/release/backend-rust

Dec 28 12:55:35 hadoop01 systemd[1]: Started backend-rust.service - Backend Rust Service.


# 查看日志
sudo journalctl -u backend-rust -f

# 停止服务
sudo systemctl stop backend-rust



# 查看实时日志
sudo journalctl -u backend-rust.service -f

# 查看今天的日志
sudo journalctl -u backend-rust.service --since today

# 查看最近100行日志
sudo journalctl -u backend-rust.service -n 100


https://etherscan.io/address/你的合约地址
https://etherscan.io/address/0xE8C48BDE89ed3e08DfB5E67573DF8c6D2C60C324

使用其他区块浏览器：
Blockchair
Etherchain
Ethplorer

https://ethereum.org/en/developers/docs/apis/json-rpc/

https://ethereum-rpc.publicnode.com