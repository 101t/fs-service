from main.apps.core.sendfile.manager import Manager
from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework import status
from rest_framework import authentication, permissions

class OriginateVoice(APIView):
	"""
## View to Originate Call in the system.

* Requires token authentication.
* Only admin users are able to access this view.
* Params: [username, filename, numbers].
## CURL Example:

	curl -X POST http://127.0.0.1:8000/api/voice/ \\
		 -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \\
		 -H 'Content-Type:application/json' \\
		 -d '{"username": "9085XXXXXXXX", "filename": "72df3bef-ec07-4968-9959-bd6a70a2e8c8.wav", "numbers": "05XXXXXXXXX"}'
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
			filename = request.data['filename']
			numbers = request.data['numbers']
			m = Manager(service="audio", username=username, file_path=filename, numbers=numbers)
			body = m.execute()
			args['status'] = 'OK'
			args['body'] = body
			return Response(args, status=status.HTTP_200_OK)
		else:
			args['status'] = 'Client Error'
			return Response(args, status=status.HTTP_400_BAD_REQUEST)

# curl -X POST http://127.0.0.1:8000/api/voice/ -H 'Authorization: Token 35552b5ac3ba375a5a00aa764c497cab20b61cea' -H 'Content-Type:application/json' -d '{"username": "908509999999", "filename": "72df3bef-ec07-4968-9959-bd6a70a2e8c8.wav", "numbers": "05319999999"}'