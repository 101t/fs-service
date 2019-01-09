auth = 75f1dd06d7fc62ef2dfe9a274920221b466bde6d

curl -X POST http://185.77.91.8/api/service/fax/ \
     -H 'Authorization: Token 75f1dd06d7fc62ef2dfe9a274920221b466bde6d' \
     -H 'Content-Type:application/json' \
     #-H 'Accept:text/html' \
     -d '{"username": "908504300003", "filename": "224e2d06-9553-4c46-a053-974a1aa2dfba.jpg", "numbers": "908504300004"}'

curl -X POST http://185.77.91.8/api/service/file/ \
     -H 'Authorization: Token 75f1dd06d7fc62ef2dfe9a274920221b466bde6d' \
     -F "filename=@/tmp/img.jpeg;type=image/jpg"

curl -X POST http://185.77.91.8/api/gateway/operation/ \
     -H 'Authorization: Token 75f1dd06d7fc62ef2dfe9a274920221b466bde6d' \
     -H 'Content-Type:application/json' \
     -d '{"service": "new", "username": "902127010339", "password": "ibo34288", "host": "sip.pasifiktelekom.com.tr", "register": true}'

curl -X POST http://185.77.91.8/api/service/file/download/ \
     -H 'Authorization: Token 75f1dd06d7fc62ef2dfe9a274920221b466bde6d' \
     -H 'Content-Type:application/json' \
     -d '{"filename": "ff2d60c6-9095-4e34-ba22-c477bc99a395.jpg", "format": "tiff"}'