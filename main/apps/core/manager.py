from gateway.gateway import GatewayManager
from sendfile.manager import Manager
from ivr.manager import IVRManager

class Manager(object):

	def __init__(self):
		pass

	def gateway_managment(self, service, username, password, host, register):
		gateway_manager = GatewayManager(service, username, password, host, register)
		gateway_manager.execute()

	def send_file(self, service, username, file_path, numbers):
		file_manager = FileManager(service, username, file_path, numbers)
		file_manager.execute()

	def ivr(self, service, username ,ivr_list_dic, numbers):
		ivr_manager = IVRManager(service, username, ivr_list_dic, numbers)
		ivr_manager.execute()
