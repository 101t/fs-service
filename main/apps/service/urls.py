from django.urls import include
from django.conf.urls import url
from .views.voice import OriginateVoice
from .views.fax import OriginateFax
from .views.fax_report import FaxReport
from .views.fax_inbox import FaxInbox
from .views.file import UploadFile, DownloadFile
from .views.root import APIServiceView, APIVoiceView, APIFileView, APIFaxView

urlpatterns = (
	url(regex=r'^$', view=APIServiceView.as_view(), name="service"),
	url(regex=r'^voice/$', view=APIVoiceView.as_view(), name="voice"),
	url(regex=r'^voice/originate/$', view=OriginateVoice.as_view(), name="voice_originate"),
	
	url(regex=r'^file/$', view=APIFileView.as_view(), name="file"),
	url(regex=r'^file/upload/$', view=UploadFile.as_view(), name="file_upload"),
	url(regex=r'^file/download/$', view=DownloadFile.as_view(), name="file_download"),

	url(regex=r'^fax/$', view=APIFaxView.as_view(), name="fax"),
	url(regex=r'^fax/originate/$', view=OriginateFax.as_view(), name="fax_originate"),
	url(regex=r'^fax/report/$', view=FaxReport.as_view(), name="fax_report"),
	url(regex=r'^fax/inbox/$', view=FaxInbox.as_view(), name="fax_inbox"),
)