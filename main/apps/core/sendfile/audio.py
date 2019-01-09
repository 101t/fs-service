import main.utils.esl.ESL as ESL
from ..vars import FREESWITCH_IP_ADDRESS, FREESWITCH_PORT, FREESWITCH_PASSWORD

class Audio(object):
	def __init__(self, name, username, file_path, numbers):
		self.username = username
		self.file_path = file_path
		self.numbers = numbers
		self.name = name

	def execute(self):
		body = []
		numbers_list = self.numbers.split(',')
		con = ESL.ESLconnection(FREESWITCH_IP_ADDRESS, FREESWITCH_PORT, FREESWITCH_PASSWORD)
		if con.connected():
			for number in numbers_list:
				xx = con.bgapi("originate", str("{ignore_early_media=true}sofia/gateway/%s/%s &playback(%s)" % (self.name, number, self.file_path)))
				if xx:
					# print xx.getInfo()
					# print str(xx.getHeader())
					print str(xx.getBody())
					print str(xx.getType())
					print str(xx.firstHeader())
					print str(xx.nextHeader())
					# body.append(xx.getBody())
		con.disconnect()
		return body
