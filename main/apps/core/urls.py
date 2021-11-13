from django.urls import include, path
from rest_framework.routers import DefaultRouter, SimpleRouter
from .views.root import APIRootView

router = DefaultRouter()

urlpatterns = (
	path('auth/', include('rest_framework.urls', namespace='rest_framework')),
	path('gateway/', include('main.apps.gateway.urls')), # , namespace='gateway'
	path('service/', include('main.apps.service.urls')), # , namespace='voice'
	path('', APIRootView.as_view(), name="root-api"), # include(router.urls)
)
