from django.contrib.admin import AdminSite
from django.utils.translation import ugettext_lazy
from django.contrib import admin
from compat import format_html
from django.core import urlresolvers
class AdminSiteTitles(AdminSite):
	# Text to put at the end of each page's <title>.
	site_title = ugettext_lazy('FreeSWITCH site admin')

	# Text to put in each page's <h1>.
	site_header = ugettext_lazy('FreeSWITCH Service administration')

	# Text to put at the top of the admin index page.
	index_title = ugettext_lazy('Dashboard administration')

# admin.site = AdminSiteTitles()
admin.site.site_title = format_html('FreeSWITCH site admin')
admin.site.site_header = format_html('<a href="' + urlresolvers.reverse('admin:index') + '"><span style="font-size:18px">FreeSWITCH <b style="font-size:18px">Service</b></a> administration</span>')
admin.site.index_title = format_html('Dashboard Administration')