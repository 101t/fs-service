```
                             ! Execute a shell command
                      acl show Show a named ACL or list all named ACLs
                    ael reload Reload AEL configuration
ael set debug {read|tokens|mac Enable AEL debugging flags
                  agent logoff Sets an agent offline
                    agent show Show status of agents
             agent show online Show all online agents
                 agi dump html Dumps a list of AGI commands in HTML format
                      agi exec Add AGI command to a channel in Async AGI
        agi set debug [on|off] Enable/Disable AGI debugging
     agi show commands [topic] List AGI commands or specific help
                 aoc set debug enable cli debugging of AOC messages
           calendar dump sched Dump calendar sched context
        calendar show calendar Display information about a calendar
       calendar show calendars Show registered calendars
           calendar show types Show all calendar types loaded
               cb mysql status Show connection status of CBMySQL
                     cc cancel Kill a CC transaction
              cc report status Reports CC stats
              cdr mysql status Show connection status of cdr_mysql
               cdr show status Display the CDR status
               cel show status Display the CEL status
             channel originate Originate a call
              channel redirect Redirect a call
        channel request hangup Request a hangup on a given channel
         cli check permissions Try a permissions config for a user
        cli reload permissions Reload CLI permissions config
              cli show aliases Show CLI command aliases
          cli show permissions Show CLI permissions
               confbridge kick Kick participants out of conference bridges.
               confbridge list List conference bridges and participants.
               confbridge lock Lock a conference.
               confbridge mute Mute a participant.
       confbridge record start Start recording a conference
        confbridge record stop Stop recording a conference.
          confbridge show menu Show a conference menu
         confbridge show menus Show a list of conference menus
confbridge show profile bridge Show a conference bridge profile.
confbridge show profile bridge Show a list of conference bridge profiles.
  confbridge show profile user Show a conference user profile.
 confbridge show profile users Show a list of conference user profiles.
             confbridge unlock Unlock a conference.
             confbridge unmute Unmute a participant.
                   config list Show all files that have loaded a configuration file
                 config reload Force a reload on modules using a particular configuration file
           core abort shutdown Cancel a running shutdown
            core clear profile Clear profiling info
       core ping taskprocessor Ping a named task processor
                   core reload Global reload
       core restart gracefully Restart Asterisk gracefully
              core restart now Restart Asterisk immediately
  core restart when convenient Restart Asterisk at empty call volume
        core set debug channel Enable/disable debugging on a channel
                core set debug Set level of debug chattiness
              core set verbose Set level of verbose chattiness
core show applications [like|d Shows registered dialplan applications
         core show application Describe a specific dialplan application
      core show calls [uptime] Display information on calls
core show channels [concise|ve Display information on channels
             core show channel Display information on a specific channel
        core show channeltypes List available channel types
         core show channeltype Give more details on that channel type
core show codecs [audio|video| Displays a list of codecs
               core show codec Shows a specific codec
     core show config mappings Display config mappings (file names to config engines)
        core show file formats Displays file formats
 core show file version [like] List versions of files used to build Asterisk
    core show functions [like] Shows registered dialplan functions
            core show function Describe a specific dialplan function
  core show hanguphandlers all Show hangup handlers of all channels
      core show hanguphandlers Show hangup handlers of a specified channel
                core show help Display help list, or specific help on a command
               core show hints Show dialplan hints
                core show hint Show dialplan hint
       core show image formats Displays image formats
             core show license Show the license(s) for this copy of Asterisk
             core show profile Display profiling info
            core show settings Show some core settings
            core show switches Show alternative switches
             core show sysinfo Show System Information
      core show taskprocessors List instantiated task processors and statistics
             core show threads Show running threads
         core show translation Display translation matrix
    core show uptime [seconds] Show uptime information
             core show version Display version info
            core show warranty Show the warranty (if any) for this copy of Asterisk
          core stop gracefully Gracefully shut down Asterisk
                 core stop now Shut down Asterisk immediately
     core stop when convenient Shut down Asterisk at empty call volume
          core waitfullybooted Wait for Asterisk to be fully booted
         dahdi destroy channel Destroy a channel
                 dahdi restart Fully restart DAHDI channels
                 dahdi set dnd Sets/resets DND (Do Not Disturb) mode on a channel
      dahdi set hwgain {rx|tx} Set hardware gain on a channel
      dahdi set swgain {rx|tx} Set software gain on a channel
           dahdi show cadences List cadences
dahdi show channels [group|con Show active DAHDI channels
            dahdi show channel Show information on a channel
             dahdi show status Show all DAHDI cards status
            dahdi show version Show the DAHDI version in use
                      data get Data API get
           data show providers Show data providers
                  database del Removes database key/value
              database deltree Removes database keytree/values
                  database get Gets database value
                  database put Adds/updates database value
                database query Run a user-specified query on the astdb
                 database show Shows database contents
              database showkey Shows database contents
               devstate change Change a custom device state
                 devstate list List currently known custom device states
        dialplan add extension Add new extension into context
        dialplan add ignorepat Add new ignore pattern
          dialplan add include Include context in other context
                dialplan debug Show fast extension pattern matching data structures
               dialplan reload Reload extensions and *only* extensions
       dialplan remove context Remove a specified context
     dialplan remove extension Remove a specified extension
     dialplan remove ignorepat Remove ignore pattern from context
       dialplan remove include Remove a specified include from context
                 dialplan save Save current dialplan into a file
          dialplan set chanvar Set a channel variable
dialplan set extenpatternmatch Use the Old extension pattern matching algorithm.
dialplan set extenpatternmatch Use the New extension pattern matching algorithm.
           dialplan set global Set global dialplan variable
         dialplan show chanvar Show channel variables
         dialplan show globals Show global dialplan variables
                 dialplan show Show dialplan
                dnsmgr refresh Performs an immediate refresh
                 dnsmgr reload Reloads the DNS manager configuration
                 dnsmgr status Display the DNS manager status
           dundi flush [stats] Flush DUNDi cache
                  dundi lookup Lookup a number in DUNDi
                dundi precache Precache a number in DUNDi
                   dundi query Query a DUNDi EID
      dundi set debug {on|off} Enable/Disable DUNDi debugging
              dundi show cache Show DUNDi cache
           dundi show entityid Display Global Entity ID
              dundi show hints Show DUNDi hints in the cache
           dundi show mappings Show DUNDi mappings
dundi show peers [registered|i Show defined DUNDi peers
               dundi show peer Show info on a specific DUNDi peer
           dundi show precache Show DUNDi precache
           dundi show requests Show DUNDi requests
              dundi show trans Show active DUNDi transactions
  dundi store history {on|off} Enable/Disable DUNDi historic records
              event dump cache Dump the internal event cache (for debugging)
         extra destroy channel Destroy a channel
                 extra restart Fully restart EXTRA channels
                 extra set dnd Sets/resets DND (Do Not Disturb) mode on a channel
              extra set hwgain Set hardware gain on a channel
              extra set swgain Set software gain on a channel
extra show channels [group|con Show active EXTRA channels
            extra show channel Show information on a channel
             extra show status Show all EXTRA cards status
            extra show version Show the EXTRA version in use
        fax set debug {on|off} Enable/Disable FAX debugging on new FAX sessions
         fax show capabilities Show the capabilities of the registered FAX technology modules
              fax show session Show the status of the named FAX sessions
             fax show sessions Show the current FAX sessions
             fax show settings Show the global settings and defaults of both the FAX core and technology modules
                fax show stats Summarize FAX session history
              fax show version Show versions of FAX For Asterisk components
               features reload Reloads configured features
                 features show Lists configured features
                  file convert Convert audio file
                    g729 debug Toggle g729 codec frame size statistics
           group show channels Display active channels with group(s)
          gsm check phone stat Check the stat of the phone
                gsm debug span Enables GSM debugging on a span
      gsm intensive debug span Enables REALLY INTENSE GSM debugging
             gsm no debug span Disables GSM debugging on a span
                 gsm power off Power off gsm module
                  gsm power on Power on gsm module
               gsm power reset Power reset gsm module
                gsm power stat Get gsm module power stat
               gsm reload span Reload GSM module configure
                   gsm send at Send AT Commmand on a given GSM span
                  gsm send pdu Send PDU on a given GSM span
                  gsm send sms Send SMS on a given GSM span
              gsm send sync at Send AT Command and wait finish on a given GSM span
            gsm send sync csms Send Concatenated SMS and wait finish on a given GSM span
             gsm send sync sms Send SMS and wait finish on a given GSM span
                 gsm send ussd Send USSD on a given GSM span
              gsm set debug at Set at command debug mode on a given GSM span
            gsm set debug file Sends GSM debug output to the specified file
       gsm set send sms coding Setting send sms character coding
     gsm set send sms mode pdu Setting send sms mode is pdu
    gsm set send sms mode text Setting send sms mode is text
         gsm set send sms smsc Setting send sms Service Message Center number
                gsm show debug Displays current GSM debug settings
             gsm show debug at Show at command debug stat on a given GSM span
      gsm show send sms coding Show send sms character coding
        gsm show send sms mode Show send sms mode
        gsm show send sms smsc Show send sms Service Message Center number
                gsm show spans Displays GSM Information
                 gsm show span Displays GSM Information
  gsm show statistics outbound Displays statistics of outbound
              gsm show version Displays libgsmat version
          gsm unset debug file Ends GSM debug output to file
           gtalk show channels Show GoogleTalk channels
           gtalk show settings Show GoogleTalk global settings
                hangup request <no description available>
                          help <no description available>
              http show status Display HTTP server status
                iax2 provision Provision an IAX device
           iax2 prune realtime Prune a cached realtime lookup
                   iax2 reload Reload IAX configuration
  iax2 set debug {on|off|peer} Enable/Disable IAX debugging
    iax2 set debug jb {on|off} Enable/Disable IAX jitterbuffer debugging
 iax2 set debug trunk {on|off} Enable/Disable IAX trunk debugging
                  iax2 set mtu Set the IAX systemwide trunking MTU
               iax2 show cache Display IAX cached dialplan
    iax2 show callnumber usage Show current entries in IP call number limit table
            iax2 show channels List active IAX channels
            iax2 show firmware List available IAX firmware
            iax2 show netstats List active IAX channel netstats
                iax2 show peer Show details on specific IAX peer
               iax2 show peers List defined IAX peers
        iax2 show provisioning Display iax provisioning
            iax2 show registry Display IAX registration status
               iax2 show stats Display IAX statistics
             iax2 show threads Display IAX helper thread info
        iax2 show users [like] List defined IAX users
             iax2 test losspct Set IAX2 incoming frame loss percentage
               iax2 unregister Unregister (force expiration) an IAX2 peer from the registry
                indication add Add the given indication to the country
             indication remove Remove the given indication from the country
               indication show Display a list of all countries/indications
      jabber create collection Creates a PubSub node collection.
            jabber create leaf Creates a PubSub leaf node
            jabber delete node Deletes a PubSub node
             jabber list nodes Lists PubSub nodes
            jabber purge nodes Purges PubSub nodes
                 jabber reload Reload Jabber configuration
     jabber set debug {on|off} Enable/Disable Jabber debug
           jabber show buddies Show buddy lists of our clients
       jabber show connections Show state of clients and components
                 jingle reload Reload Jingle configuration
          jingle show channels Show Jingle channels
                     keys init Initialize RSA key passcodes
                     keys show Displays RSA key information
           local show channels List status of local channels
                   logger mute Toggle logging output to a console
                 logger reload Reopens the log files
                 logger rotate Rotates and reopens the log files
logger set level {DEBUG|NOTICE Enables/Disables a specific logging level for this console
          logger show channels List configured log channels
                manager reload Reload manager configurations
    manager set debug [on|off] Show, enable, disable debugging of the manager code
          manager show command Show a manager interface command
         manager show commands List manager interface commands
        manager show connected List connected manager interface users
           manager show eventq List manager interface queued events
           manager show events List manager interface events
            manager show event Show a manager interface event
         manager show settings Show manager global settings
            manager show users List configured manager users
             manager show user Display information on a specific manager user
                   meetme kick Kick a conference or a user in a conference.
                   meetme list List all conferences or a specific conference.
          meetme {lock|unlock} Lock or unlock a conference to new users.
          meetme {mute|unmute} Mute or unmute a conference or a user in a conference.
     mfcr2 call files [on|off] Enable/Disable MFC/R2 call files
             mfcr2 set blocked Reset MFC/R2 channel forcing it to BLOCKED
               mfcr2 set debug Set MFC/R2 channel logging level
                mfcr2 set idle Reset MFC/R2 channel forcing it to IDLE
mfcr2 show channels [group|con Show MFC/R2 channels
           mfcr2 show variants Show supported MFC/R2 variants
            mfcr2 show version Show OpenR2 library version
           mgcp audit endpoint Audit specified MGCP endpoint
                   mgcp reload Reload MGCP configuration
       mgcp set debug {on|off} Enable/Disable MGCP debugging
           mgcp show endpoints List defined MGCP endpoints
          minivm list accounts List defined mini-voicemail boxes
         minivm list templates List message templates
             minivm list zones List zone message formats
                 minivm reload Reload Mini-voicemail configuration
          minivm show settings Show mini-voicemail general settings
             minivm show stats Show some mini-voicemail statistics
  mixmonitor {start|stop|list} Execute a MixMonitor command
                   module load Load a module by name
                 module reload Reload configuration for a module
            module show [like] List modules and info
                 module unload Unload a module by name
                    moh reload Reload MusicOnHold
              moh show classes List MusicOnHold classes
                moh show files List MusicOnHold file-based classes
              no debug channel Disable debugging on channel(s)
                     odbc read Test reading a func_odbc function
                     odbc show List ODBC DSN(s)
                    odbc write Test setting a func_odbc function
                     originate <no description available>
              parkedcalls show List currently parked calls
         phoneprov show routes Show registered phoneprov http routes
              pktccops gatedel Send Gate-Det to cmts
              pktccops gateset Send Gate-Set to cmts
   pktccops set debug {on|off} Enable/Disable COPS debugging
          pktccops show cmtses List PacketCable COPS CMTSes
           pktccops show gates List PacketCable COPS GATEs
           pktccops show pools List PacketCable MTA pools
          presencestate change Change a custom presence state
            presencestate list List currently know custom presence states
              pri destroy span Destroy a PRI span
        pri intense debug span <no description available>
   pri service disable channel Remove a channel from service
    pri service enable channel Return a channel to service
pri set debug {on|off|hex|inte Enables PRI debugging on a span
            pri set debug file Sends PRI debug output to the specified file
             pri show channels Displays PRI channel information
                pri show debug Displays current PRI debug settings
                pri show spans Displays PRI span information
                 pri show span Displays PRI span information
              pri show version Displays libpri version
              queue add member Add a channel to a specified queue
queue reload {parameters|membe Reload queues, members, queue rules, or parameters
           queue remove member Removes a channel from a specified queue
             queue reset stats Reset statistics for a queue
             queue set penalty Set penalty for a channel of a specified queue
           queue set ringinuse Set ringinuse for a channel of a specified queue
                    queue show Show status of a specified queue
              queue show rules Show the rules defined in queuerules.conf
  queue {pause|unpause} member Pause or unpause a queue member
              realtime destroy Delete a row from a RealTime database
                 realtime load Used to print out RealTime variables.
          realtime mysql cache Shows cached tables within the MySQL realtime driver
         realtime mysql status Shows connection information for the MySQL RealTime driver
                realtime store Store a new row into a RealTime database
               realtime update Used to update RealTime variables.
              realtime update2 Used to test the RealTime update2 method
                        reload <no description available>
    rtcp set debug {on|off|ip} Enable/Disable RTCP debugging
       rtcp set stats {on|off} Enable/Disable RTCP stats
     rtp set debug {on|off|ip} Enable/Disable RTP debugging
            say load [new|old] Set or show the say mode
             sla show stations Show SLA Stations
               sla show trunks Show SLA Trunks
                 ss7 block cic Blocks the given CIC
             ss7 block linkset Blocks all CICs on a linkset
ss7 set debug {on|off} linkset Enables SS7 debugging on a linkset
             ss7 show channels Displays SS7 channel information
              ss7 show linkset Shows the status of a linkset
              ss7 show version Displays libss7 version
               ss7 unblock cic Unblocks the given CIC
           ss7 unblock linkset Unblocks all CICs on a linkset
       stun set debug {on|off} Enable/Disable STUN debugging
              stun show status Show STUN servers and statuses
                   timing test Run a timing test
               transcoder show Display DAHDI transcoder utilization.
   udptl set debug {on|off|ip} Enable/Disable UDPTL debugging
             udptl show config Show UDPTL config options
                        ulimit Set or show process resource limits
                unistim reload Reload UNISTIM configuration
           unistim send packet Send packet (for reverse engineering)
    unistim set debug {on|off} Toggle UNITSTIM debugging
          unistim show devices Show UNISTIM devices
             unistim show info Show UNISTIM info
              voicemail reload Reload voicemail configuration
          voicemail show users List defined voicemail boxes
          voicemail show zones List zone message formats
                     wat debug Enables WAT debugging
                      wat exec Executes an arbitrary AT command
                  wat send sms Sends a SMS
                wat show spans Displays WAT span information
                 wat show span Displays WAT span information
              wat show version Displays libwat version
        xmpp create collection Creates a PubSub node collection.
              xmpp create leaf Creates a PubSub leaf node
              xmpp delete node Deletes a PubSub node
               xmpp list nodes Lists PubSub nodes
              xmpp purge nodes Purges PubSub nodes
       xmpp set debug {on|off} Enable/Disable Jabber debug
             xmpp show buddies Show buddy lists of our clients
         xmpp show connections Show state of clients and components
```