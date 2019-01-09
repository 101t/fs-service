from main.apps.core.sendfile.manager import Manager
from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework import status
from rest_framework import authentication, permissions
import os
from .utils.inbox import GeneralInbox
class FaxInbox(APIView):
	"""
## View to FaxInbox in the system.

* Requires token authentication.
* Only admin users are able to access this view.
* Params: `start_date`, `end_date`, `receivers`.
* `start_date`, `end_date` should be string as `"%Y-%m-%d %H:%M"` format.
* `receivers` should be list such as ["90850XXXXXXX", "90850XXXXXXX"].
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
		args = {}
		params = ['start_date', 'end_date', "receivers"]
		if all(param in request.data for param in params):
			start_date = request.data["start_date"]
			end_date = request.data["end_date"]
			receivers = request.data["receivers"]
			args["body"] = GeneralInbox(start_date=start_date, end_date=end_date, receivers=receivers).getdata()
			args["code"] = 200
			args["status"] = "OK"
			return Response(args, status=status.HTTP_200_OK)
		else:
			args["body"] = None
			args["code"] = 400
			args['status'] = 'Client Error'
			return Response(args, status=status.HTTP_400_BAD_REQUEST)