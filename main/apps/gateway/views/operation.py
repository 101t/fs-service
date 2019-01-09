from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework import status
from rest_framework import authentication, permissions
from main.apps.core.gateway.manager import GatewayManager


class GatewayOperation(APIView):
	"""
##Configure and Manage Gateway in the FreeSWITCH.

* Requires token authentication.
* Only users are able to access this view.
* Params: [service, username, password, host, register].

Notice 'service' take ("new", "edit", "delete"), 'register' (true, false as default)
## CURL Example:

	curl -X POST http://127.0.0.1:8000/api/gateway/operation/ \\
		 -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \\
		 -H 'Content-Type:application/json' \\
		 -d '{"service": "new", "username": "908599999999", "password": "XXXXXXX", "host": "sip.domainprovider.com", "register": false}'
"""
	authentication_classes = (authentication.TokenAuthentication,)
	permission_classes = (permissions.IsAuthenticatedOrReadOnly,)
	def get(self, request):
		ip_addr = request.META['REMOTE_ADDR']
		#print ip_addr
		return Response({"status": "OK", "code": 200}, status=status.HTTP_200_OK)
	def post(self, request, format='json'):
		args = {}
		params = ['service', 'username', 'password', 'host', 'register']
		# print request.data
		if all(param in params for param in request.data):
			
			service = request.data['service']
			username = request.data['username']
			password = request.data['password']
			host = request.data['host']
			register = 'true' if request.data['register'] else 'false'

			gm = GatewayManager(service, username, password, host, register)
			body = gm.execute()
			args['status'] = 'OK'
			args["code"] = 200
			args['body'] = body
			return Response(args, status=status.HTTP_200_OK)
		else:
			args["code"] = 400
			args['status'] = 'Client Error'
			return Response(args, status=status.HTTP_400_BAD_REQUEST)

# curl -X POST http://127.0.0.1:8000/api/gateway/managemet/ -H 'Authorization: Token 35552b5ac3ba375a5a00aa764c497cab20b61cea' -H 'Content-Type:application/json' -d '{"service": "new", "username": "908504300003", "password": "34tarek34", "host": "sip.pasifiktelekom.com.tr", "register": false}'