from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework import status
from rest_framework import authentication, permissions

from rest_framework.parsers import MultiPartParser, FormParser
import uuid, os
from django.shortcuts import HttpResponse, redirect
from .utils.converter import FileConverter
from main.apps.core.vars import RXFAX_DIR, TXFAX_DIR

class UploadFile(APIView):
	"""
## View To Manage Upload File in the system.
* Requires token authentication.
* Only users are able to access this view.
* Params: [filename].

Notice: This service accept only **multipart/form-data**
## CURL Example:

	curl -X POST http://127.0.0.1:8000/api/service/file/ \\
		 -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \\
		 -F "filename=@/full/path/to/img.jpg;type=image/jpg"

	curl -X POST http://127.0.0.1:8000/api/service/file/ \\
		 -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \\
		 -F "filename=@/full/path/to/audio.wav;type=audio/x-wav"

"""
	parser_classes = (MultiPartParser, FormParser,)
	authentication_classes = (authentication.TokenAuthentication,)
	permission_classes = (permissions.IsAuthenticatedOrReadOnly,)
	def get(self, request):
		ip_addr = request.META['REMOTE_ADDR']
		#print ip_addr
		return Response({"status": "OK", "code": 200}, status=status.HTTP_200_OK)
	def post(self, request, format=None):
		args = {}
		my_file = request.FILES['filename']
		name = str(uuid.uuid4())
		fn, fx  = os.path.splitext(unicode(my_file.name))
		new_filename = "%s%s%s" % (TXFAX_DIR, name, fx)
		filename = "%s%s" % (name, fx)
		with open(new_filename, 'wb+') as temp_file:
			for chunk in my_file.chunks():
				temp_file.write(chunk)

		# my_saved_file = open(filename)
		args['filename'] = filename
		args['status'] = 'OK'
		return Response(args, status=status.HTTP_201_CREATED)

# curl -X POST http://127.0.0.1:8000/api/voice/upload/ -H 'Authorization: Token 35552b5ac3ba375a5a00aa764c497cab20b61cea' -F "filename=@img.jpg;type=image/jpg"
# curl -X POST http://127.0.0.1:8000/api/voice/upload/ -H 'Authorization: Token 35552b5ac3ba375a5a00aa764c497cab20b61cea' -F "filename=@/home/tarek/s/rock.wav;type=audio/x-wav"

class DownloadFile(APIView):
	"""
## View To Manage Upload File in the system.
* Requires token authentication.
* Only users are able to access this view.
* Params: `filename`, `format`.
* Optional: `is_enhanced` boolean field, `directory` has `"rxfax", "txfax"`, default is `"txfax"`.

Notice: This service will send file to you
## CURL Example:

	curl -X POST http://127.0.0.1:8000/api/service/file/download/ \\
		 -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \\
		 -H 'Content-Type:application/json' \\
		 -d '{"filename": "ef26d474-6be3-4add-8b0f-bd850ac31db4.jpg", "format": "tiff"}'

"""
	authentication_classes = (authentication.TokenAuthentication,)
	permission_classes = (permissions.IsAuthenticatedOrReadOnly,)
	def get(self, request):
		ip_addr = request.META['REMOTE_ADDR']
		#print ip_addr
		return Response({"status": "OK", "code": 200}, status=status.HTTP_200_OK)
	def post(self, request):
		args = {}
		params = ['filename', 'format']
		if all(param in request.data for param in params):
			filename = request.data['filename']
			formati = request.data['format'].lower()
			is_enhanced = False
			if "is_enhanced" in request.data:
				is_enhanced = request.data['is_enhanced']
			directory = "txfax"
			if "directory" in request.data:
				directory = request.data["directory"]
				directory = directory if directory in ["txfax", "rxfax"] else "txfax"
			
			if directory == "rxfax":
				filename = "%s%s" % (RXFAX_DIR, filename)
			elif directory == "txfax":
				filename = "%s%s" % (TXFAX_DIR, filename)
			else:
				filename = "/tmp/%s" % filename

			if not os.path.isfile(filename):
				args["code"] = 404
				args["status"] = "File not found"
				return Response(args, status=status.HTTP_404_NOT_FOUND)
			elif formati in ["tiff", "tif"]:
				filename = FileConverter(filename=filename, is_enhanced=is_enhanced, DEBUG=False).convert()
				tiff = open(filename, 'rb')
				response = HttpResponse(tiff, content_type="image/tiff")
				response['Content-Disposition'] = 'attachment; filename=%s' % filename
				return response
			elif formati == "pdf":
				filename = FileConverter(filename=filename).convert2pdf()
				pdf = open(filename, 'rb')
				response = HttpResponse(pdf, content_type="application/pdf")
				response['Content-Disposition'] = 'attachment; filename=%s' % filename
				return response
			else:
				args["code"] = 400
				args['status'] = 'format does not defined'
				return Response(args, status=status.HTTP_400_BAD_REQUEST)
		else:
			args["code"] = 400
			args['status'] = 'Client Error'
			return Response(args, status=status.HTTP_400_BAD_REQUEST)
