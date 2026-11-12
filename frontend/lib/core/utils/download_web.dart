import 'dart:html' as html;

/// Descarga un fichero en el navegador (Flutter web).
void downloadFile(String filename, List<int> bytes) {
  final blob = html.Blob([bytes]);
  final url = html.Url.createObjectUrlFromBlob(blob);
  final anchor = html.AnchorElement(href: url)
    ..download = filename
    ..click();
  html.Url.revokeObjectUrl(url);
}

/// Abre HTML en una pestaña nueva (impresión de documentos del admin).
void openHtmlInNewTab(List<int> bytes) {
  final blob = html.Blob([bytes], 'text/html');
  final url = html.Url.createObjectUrlFromBlob(blob);
  html.window.open(url, '_blank');
}
