# This is an auto-generated Django model module.
# You'll have to do the following manually to clean this up:
#   * Rearrange models' order
#   * Make sure each model has one field with primary_key=True
#   * Make sure each ForeignKey has `on_delete` set to the desired behavior.
#   * Remove `managed = False` lines if you wish to allow Django to create, modify, and delete the table
# Feel free to rename the models, but don't rename db_table values or field names.
from __future__ import unicode_literals

from django.db import models


class Cdr(models.Model):
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
    imported = models.BooleanField()

    class Meta:
        managed = False
        db_table = 'cdr'


class CdrImport(models.Model):
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
        db_table = 'cdr_import'
