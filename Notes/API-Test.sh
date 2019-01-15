auth = xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

curl -X POST http://0.0.0.0/api/service/fax/ \
     -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \
     -H 'Content-Type:application/json' \
     #-H 'Accept:text/html' \
     -d '{"username": "908509999999", "filename": "224e2d06-9553-4c46-a053-974a1aa2dfba.jpg", "numbers": "908509999998"}'

curl -X POST http://0.0.0.0/api/service/file/ \
     -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \
     -F "filename=@/tmp/img.jpeg;type=image/jpg"

curl -X POST http://0.0.0.0/api/gateway/operation/ \
     -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \
     -H 'Content-Type:application/json' \
     -d '{"service": "new", "username": "902129999999", "password": "secret", "host": "sip.providerexample.com", "register": true}'

curl -X POST http://0.0.0.0/api/service/file/download/ \
     -H 'Authorization: Token xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' \
     -H 'Content-Type:application/json' \
     -d '{"filename": "ff2d60c6-9095-4e34-ba22-c477bc99a395.jpg", "format": "tiff"}'