from main.apps.service.models import CDRImport, CDRDetailed
import os
import datetime as dt
from main.apps.core.vars import RXFAX_DIR
class GeneralInbox(object):
	def __init__(self, start_date, end_date, receivers):
		self.start_date = dt.datetime.strptime(start_date, "%Y-%m-%d %H:%M")
		self.end_date = dt.datetime.strptime(end_date, "%Y-%m-%d %H:%M")
		self.receivers = receivers
		self.fax_filename = ""
	def file_exist(self, uuid):
		self.fax_filename = "rxfax-%s.tiff" % uuid
		file_path = "%s%s" % (RXFAX_DIR, self.fax_filename)
		return os.path.exists(file_path)
	def getdata(self):
		listo = []
		cdr_imp = CDRImport.objects.using("cdr-pusher").order_by("-starting_date")\
			.filter(starting_date__range=(self.start_date, self.end_date))\
			.filter(destination_number__in=self.receivers)
		for cdr in cdr_imp:
			dicto = {}
			dicto["cli"] = cdr.caller_id_number
			dicto["cld"] = cdr.destination_number
			dicto["res_id"] = cdr.callid
			dicto["fax_filename"] = self.fax_filename if self.file_exist(cdr.callid) else ""
			dicto["datetime"] = cdr.starting_date.strftime("%Y-%m-%d %H:%M:%S")
			listo.append(dicto)
		return listo

