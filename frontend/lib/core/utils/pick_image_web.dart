import 'dart:html' as html;
import 'dart:typed_data';

/// Selector de fichero de imagen para Flutter web.
Future<({String filename, Uint8List bytes})?> pickImage() async {
  final input = html.FileUploadInputElement()..accept = 'image/*';
  input.click();
  await input.onChange.first;
  final files = input.files;
  if (files == null || files.isEmpty) return null;
  final file = files.first;
  final reader = html.FileReader();
  reader.readAsArrayBuffer(file);
  await reader.onLoad.first;
  return (
    filename: file.name,
    bytes: Uint8List.fromList(reader.result as List<int>),
  );
}
