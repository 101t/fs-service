from ivr import IVR
from xml.etree import ElementTree as ET
from ..vars import IVR_DIR , GATEWAY_PREFIX, FREESWITCH_IP_ADDRESS, FREESWITCH_PORT, FREESWITCH_PASSWORD
import time
import datetime
import main.utils.esl.ESL as ESL

class IVRManager(object):

	def __init__(self, service, username, ivr_list_dic, numbers):
		self.service = service
		self.username = username
		self.ivr_list_dic = ivr_list_dic
		self.numbers = numbers
		self.name = GATEWAY_PREFIX + self.username

	def indent(self, elem, level=0):
		i = "\n" + level*"  "
		if len(elem):
			if not elem.text or not elem.text.strip():
				elem.text = i + "  "
			if not elem.tail or not elem.tail.strip():
				elem.tail = i
			for elem in elem:
				self.indent(elem, level+1)
			if not elem.tail or not elem.tail.strip():
				elem.tail = i
		else:
			if level and (not elem.tail or not elem.tail.strip()):
				elem.tail = i

	def execute(self):
		file_path = self.write_xml_ivr()
		self.reload_xml()
		numbers_list = []
		numbers_list = self.numbers.split(',')
		con = ESL.ESLconnection(FREESWITCH_IP_ADDRESS, FREESWITCH_PORT, FREESWITCH_PASSWORD)
		if con.connected():
			for number in numbers_list:
				xx = con.bgapi("originate", str("sofia/gateway/%s/%s &ivr(%s)" % (self.name, number, self.ivr_list_dic[0]["name"])))
				if xx:
					print xx.getBody()
		con.disconnect()

	def write_xml_ivr(self):
		root = ET.Element("include")
		for menu in self.ivr_list_dic:
			actions = []
			actions_string = []
			actions_string = menu["entry"].split(',')
			for x in actions_string:
				tmp_list = []
				tmp_list = x.split(':')
				tmp_dic = {}
				tmp_dic["action"] = tmp_list[0]
				tmp_dic["digits"] = tmp_list[1]
				tmp_dic["params"] = tmp_list[2]
				actions.append(tmp_dic)
			tmp_ivr_menu = IVR(menu["name"], menu["greet_long"], menu["greet_short"], menu["invalid_sound"], menu["exit_sound"], menu["confirm_macro"], menu["confirm_key"], menu["tts_engine"], menu["tts_voice"], menu["confirm_attempts"], menu["timeout"], menu["inter_digit_timeout"], menu["max_failures"], menu["max_timeouts"], menu["digit_len"], actions)
			tmp_ivr_menu.write(root)
		ts = time.time()
		st = datetime.datetime.fromtimestamp(ts).strftime('%Y%m%d%H%M%S')
		file_name = self.username + "IVR" + st + ".xml"
		self.indent(root)
		tree = ET.ElementTree(root)
		file_path = "%s%s" % (IVR_DIR, file_name)
		tree.write(file_path)
		return file_path

	def reload_xml(self):
		con = ESL.ESLconnection(FREESWITCH_IP_ADDRESS, FREESWITCH_PORT, FREESWITCH_PASSWORD)
		if con.connected():
			xx = con.api("reloadxml")
			if xx:
				print xx.getBody()
		con.disconnect()