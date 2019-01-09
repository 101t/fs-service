from django.conf.urls import include, url
from rest_framework.routers import DefaultRouter, SimpleRouter
from views.root import APIRootView

router = DefaultRouter()

urlpatterns = (
	url(r'^auth/', include('rest_framework.urls', namespace='rest_framework')),
	url(r'^gateway/', include('main.apps.gateway.urls')), # , namespace='gateway'
	url(r'^service/', include('main.apps.service.urls')), # , namespace='voice'
	url(r'^$', APIRootView.as_view(), name="root-api"), # include(router.urls)
)
