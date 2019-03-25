# Helpfull commands to getting started:

### reload XML 
```sh
reloadxml
```
### Restart FreeSWITCH Service
```sh
fsctl shutdown restart
```
### It will restart the freeswitch after all calls are hanged up.
```sh
fsctl shutdown elegant restart
```
### To know profile status
```sh
sofia status profile
```
### sip profile
```sh
sofia status profile external
```
### Show Channels
```sh
show channels
```
### Show Calls
```sh
show calls
```
###Originate Call without Registration :))
```sh
/usr/local/freeswitch/bin/fs_cli -x "originate {sip_auth_username=908504300006,sip_auth_password=Ytungftr54ey7}sofia/external/905999999999@0.0.0.0 &echo"
```
make a call between two sip phones using fs_cli
originate user/1000<context> &bridge(user/1001@<context>)

### Send Fax using fs_cli
```sh
originate {ignore_early_media=true,absolute_codec_string='PCMU,PCMA',fax_enable_t38=true,fax_verbose=true,fax_use_ecm=true,fax_enable_t38_request=true}sofia/gateway/908509999998/908509999999 &txfax('/home/tarek/fax/txfax-sample.tiff')

originate {ignore_early_media=true,fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/908509999999/⁠⁠⁠02164958288 &txfax('/home/user1/fax/txfax-sample.tiff')

sftp://user1@0.0.0.0/home/user1/fax/txfax-sample.tiff

originate {ignore_early_media=true,fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/908509999998/908509999999⁠⁠⁠ &txfax('/home/tarek/fax/sample.tif')

originate {ignore_early_media=true,fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/908509999998/08509999999 &txfax('/home/tarek/fax/sample.tiff')

originate {ignore_early_media=true,absolute_codec_string='PCMU,PCMA',fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/908509999998/908509999999 &txfax('/home/tarek/fax/sample.tiff')
```