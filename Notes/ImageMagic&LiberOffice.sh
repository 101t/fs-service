#sudo apt-get install libtiff-tools
#sudo apt-get install graphicsmagick-imagemagick-compat
sudo apt-get install imagemagick
sudo apt-get install libreoffice

convert FileIN -resize 1728x2255 -monochrome -units PixelsPerInch -density 204x196 -compress group4 -endian lsb FileOut

convert FileIN -resize 1728x1078 -monochrome -units PixelsPerInch -density 204x98 -compress group4 -endian lsb FileOut.tiff

soffice --headless --convert-to pdf filename.xlsx




# libreoffice-script-provider-python uno-libs3 python3-uno