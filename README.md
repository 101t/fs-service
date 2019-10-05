FS - SERVICE
============

A Simple **FreeSWITCH REST API** built with django and python2, sending and receiving **FAX**, sending **Voice Message**.

## Requirements

1. First of all you must install FreeSWITCH
2. You also need to install [cdr-pusher](https://github.com/areski/cdr-pusher) that built with GoLang, it is important to grap cdr logs and save it in fax inbox table, do not forget to configure postgresql database as secondary db in django `settings.py`:
```python
DATABASES = {
    'default': env.db('DEFAULTDB_URL'),
    'cdr-pusher': env.db('CDRPUSHERDB_URL'),
}
````
Copy `Sample.env` as `.env` in the project directory
```sh
cp Sample.env .env
```
You can modify these variables in `.env` file as
```
DEFAULTDB_URL=sqlite:///db.sqlite3
CDRPUSHERDB_URL=postgres://postgres:123@127.0.0.1:5432/cdr-pusher
```
3. Install third-party dependencies such as `LibreOffice`, `ImageMagick`, `swig` for ESL, `python-lxml` by run: 

```sh
sh ./dep-requiremets.sh
```

4. Finally you may install django project on your host:
	
	1. Download and Extract folder We recommend installing in a virtualenv by: 
	```sh 
	virtualenv -p python27 env && source env/bin/activate
	```
	2. Install django dependencies:
	```sh 
	pip install -r requirements.pip
	```
	3. cd to `fs-service` and run:
	```sh
	# Initialize DB
	python manage.py migrate
	# Create Superuser
	python manage.py createsuperuser
	# collect static files
	python manage.py collectstatic
	# Run API service
	python manage.py runserver 8000
	```

after installation complete navigate to `127.0.0.1:8000` in your browser.

## Contribute
Special Thanks to [Areski Belaid](https://github.com/areski) and [FreeSWITCH Community](https://freeswitch.org).