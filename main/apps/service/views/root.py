from rest_framework.decorators import api_view
from rest_framework.response import Response
from rest_framework.reverse import reverse
from rest_framework.views import APIView
#from django.core.urlresolvers import reverse
from rest_framework.decorators import api_view, permission_classes
from rest_framework import authentication, permissions
class APIVoiceView(APIView):
	permission_classes = (permissions.AllowAny,)
	def get(self, request, *args, **kw):
		data = super(APIVoiceView, self)
		return Response({
			'Originate': reverse('voice_originate', request=request),
		})
class APIFileView(APIView):
	permission_classes = (permissions.AllowAny,)
	def get(self, request, *args, **kw):
		data = super(APIFileView, self)
		return Response({
			'Upload': reverse('file_upload', request=request),
			'Download': reverse('file_download', request=request),
		})
class APIFaxView(APIView):
	permission_classes = (permissions.AllowAny,)
	def get(self, request, *args, **kw):
		data = super(APIFaxView, self)
		return Response({
			'Originate': reverse('fax_originate', request=request),
			'Report': reverse('fax_report', request=request),
			'Inbox': reverse('fax_inbox', request=request),
		})
class APIServiceView(APIView):
	permission_classes = (permissions.AllowAny,)
	def get(self, request, *args, **kw):
		data = super(APIServiceView, self)
		return Response({
			'File': reverse('file', request=request),
			'Voice': reverse('voice', request=request),
			'Fax': reverse('fax', request=request),
		})