from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework import status
from rest_framework import authentication, permissions
from main.apps.core.gateway.manager import GatewayManager

class GatewayCheck(APIView):
	"""
### Check Status Gateway in the FreeSWITCH
* Requires token authentication.
* Only users are able to access this view.
* Params: [username].

## CURL Example:

	curl -X POST http://127.0.0.1:8000/api/gateway/check/ \\
		 -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \\
		 -H 'Content-Type:application/json' -d '{"username": "908599999999"}'
"""
	authentication_classes = (authentication.TokenAuthentication,)
	permission_classes = (permissions.IsAuthenticatedOrReadOnly,)
	def get(self, request):
		ip_addr = request.META['REMOTE_ADDR']
		#print ip_addr
		return Response({"status": "OK", "code": 200}, status=status.HTTP_200_OK)
	def post(self, request): # , format='json'
		args = {}
		params = ['username']
		# print request.data
		if all(param in params for param in request.data):
			username = request.data['username']
			gm = GatewayManager(username=username)
			args = gm.check()
			return Response(args, status=status.HTTP_200_OK)
		else:
			args['status'] = 'Bad Request'
			args['code'] = 400
			return Response(args, status=status.HTTP_400_BAD_REQUEST)

# curl -X POST http://127.0.0.1:8001/api/gateway/check/ -H 'Authorization: Token 35552b5ac3ba375a5a00aa764c497cab20b61cea' -H 'Content-Type:application/json' -d '{"username": "908504300004"}'