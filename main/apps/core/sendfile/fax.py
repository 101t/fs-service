import main.utils.esl.ESL as ESL
from ..vars import FREESWITCH_IP_ADDRESS, FREESWITCH_PORT, FREESWITCH_PASSWORD
import os, uuid
from main.apps.service.models import CDRDetailed

class Fax(object):
	uuid = ""
	core_uuid = ""
	def __init__(self, name, username, file_path, numbers):
		self.username = username
		self.file_path = file_path
		self.numbers = numbers
		self.name = name
		self.numbers_list = self.numbers.split(',')

	def execute(self):
		args = {}
		listo = []
		con = ESL.ESLconnection(FREESWITCH_IP_ADDRESS, FREESWITCH_PORT, FREESWITCH_PASSWORD)
		if con.connected():
			self.uuid = str(uuid.uuid4())
			args["uuid"] = self.uuid
			for number in self.numbers_list:
				self.core_uuid = con.api("create_uuid").getBody()
				str_cmd = str("""{origination_uuid=%s,ignore_early_media=true,
								absolute_codec_string='PCMU,PCMA',
								fax_enable_t38=true,
								fax_verbose=true,
								fax_use_ecm=false,
								fax_enable_t38_request=false,
								fax_ident=%s}sofia/gateway/%s/%s &txfax(%s)""" % 
								(self.core_uuid, self.username, self.username, number, self.file_path))
				str_cmd = str_cmd.replace("\n", "")
				res = con.bgapi("originate", str_cmd)
				#self.uuid = res.getHeader("Job-UUID")
					# print res.getBody()
				dicto = {}
				dicto["cli"] = self.username
				dicto["cld"] = number
				dicto["file"] = os.path.basename(self.file_path)
				dicto["body"] = res.getBody()
				dicto["event-name"] = res.getHeader("Event-Name")
				dicto["job-uuid"] = self.core_uuid #res.getHeader("Job-UUID")
				cdr_d = CDRDetailed(
					cli=dicto["cli"],
					cld=dicto["cld"],
					file=dicto["file"],
					body=dicto["body"],
					job_uuid=dicto["job-uuid"],
					uuid=self.uuid,
					event_name=dicto["event-name"],
				)
				cdr_d.save(using="cdr-pusher")
				listo.append(dicto)
				#con.api("uuid_kill", self.uuid) # this will kill all the hole originate channel
			args["details"] = listo
		con.disconnect()
		return args

"""
import ESL
con = ESL.ESLconnection("127.0.0.1", "8021", "ClueCon")
con.connected()
res = con.bgapi("originate", "{ignore_early_media=true,fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/908504300003/908504300004 &txfax(/tmp/224e2d06-9553-4c46-a053-974a1aa2dfba.tiff)")
"""

"""
===Example===
An example to start a fax from the command line:
originate {ignore_early_media=true,absolute_codec_string='PCMU,PCMA',fax_enable_t38=true,fax_verbose=true,fax_use_ecm=true,fax_enable_t38_request=true}sofia/gateway/default/1231231234 &txfax('test_fax.tif')
And if this fails further retries with:
originate {ignore_early_media=true,absolute_codec_string='PCMU,PCMA',fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/default/1231231234 &txfax('test_fax.tif')

for more information about Event header and body
https://wiki.freeswitch.org/wiki/Event_List
for more information about sched_api
https://freeswitch.org/confluence/display/FREESWITCH/mod_commands
"""