from django.db import models
from main.apps.core.models import TimeStampedModel
from main.apps.core.applabel import string_with_title

class CDRImport(models.Model):
	switch = models.CharField(max_length=80)
	cdr_source_type = models.IntegerField(blank=True, null=True)
	callid = models.CharField(max_length=80)
	caller_id_number = models.CharField(max_length=80)
	caller_id_name = models.CharField(max_length=80)
	destination_number = models.CharField(max_length=80)
	dialcode = models.CharField(max_length=10, blank=True, null=True)
	state = models.CharField(max_length=5, blank=True, null=True)
	channel = models.CharField(max_length=80, blank=True, null=True)
	starting_date = models.DateTimeField()
	duration = models.IntegerField()
	billsec = models.IntegerField()
	progresssec = models.IntegerField(blank=True, null=True)
	answersec = models.IntegerField(blank=True, null=True)
	waitsec = models.IntegerField(blank=True, null=True)
	hangup_cause_id = models.IntegerField(blank=True, null=True)
	hangup_cause = models.CharField(max_length=80, blank=True, null=True)
	direction = models.IntegerField(blank=True, null=True)
	country_code = models.CharField(max_length=3, blank=True, null=True)
	accountcode = models.CharField(max_length=40, blank=True, null=True)
	buy_rate = models.DecimalField(max_digits=10, decimal_places=5, blank=True, null=True)
	buy_cost = models.DecimalField(max_digits=12, decimal_places=5, blank=True, null=True)
	sell_rate = models.DecimalField(max_digits=10, decimal_places=5, blank=True, null=True)
	sell_cost = models.DecimalField(max_digits=12, decimal_places=5, blank=True, null=True)
	extradata = models.TextField(blank=True, null=True)  # This field type is a guess.

	class Meta:
		managed = False
		verbose_name='CDR General'
		verbose_name_plural = verbose_name
		db_table = 'cdr_import'
		#app_label = string_with_title("service", verbose_name)
		app_label = 'service'
		ordering = ['-starting_date']
# Create your models here.
class CDRDetailed(TimeStampedModel):
	class Meta:
		db_table = "cdr_detailed"
		verbose_name='CDR Detailed'
		#app_label = string_with_title("service", verbose_name)
		app_label = 'service'
		verbose_name_plural = verbose_name
		managed = True
	uuid = models.CharField(max_length=80)
	job_uuid = models.CharField(max_length=80)
	body = models.TextField(blank=True, null=True)
	cli = models.CharField(max_length=80)
	cld = models.CharField(max_length=80)
	event_name = models.CharField(max_length=80)
	file = models.CharField(max_length=80)

"""
To define this model on PostgresSQL database you shoud do:
./manage.py makemigrations service
./manage.py migrate service --database=cdr-pusher
"""