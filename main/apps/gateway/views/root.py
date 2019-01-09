from rest_framework.decorators import api_view
from rest_framework.response import Response
from rest_framework.reverse import reverse
from rest_framework.views import APIView
#from django.core.urlresolvers import reverse
from rest_framework.decorators import api_view, permission_classes
from rest_framework import authentication, permissions

class APIGatewayView(APIView):
	permission_classes = (permissions.AllowAny,)
	def get(self, request, *args, **kw):
		data = super(APIGatewayView, self)
		return Response({
			'Operation': reverse('operation', request=request),
			'Check': reverse('check', request=request),
			})