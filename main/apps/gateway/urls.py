from django.urls import include
from django.conf.urls import url
from .views.operation import GatewayOperation
from .views.check import GatewayCheck
from .views.root import APIGatewayView

urlpatterns = (
	url(regex=r'^$', view=APIGatewayView.as_view(), name="gateway"),
	url(regex=r'^operation/$', view=GatewayOperation.as_view(), name="operation"), #POST
	url(regex=r'^check/$', view=GatewayCheck.as_view(), name="check"),
)