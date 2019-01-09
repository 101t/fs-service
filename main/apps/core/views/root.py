from rest_framework.decorators import api_view
from rest_framework.response import Response
from rest_framework.reverse import reverse
from rest_framework.views import APIView
#from django.core.urlresolvers import reverse
from rest_framework.decorators import api_view, permission_classes
from rest_framework import authentication, permissions

class APIRootView(APIView):
	"""
### Welcome To FreeSWITCH Service API
* Setting Up Your Gateway
* Originate Voice Message and Fax
* Before Originate Upload Your File
"""
	permission_classes = (permissions.AllowAny,)
	def get(self, request, *args, **kw):
		data = super(APIRootView, self)
		return Response({
			'Gateway': reverse('gateway', request=request),
			'Service': reverse('service', request=request),
			})
	def get_view_name(self):
		data = super(APIRootView, self).get_view_name()
		# print data
		return "API"