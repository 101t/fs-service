# reload XML 
reloadxml
# Restart FreeSWITCH Service
fsctl shutdown restart
# It will restart the freeswitch after all calls are hanged up.
fsctl shutdown elegant restart
# To know profile status
sofia status profile
# sip profile
sofia status profile external
# Show Channels
show channels
# Show Calls
show calls
#Originate Call without Registration :))
/usr/local/freeswitch/bin/fs_cli -x "originate {sip_auth_username=908504300006,sip_auth_password=Ytungftr54ey7}sofia/external/905424226772@91.244.227.228 &echo"

make a call between two sip phones using fs_cli
originate user/1000<context> &bridge(user/1001@<context>)
# Send Fax using fs_cli
originate {ignore_early_media=true,absolute_codec_string='PCMU,PCMA',fax_enable_t38=true,fax_verbose=true,fax_use_ecm=true,fax_enable_t38_request=true}sofia/gateway/908504300004/908504300003 &txfax('/home/tarek/fax/txfax-sample.tiff')

originate {ignore_early_media=true,fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/908504300003/⁠⁠⁠02164958288 &txfax('/home/user1/fax/txfax-sample.tiff')

sftp://user1@185.77.91.8/home/user1/fax/txfax-sample.tiff

originate {ignore_early_media=true,fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/908504300004/908504300003⁠⁠⁠ &txfax('/home/tarek/fax/sample.tif')

originate {ignore_early_media=true,fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/908504300004/08504300003 &txfax('/home/tarek/fax/sample.tiff')

originate {ignore_early_media=true,absolute_codec_string='PCMU,PCMA',fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/908504300004/908504300003 &txfax('/home/tarek/fax/sample.tiff')