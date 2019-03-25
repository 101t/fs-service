### Install and Configure MySQL with FreeSWITCH

1. complete installation mysql db
```sh
apt-get install mysql-server mysql-client
```

2. install mysql driver
```sh
apt-get install libmyodbc unixodbc-bin
```

3. [Skipe it] search install libs
```sh
find / -name 'lib*odbc*.so'
```

4. for the folks working on 64 bit odbcinst.ini looks like
```
[MySQL]
Description = ODBC for MySQL
Driver = /usr/lib/x86_64-linux-gnu/odbc/libmyodbc.so
Setup = /usr/lib/x86_64-linux-gnu/odbc/libodbcmyS.so
FileUsage = 1
```

5. after creating database to test connection
```sh
apt-get install unixodbc unixodbc-dev
isql -v freeswitch
```

6. link ini files to FreeSWITCH Directory
```sh
mkdir /usr/local/freeswitch/etc
ln -s /etc/odbcinst.ini /usr/local/freeswitch/etc/odbcinst.ini
ln -s /etc/odbc.ini /usr/local/freeswitch/etc/odbc.ini
```