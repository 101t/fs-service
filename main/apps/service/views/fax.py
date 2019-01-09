from main.apps.core.sendfile.manager import Manager
from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework import status
from rest_framework import authentication, permissions
import os
from .utils.converter import FileConverter
from main.apps.core.vars import RXFAX_DIR, TXFAX_DIR

class OriginateFax(APIView):
	"""
## View to Originate Call in the system.

* Requires token authentication.
* Only admin users are able to access this view.
* Params: `username`, `filename`, `numbers`.
* Optional: `is_enhanced` boolean field.
## CURL Example:

	curl -X POST http://127.0.0.1:8000/api/service/fax/ \\
		 -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \\
		 -H 'Content-Type:application/json' \\
		 -d '{"username": "9085XXXXXXXX", "filename": "72df3bef-ec07-4968-9959-bd6a70a2e8c8.doc", "numbers": "05XXXXXXXXX"}'
"""
	authentication_classes = (authentication.TokenAuthentication,)
	permission_classes = (permissions.IsAuthenticatedOrReadOnly,) # permissions.IsAdminUser
	
	def get(self, request, *args, **kw):
		return Response({"status": "OK", "code": 200}, status=status.HTTP_200_OK)

	def post(self, request, *args, **kw):
		# print request.user
		# print request.auth
		args = {}
		params = ['username', 'filename', 'numbers']
		if all(param in request.data for param in params):
			username = request.data['username']
			filename = "%s%s" % (TXFAX_DIR, request.data['filename'])
			is_enhanced = False
			if "is_enhanced" in request.data:
				is_enhanced = True if request.data["is_enhanced"] else False
			numbers = request.data['numbers']
			if not os.path.isfile(filename):
				args["code"] = 404
				args["status"] = "File not found"
				return Response(args, status=status.HTTP_404_NOT_FOUND)
			else:
				filename = FileConverter(filename=filename, is_enhanced=is_enhanced, DEBUG=False).convert()
			m = Manager(service="fax", username=username, file_path=filename, numbers=numbers)
			body = m.execute()
			args['status'] = 'OK'
			args["code"] = 200
			args['body'] = body
			return Response(args, status=status.HTTP_200_OK)
		else:
			args["code"] = 400
			args['status'] = 'Client Error'
			return Response(args, status=status.HTTP_400_BAD_REQUEST)

# curl -X POST http://127.0.0.1:8000/api/service/fax/ -H 'Authorization: Token 35552b5ac3ba375a5a00aa764c497cab20b61cea' -H 'Content-Type:application/json' -d '{"username": "908504300003", "filename": "72df3bef-ec07-4968-9959-bd6a70a2e8c8.doc", "numbers": "05312089995"}'

"""
#RESPONSE
{
    "status": "OK",
    "body": {
        "uuid": "315ded86-99e9-11e6-88e6-c7aaf2c109a7",
        "details": [{
            "body": null,
            "cli": "908504300003",
            "cld": "902127010339",
            "event-name": "SOCKET_DATA",
            "job-uuid": "315dfd94-99e9-11e6-88e7-c7aaf2c109a7",
            "file": "36e7b256-e441-4ec4-a7cf-9c2323c82f96.tiff"
        }]
    },
    "code": 200
}
"""