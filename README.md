FS - SERVICE
============

FreeSWITCH REST API built with django and python2, sending and receiving FAX, sending Voice Message.

## Requirements

1. First of all you must install FreeSWITCH
2. You also need to install [cdr-pusher](https://github.com/areski/cdr-pusher) that built with GoLang, it is important to grap cdr logs and save it in fax inbox table, do not forget to configure postgresql database as secondary db in django `settings.py` 
3. Install third-party dependencies such as `LibreOffice`, `ImageMagick`, `swig` for ESL, `python-lxml` by run: 

```sh
sh dep-requiremets.sh
```
4. Finally you may install django project on your host.


## Contribute
Special Thanks to [Areski Belaid](https://github.com/areski) and [FreeSWITCH Community](https://freeswitch.org).