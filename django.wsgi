import os
import os.path
import sys
import site

site.addsitedir('/var/www/html/fs/env/lib/python2.7/site-packages')

sys.path.append('/var/www/html/fs')
sys.path.append('/var/www/html/fs/main')

os.environ['PYTHON_EGG_CACHE'] = '/var/www/html/fs/egg_cache'
os.environ['DJANGO_SETTINGS_MODULE'] = 'main.settings'

# for Django <= 1.6
# import django.core.handlers.wsgi
# application = django.core.handlers.wsgi.WSGIHandler()

# for Django 1.8 and 1.7
from django.core.wsgi import get_wsgi_application
application = get_wsgi_application()