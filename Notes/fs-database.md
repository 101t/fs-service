

# complete installation mysql db
apt-get install mysql-server mysql-client

# install mysql driver
apt-get install libmyodbc unixodbc-bin


# [Skipe it] search install libs
find / -name 'lib*odbc*.so'

#for the folks working on 64 bit odbcinst.ini looks like
[MySQL]
Description = ODBC for MySQL
Driver = /usr/lib/x86_64-linux-gnu/odbc/libmyodbc.so
Setup = /usr/lib/x86_64-linux-gnu/odbc/libodbcmyS.so
FileUsage = 1

# after creating database to test connection
apt-get install unixodbc unixodbc-dev
isql -v freeswitch

# link ini files to FreeSWITCH Directory
mkdir /usr/local/freeswitch/etc
ln -s /etc/odbcinst.ini /usr/local/freeswitch/etc/odbcinst.ini
ln -s /etc/odbc.ini /usr/local/freeswitch/etc/odbc.ini