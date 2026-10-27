Vendored Windows x64 Tesseract OCR runtime

Tesseract version: 5.5.3.20260724
Source installer: https://github.com/tesseract-ocr/tesseract/releases/download/5.5.3/tesseract-ocr-w64-setup-5.5.3.20260724.exe
Installer SHA-256: BEE9E3434BD94FD65387D9BE28CD467A41F61B1275383B55B0F59A1331270AE4

This directory contains tesseract.exe, its Windows runtime DLL dependencies,
standard Tesseract data/configuration files, and the English (eng), Persian
(fas), and orientation/script detection (osd) traineddata files. The engine
installer and all required language data are checked into the repository so
packaging does not need to install or download Tesseract.

Tesseract and tessdata_fast license texts are in licenses/.
The traineddata files come from:
https://github.com/tesseract-ocr/tessdata_fast/tree/87416418657359cb625c412a48b6e1d6d41c29bd
