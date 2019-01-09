
import os, time
import subprocess

class FileConverter(object):
	"""docstring for FileConverter
## Supported Formats:
doc, docx, rtf, xls, xlsx, ppt, pptx, odt, ods, odp, txt, html, csv, psd, pdf, jpg, jpeg, tiff, gif, bmp, png.
One Page:
jpg, jpeg, tiff, gif, bmp, png, psd
Multi Page:
doc, docx, rtf, xls, xlsx, ppt, pptx, odt, ods, odp, txt, html, csv, pdf
"""
	img_list = ["jpg", "jpeg", "tiff", "gif", "bmp", "png", "psd"]
	doc_list = ["doc", "docx", "rtf", "xls", "xlsx", "ppt", 
		"pptx", "odt", "ods", "odp", "txt", "html", "csv", "pdf"]
	def __init__(self, filename, is_enhanced=False, DEBUG=False):
		self.filename = filename
		self.dirname = os.path.dirname(filename)
		fn, fx = os.path.splitext(self.filename)
		self.fn = fn
		self.fx = fx[1:]
		self.is_doc = True if self.fx in self.doc_list else False
		self.is_enhanced = is_enhanced
		self.DEBUG = DEBUG

	def convert(self):
		if self.is_doc:
			cmd_str = "/usr/bin/soffice --headless \"-env:UserInstallation=file:///tmp/LibreOffice_Conversion_${USER}\" --convert-to pdf \"%s\" --outdir %s" % (self.filename, self.dirname)
			result = subprocess.Popen(cmd_str, shell=True)
			result.wait()
			if self.DEBUG: 
				print "LiberOffice Convert:\n %s" % result.communicate()[0]
			self.filename = "%s.pdf" % self.fn
		#cmd1 = "convert %s -resize 1728x2255 -monochrome -units PixelsPerInch -density 204x196 -compress group4 -endian lsb %s.tiff" % (self.filename, self.fn)
		#cmd2 = "convert %s -resize 1728x1078 -monochrome -units PixelsPerInch -density 204x98 -compress group4 -endian lsb %s.tiff" % (self.filename, self.fn)
		os.chmod(self.filename, 0o666)
		cmd1 = "convert %s -resize 1728x2255 -background white -alpha remove -monochrome -units PixelsPerInch -density 204x196 -compress Fax %s.tiff" % (self.filename, self.fn) # x2156
		cmd2 = "convert %s -resize 1728x1078 -background white -alpha remove -monochrome -units PixelsPerInch -density 204x98  -compress Fax %s.tiff" % (self.filename, self.fn) # x1186
		cmd_str = cmd1 if self.is_enhanced else cmd2
		result = subprocess.Popen(cmd_str, shell=True)
		#result = subprocess.call(cmd_str.split(" "))
		result.wait()
		if self.DEBUG:
			print "ImageMagic Convert:\n %s" % result.communicate()[0]
			#.communicate()[0]
		return "%s.tiff" % self.fn
	def convert2pdf(self):
		if self.is_doc:
			cmd_str = "soffice --headless \"-env:UserInstallation=file:///tmp/LibreOffice_Conversion_${USER}\" --convert-to pdf --outdir \"%s\" %s" % (self.dirname, self.filename)
			result = subprocess.Popen(cmd_str, shell=True)
			result.wait()
			if self.DEBUG: 
				print "LiberOffice Convert:\n %s" % result.communicate()[0]
			self.filename = "%s.pdf" % self.fn
		cmd_str = "convert %s %s.pdf" % (self.filename, self.fn)
		result = subprocess.Popen(cmd_str, shell=True)
		result.wait()
		if self.DEBUG:
			print "ImageMagic Convert2PDF:\n %s" % result.communicate()[0]
			#.communicate()[0]
		return "%s.pdf" % self.fn
"""
from main.apps.service.views.utils.converter import FileConverter
fc = FileConverter(filename="/tmp/img.jpeg", is_enhanced=False, DEBUG=True)
fc.convert()
"""
"""
https://www.soft-switch.org/spandsp_faq/ar01s14.html

REMEMBER THAT ImageMagick has:
24bit JPEG:  1294kB
24bit LZW:   1759kB
1bit  Fax:    135kB
"""