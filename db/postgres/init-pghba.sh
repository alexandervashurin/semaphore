#!/bin/sh
# Настройка pg_hba.conf для разрешения подключений с хоста

echo "host    all             all             0.0.0.0/0               md5" >> /var/lib/postgresql/data/pg_hba.conf
echo "host    all             all             172.16.0.0/12           md5" >> /var/lib/postgresql/data/pg_hba.conf

# Перезагружаем конфигурацию PostgreSQL
pg_ctl reload -D /var/lib/postgresql/data

echo "pg_hba.conf настроен для подключения с хоста"
