### Watch number of active channels:

watch "asterisk -vvvvvrx 'core show channels' | grep channels"
[edit] Watch number of active calls

watch "asterisk -vvvvvrx 'core show channels' | grep calls"
[edit] Watch active channels

watch "asterisk -vvvvvrx 'core show channels verbose'"
[edit] Watch active channels in Asterisk 1.8

watch "asterisk -vvvvvrx 'core show channels verbose'" 