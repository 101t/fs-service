from xml.etree import ElementTree as ET
from .gateway import GateWay
import main.utils.esl.ESL as ESL
from ..vars import FREESWITCH_IP_ADDRESS, FREESWITCH_PORT, FREESWITCH_PASSWORD, GATEWAY_PREFIX

class GatewayManager(object):
	def __init__(self, service="", username="", password="", host="", register=False):
		self.service = service
		self.username = username
		self.password = password
		self.host = host
		self.register = register

	def execute(self):
		body = None
		name = GATEWAY_PREFIX + self.username
		con = ESL.ESLconnection(FREESWITCH_IP_ADDRESS, FREESWITCH_PORT, FREESWITCH_PASSWORD)
		gateway = GateWay(self.username, self.password, self.host, self.register)
		if self.service == "new":
			gateway.add_new()
			if con.connected():
				xx = con.api("sofia", str("profile external killgw %s reloadxml" % name))
				xx = con.api("sofia profile external rescan reloadxml")
				if xx:
					# print xx.getBody()
					body = xx.getBody()
		elif self.service == "edit":
			gateway.modify_existed()
			if con.connected():
				xx = con.api("sofia", str("profile external killgw %s reloadxml" % name))
				xx = con.api("sofia profile external rescan reloadxml")
				if xx:
					#print xx.getBody()
					body = xx.getBody()
		elif self.service == "delete":
			gateway.delete()
			if con.connected():
				xx = con.api("sofia", str("profile external killgw %s reloadxml" % name))
				xx = con.api("sofia profile external rescan reloadxml")
				if xx:
					# print xx.getBody()
					body = xx.getBody()
		con.disconnect()
		return body
	def check(self):
		gateway = GateWay(self.username, "", "", "")
		return gateway.check()