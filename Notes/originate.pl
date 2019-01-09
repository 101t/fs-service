#!/usr/bin/perl                                                                                
use strict;
use warnings;
require ESL;

my $host = "127.0.0.1";
my $port = "8021";
my $pass = "ClueCon";
my $con  = new ESL::ESLconnection($host, $port, $pass);
if (! $con) { die "Unable to establish connection to $host:$port\n"; }
$con->events("plain","all");

my $target = shift;
my $uuid = $con->api("create_uuid")->getBody();
my $res =
    $con->bgapi("originate","{origination_uuid=$uuid}$target 9664");
my $job_uuid = $res->getHeader("Job-UUID");
print "Call to $target has Job-UUID of $job_uuid and call uuid of $uuid \n";

my $stay_connected = 1;
while ( $stay_connected ) {
    my $e = $con->recvEventTimed(20);
    if ( $e ) {
        my $ev_name = $e->getHeader("Event-Name");
        # Should we check for the $job_uuid to match the background job ?                      
        if ( $ev_name eq 'BACKGROUND_JOB' ) {
            if ( $e->getHeader("Job-UUID") eq $job_uuid ) {
                my $call_result = $e->getBody();
                print "Result of call to $target was $call_result\n\n";
            }
        } elsif ( $ev_name eq 'DTMF' ) {
            my $digit = $e->getHeader("DTMF-Digit");
            print "Received DTMF digit: $digit\n";
            if ( $digit =~ m/\D/ ) {
                print "Exiting...\n";
                $stay_connected = 0;
            }
        } else {
      # Some other event                                                                       
        }
    } else {
    # do other things while waiting for events...                                              
    }
}
$con->api("uuid_kill",$uuid)