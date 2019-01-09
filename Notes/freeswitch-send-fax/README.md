# How to Send Fax using FreeSWITCH

You can transmit electronic documents to a destination fax machine using FreeSWITCH or Asterisk.
Now in this tutorial we will show you the most common configuration and focus on simplest way to
run your Fax Command via FreeSWITCH.
Only TIFF documents can be transmitted, however it is possible to convert a number of graphical formats to TIFF format.

## STEP 1:

First of all you should install Dependencies.

tiff2pdf command line

CentOS:

```shell
yum install ghostscript
yum install libtiff
```

Ubuntu:

```shell
apt-get install ghostscript
apt-get install libtiff-tools
```

this script important to convert document to TIFF format.

Ghostscript is the key tool for converting PDF files to FAX compatible TIFF files. You may see other programs being used for this job, but behind the scenes they usually use Ghostscript to do the hard work.

You can use Ghostscript for PDF/PS files. Here is an example, producing a standard resolution TIFF file.

```shell
gs -q -r204x98 -g1728x1078 -dNOPAUSE -dBATCH -dSAFER -sDEVICE=tiffg3 -sOutputFile=txfax.tiff -- txfax.pdf
```
Here is an example producing a fine resolution TIFF file:

```shell
gs -q -r204x196 -g1728x2156 -dNOPAUSE -dBATCH -dSAFER -sDEVICE=tiffg3 -sOutputFile=txfax.tiff -- txfax.pdf
```

## STEP 2:

Setting up your gateway provider number that support VoIP service.

${CONF}/sip_profiles/external/90XXXXXXXXXX.xml

```xml
<include>
  <gateway name="90XXXXXXXXXX">
    <param name="username" value="90XXXXXXXXXX"/>
    <param name="password" value="YOURVOIPPASSWORD"/>
    <param name="realm" value="sip.voipcomapny.com"/>
    <param name="proxy" value="sip.voipcomapny.com"/>
    <param name="register" value="true"/>
  </gateway>
</include>
```

## STEP 3:

After restart your freeswitch by typing `service freeswitch restart`, now we are ready to originate fax call to your destination fax number, go to `cd /usr/local/freeswitch/bin` and type `./fs_cli` you will start freeswitch command line program to execute the following code:

```shell
originate {ignore_early_media=true,absolute_codec_string='PCMU,PCMA',fax_enable_t38=true,fax_verbose=true,fax_use_ecm=true,fax_enable_t38_request=true}sofia/gateway/90XXXXXXXXXX/909999999999 &txfax('/path/to/file/sample.tiff')
```

And if this fails further retries with:

```shell
originate {ignore_early_media=true,absolute_codec_string='PCMU,PCMA',fax_enable_t38=true,fax_verbose=true,fax_use_ecm=false,fax_enable_t38_request=false}sofia/gateway/90XXXXXXXXXX/909999999999 &txfax('/path/to/file/sample.tiff')
```

Follow the log result you will see if Fax has been received successfully.

If it not received check your sample.tiff format it should be compatible with tiff fax standart mode.

or check your installed codic "PCMU,PCMA" inside freeswitch.