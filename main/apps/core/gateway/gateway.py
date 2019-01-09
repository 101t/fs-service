from xml.etree import ElementTree as ET
from xml.dom import minidom
from lxml import objectify
import os

class GateWay(object):

	#file_path = "/usr/local/freeswitch/conf/sip_profiles/external/company.xml"
	path = "/usr/local/freeswitch/conf/sip_profiles/external/"

	def __init__(self, username, password, host, register):
		self.username = username
		self.password = password
		self.host = host
		self.register = register
		self.file_path = "%s%s.xml" % (self.path, self.username)
		self.file_exist = True if os.path.exists(self.file_path) else False
		if self.file_exist:
			self.tree = ET.parse(self.file_path)
			self.root = self.tree.getroot()
		else:
			self.tree = None#ET.parse(self.file_path)
			self.root = None #self.tree.getroot()

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

	def add_new1(self):
		gate_way = ET.SubElement(self.root, "gateway")
		gate_way.set("name", self.username)
		param1 = ET.SubElement(gate_way, "param")
		param1.set("name", "username")
		param1.set("value", self.username)
		param2 = ET.SubElement(gate_way, "param")
		param2.set("name", "password")
		param2.set("value", self.password)
		param3 = ET.SubElement(gate_way, "param")
		param3.set("name", "realm")
		param3.set("value", self.host)
		param4 = ET.SubElement(gate_way, "param")
		param4.set("name", "register")
		param4.set("value", self.register)
		self.indent(self.root)
		self.tree.write(self.file_path)
	def add_new(self):
		open(self.file_path, 'a').close()
		include = ET.Element("include")
		gateway = ET.SubElement(include, "gateway")
		gateway.set("name", self.username)
		param1 = ET.SubElement(gateway, "param")
		param1.set("name", "username")
		param1.set("value", self.username)
		param2 = ET.SubElement(gateway, "param")
		param2.set("name", "password")
		param2.set("value", self.password)
		param3 = ET.SubElement(gateway, "param")
		param3.set("name", "realm")
		param3.set("value", self.host)
		param4 = ET.SubElement(gateway, "param")
		param4.set("name", "register")
		param4.set("value", self.register)
		self.indent(include)
		self.tree = ET.ElementTree(include)
		self.tree.write(self.file_path)
	def delete(self):
		if self.file_exist:
			os.remove(self.file_path)
		# 	for tmp_gateway in self.root.findall('gateway'):
		# 		if tmp_gateway.get('name') == self.username:
		# 			self.root.remove(tmp_gateway)
		# 	self.tree.write(self.file_path)

	def modify_existed(self):
		if self.file_exist:
			self.delete()
			self.add_new()

	def check(self):
		args = {}
		if self.file_exist:
			result = {}
			for tmp_gateway in self.root.findall('gateway'):
				if tmp_gateway.get('name') == self.username:
					for tmp_param in tmp_gateway.iter(tag="param"):
						result[tmp_param.get("name")] = tmp_param.get("value")
			args["status"] = "Ok"
			args["code"] = 200
			args["body"] = result
		else:
			args['status'] = "Not Found"
			args["code"] = 404
		#print result
		return args

"""
<include>
  <gateway name="908504300003">
    <param name="username" value="908504300003"/>
    <param name="password" value="3t4ar3i4k"/>
    <param name="realm" value="91.244.227.228"/>
    <param name="proxy" value="91.244.227.228"/>
    <param name="register" value="true"/>
  </gateway>
</include>
"""