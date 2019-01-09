from main.apps.service.models import CDRImport, CDRDetailed
class GeneralReport(object):
	def __init__(self, uuid):
		self.uuid = uuid
	def getdata(self):
		args = {}
		count = 0
		delivered = 0
		undelivered = 0
		sender = ""
		cdr_det = CDRDetailed.objects.using("cdr-pusher").filter(uuid=self.uuid)
		for cdr_d in cdr_det:
			cdr_imp = CDRImport.objects.using("cdr-pusher").filter(callid=cdr_d.job_uuid).first()
			if cdr_imp:
				count += 1
				if cdr_imp.duration > 0:
					delivered += 1
				else:
					undelivered += 1
				sender = cdr_d.cli
		args["count"] = count
		args["delivered"] = delivered
		args["undelivered"] = undelivered
		args["uuid"] = self.uuid
		args["sender"] = sender
		return args