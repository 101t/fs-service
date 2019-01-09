from django.contrib import admin
from .models import CDRDetailed, CDRImport

# Register your models here.

class CDRImportAdmin(admin.ModelAdmin):
	date_hierarchy = 'starting_date'
	list_display = ("callid", "caller_id_number", "caller_id_name", "destination_number", "starting_date", "duration", "billsec", "hangup_cause",)
	using = 'cdr-pusher'
	search_fields = ['callid', 'caller_id_number', 'caller_id_number', 'destination_number',]

	def has_add_permission(self, request):
		return False
	def save_model(self, request, obj, form, change):
		# Tell Django to save objects to the 'other' database.
		obj.save(using=self.using)
	def delete_model(self, request, obj):
		# Tell Django to delete objects from the 'other' database
		obj.delete(using=self.using)
	def get_queryset(self, request):
		# Tell Django to look for objects on the 'other' database.
		return super(CDRImportAdmin, self).get_queryset(request).using(self.using)

class CDRDetailedAdmin(admin.ModelAdmin):
	date_hierarchy = 'created'
	list_display = ("uuid", "job_uuid", "cli", "cld", "event_name", "file",)
	search_fields = ["uuid", "job_uuid", "cli", "cld", "file"]
	using = 'cdr-pusher'
	def save_model(self, request, obj, form, change):
		# Tell Django to save objects to the 'other' database.
		obj.save(using=self.using)
	def delete_model(self, request, obj):
		# Tell Django to delete objects from the 'other' database
		obj.delete(using=self.using)
	def get_queryset(self, request):
		# Tell Django to look for objects on the 'other' database.
		return super(CDRDetailedAdmin, self).get_queryset(request).using(self.using)

admin.site.register(CDRImport, CDRImportAdmin)
admin.site.register(CDRDetailed, CDRDetailedAdmin)