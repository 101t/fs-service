from main.apps.core.sendfile.manager import Manager
from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework import status
from rest_framework import authentication, permissions
import os
from .utils.report import GeneralReport
class FaxReport(APIView):
	"""
## View to Originate Call in the system.

* Requires token authentication.
* Only admin users are able to access this view.
* Params: `uuid`.
## CURL Example:

	curl -X POST http://127.0.0.1:8000/api/service/fax/report/ \\
		 -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \\
		 -H 'Content-Type:application/json' \\
		 -d '{"uuid": "72df3bef-ec07-4968-9959-bd6a70a2e8c8"}'
"""
	authentication_classes = (authentication.TokenAuthentication,)
	permission_classes = (permissions.IsAuthenticatedOrReadOnly,) # permissions.IsAdminUser
	
	def get(self, request, *args, **kw):
		return Response({"status": "OK", "code": 200}, status=status.HTTP_200_OK)
	def post(self, request, *args, **kw):
		# print request.user
		# print request.auth
		args = {}
		params = ['uuid']
		if all(param in request.data for param in params):
			uuid = str(request.data["uuid"]).strip()
			args["body"] = GeneralReport(uuid=uuid).getdata()
			args["code"] = 200
			args["status"] = "OK"
			return Response(args, status=status.HTTP_200_OK)
		else:
			args["body"] = None
			args["code"] = 400
			args['status'] = 'Client Error'
			return Response(args, status=status.HTTP_400_BAD_REQUEST)