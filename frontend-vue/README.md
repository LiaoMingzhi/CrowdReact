# frontend-vue

## Project setup
```
npm install
```

### Compiles and hot-reloads for development
```
npm run dev
```

### Compiles and minifies for production
```
npm run build
```

# 构建生产版本
```
npm run build:prod
```

### Lints and fixes files
```
npm run lint
```

### Customize configuration
See [Configuration Reference](https://cli.vuejs.org/config/).


apt install nginx

vim /etc/nginx/sites-available/frontend-vue

创建符号链接
ln -s /etc/nginx/sites-available/frontend-vue /etc/nginx/sites-enabled/
# 检查 Nginx 配置是否正确
sudo nginx -t

nginx: the configuration file /etc/nginx/nginx.conf syntax is ok
nginx: configuration file /etc/nginx/nginx.conf test is successful

# 重新加载 Nginx 配置
sudo nginx -s reload
# 重启 Nginx 服务
sudo systemctl restart nginx
如果配置正确，重启 Nginx：
systemctl restart nginx

确保防火墙允许 HTTP 流量（如果开启了防火墙）
sudo ufw allow 80
sudo ufw allow 3000
sudo ufw allow 443  # 如果使用 HTTPS

vim /etc/nginx/nginx.conf

netstat -anp|grep 3000

chmod -R 755 /root/CrowdReact/frontend-vue/dist

# 检查 dist 目录权限
ls -la /root/CrowdReact/frontend-vue/dist

# 设置正确的权限
sudo chmod -R 755 /root/CrowdReact/frontend-vue/dist
sudo chown -R www-data:www-data /root/CrowdReact/frontend-vue/dist



# 编辑 nginx.conf
sudo nano /etc/nginx/nginx.conf

# 修改 user 为 root（不推荐）或确保该用户有权限访问 dist 目录
user root;


# 编辑 nginx.conf
sudo vim /etc/nginx/nginx.conf

# 改回 www-data
user www-data;

正确设置目录权限：
# 创建专门的用户组
sudo groupadd webapps

# 将 www-data 用户添加到 webapps 组
sudo usermod -a -G webapps www-data

# 将你的项目目录权限设置正确
sudo chown -R www-data:webapps /root/CrowdReact/frontend-vue/dist
sudo chmod -R 750 /root/CrowdReact/frontend-vue/dist

tail -f /var/log/nginx/error.log

sudo ufw reload

sudo ufw status

sudo ufw delete allow 80
sudo ufw delete allow 443

sudo ufw reload


# 创建一个适当的目录
sudo mkdir -p /var/www/frontend-vue

# 移动 dist 内容
sudo mv /root/CrowdReact/frontend-vue/dist/* /var/www/frontend-vue/

# 设置正确的权限
sudo chown -R www-data:www-data /var/www/frontend-vue
sudo chmod -R 755 /var/www/frontend-vue

vim /etc/nginx/sites-available/frontend-vue
server {
    listen 3000;
    server_name 46.101.77.230;

    # 更新为新的路径
    root /var/www/frontend-vue;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }
}

sudo nginx -t
sudo nginx -s reload
sudo systemctl restart nginx



Nginx禁用一些国际地区的IP。
使用 GeoIP 模块（推荐方法）：
首先需要安装 GeoIP 模块和数据库：
# Ubuntu/Debian
sudo apt-get install nginx-module-geoip geoip-database

# CentOS
sudo yum install nginx-module-geoip GeoIP-data



root@hadoop01:~# lsb_release -a
No LSB modules are available.
Distributor ID:	Ubuntu
Description:	Ubuntu 24.10
Release:	24.10
Codename:	oracular

root@hadoop01:~# nginx -V
nginx version: nginx/1.26.0 (Ubuntu)
built with OpenSSL 3.3.1 4 Jun 2024
TLS SNI support enabled
configure arguments: --with-cc-opt='-g -O2 -Werror=implicit-function-declaration -fno-omit-frame-pointer -mno-omit-leaf-frame-pointer -ffile-prefix-map=/build/nginx-p1FNon/nginx-1.26.0=. -flto=auto -ffat-lto-objects -fstack-protector-strong -fstack-clash-protection -Wformat -Werror=format-security -fcf-protection -fdebug-prefix-map=/build/nginx-p1FNon/nginx-1.26.0=/usr/src/nginx-1.26.0-2ubuntu3 -fPIC -Wdate-time -D_FORTIFY_SOURCE=3' --with-ld-opt='-Wl,-Bsymbolic-functions -flto=auto -ffat-lto-objects -Wl,-z,relro -Wl,-z,now -fPIC' --prefix=/usr/share/nginx --conf-path=/etc/nginx/nginx.conf --http-log-path=/var/log/nginx/access.log --error-log-path=stderr --lock-path=/var/lock/nginx.lock --pid-path=/run/nginx.pid --modules-path=/usr/lib/nginx/modules --http-client-body-temp-path=/var/lib/nginx/body --http-fastcgi-temp-path=/var/lib/nginx/fastcgi --http-proxy-temp-path=/var/lib/nginx/proxy --http-scgi-temp-path=/var/lib/nginx/scgi --http-uwsgi-temp-path=/var/lib/nginx/uwsgi --with-compat --with-debug --with-pcre-jit --with-http_ssl_module --with-http_stub_status_module --with-http_realip_module --with-http_auth_request_module --with-http_v2_module --with-http_v3_module --with-http_dav_module --with-http_slice_module --with-threads --build=Ubuntu --with-http_addition_module --with-http_flv_module --with-http_gunzip_module --with-http_gzip_static_module --with-http_mp4_module --with-http_random_index_module --with-http_secure_link_module --with-http_sub_module --with-mail_ssl_module --with-stream_ssl_module --with-stream_ssl_preread_module --with-stream_realip_module --with-http_geoip_module=dynamic --with-http_image_filter_module=dynamic --with-http_perl_module=dynamic --with-http_xslt_module=dynamic --with-mail=dynamic --with-stream=dynamic --with-stream_geoip_module=dynamic

首先安装必要的包：
# 安装 GeoIP 数据库和相关工具
sudo apt install geoip-database libgeoip1t64 libgeoip-dev

apt install libnginx-mod-http-geoip

加载动态模块，创建或编辑模块配置文件：
/etc/nginx/modules-enabled/geoip.conf
#load_module modules/ngx_http_geoip_module.so;
#load_module modules/ngx_stream_geoip_module.so;



# 在 http 块中加载 GeoIP 模块
http {
# 加载 GeoIP 数据库
geoip_country /usr/share/GeoIP/GeoIP.dat;

    # 定义允许的国家
    map $geoip_country_code $allowed_country {
        default no;
        CN yes;  # 允许中国
        JP yes;  # 允许日本
        # 添加其他允许的国家代码
    }

    server {
        listen 80;
        server_name example.com;

        # 检查是否允许访问
        if ($allowed_country = no) {
            return 403;
        }

        # 其他配置...
    }
}



/etc/nginx/nginx.conf
# 在 http 块中添加以下配置
http {
# ... 其他现有配置 ...

    # GeoIP 配置
    geoip_country /usr/share/GeoIP/GeoIP.dat;
    
    # 定义允许的国家
    map $geoip_country_code $allowed_country {
        default no;
        CN yes;    # 中国
        HK yes;    # 香港
        TW yes;    # 台湾
        # 添加其他需要允许的国家代码
    }

    # ... 其他现有配置 ...

    server {
        listen 80;
        server_name your_domain.com;

        # 添加访问控制
        if ($allowed_country = no) {
            return 403 "Access denied by geographic restriction";
        }

        # ... 其他服务器配置 ...
    }
}


# 购买域名与添加https的SSL服务，在nginx中应该怎样部署？
1. 首先，安装 Certbot（用于获取免费的 Let's Encrypt SSL 证书）：
# 安装 certbot
sudo apt update
sudo apt install certbot python3-certbot-nginx

sudo ufw allow 443

2. 假设您已经购买了域名（例如 example.com），首先配置基本的 HTTP 服务：
vim /etc/nginx/sites-available/front-vue
server {
   listen 3000;
   server_name luckgame123.com www.luckgame123.com;

   root /var/www/frontend-vue;
   index index.html;

   # GeoIP 配置（如果需要）
   if ($allowed_country = no) {
   return 403 "Access denied by geographic restriction";
   }

   location /api/ {
   rewrite ^/api/(.*) /$1 break;
   proxy_pass http://127.0.0.1:9080;
   proxy_http_version 1.1;
   proxy_set_header Host $host;
   proxy_set_header X-Real-IP $remote_addr;
   proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
   proxy_set_header X-Forwarded-Proto $scheme;

        add_header 'Access-Control-Allow-Origin' '*' always;
        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS, PUT, DELETE' always;
        add_header 'Access-Control-Allow-Headers' '*' always;

        try_files '' @backend;
   }

   location @backend {
   proxy_pass http://127.0.0.1:9080;
   proxy_http_version 1.1;
   proxy_set_header Host $host;
   proxy_set_header X-Real-IP $remote_addr;
   }

   location / {
   try_files $uri $uri/ /index.html;
   }

   # listen 443 ssl; # managed by Certbot
   # ssl_certificate /etc/letsencrypt/live/luckgame123.com/fullchain.pem; # managed by Certbot
   # ssl_certificate_key /etc/letsencrypt/live/luckgame123.com/privkey.pem; # managed by Certbot
   # include /etc/letsencrypt/options-ssl-nginx.conf; # managed by Certbot
   # ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem; # managed by Certbot
}

查看是否解析成功
dig luckgame123.com
dig www.luckgame123.com

3. 使用 Certbot 获取 SSL 证书：
# 获取证书并自动配置 Nginx
sudo certbot --nginx -d luckgame123.com -d www.luckgame123.com

# 或者只获取证书（如果想手动配置 Nginx）
# sudo certbot certonly --nginx -d example.com -d www.luckgame123.com

certbot --nginx -d luckgame123.com -d www.luckgame123.com

4. 创建符号链接启用站点：
sudo ln -s /etc/nginx/sites-available/frontend-vue /etc/nginx/sites-enabled/


5. 测试配置并重启 Nginx：
# 测试配置
sudo nginx -t

# 如果测试通过，重启 Nginx
sudo nginx -s reload
sudo systemctl restart nginx

6. 设置自动续期证书：
# 测试自动续期
sudo certbot renew --dry-run

# 查看现有的 crontab 任务
sudo crontab -l

# 添加自动续期任务（如果需要）
sudo crontab -e
# 添加以下行
0 0 1 * * certbot renew --quiet


重要提示：
确保您的域名 DNS 已经正确指向您的服务器 IP
确保服务器的 80 和 443 端口是开放的
如果使用防火墙，需要允许这些端口：

sudo ufw allow 80
sudo ufw allow 443

定期检查证书状态
sudo certbot certificates

如果需要更新证书
sudo certbot renew

如果遇到任何问题，可以查看 Nginx 的错误日志
sudo tail -f /var/log/nginx/error.log
















