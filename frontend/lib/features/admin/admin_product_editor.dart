import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../../../core/utils/pick_image.dart';

/// Editor de producto del admin: atributos base (nombre, precio,
/// descripciones), inventario por almacén e imágenes (subida base64).
class AdminProductEditorPage extends StatefulWidget {
  const AdminProductEditorPage({super.key, required this.productId});
  final String productId;

  @override
  State<AdminProductEditorPage> createState() => _AdminProductEditorPageState();
}

class _AdminProductEditorPageState extends State<AdminProductEditorPage> {
  Map<String, dynamic>? _product;
  Map<String, dynamic> _attributes = {};
  List<Map<String, dynamic>> _inventories = [];
  List<Map<String, dynamic>> _images = [];
  bool _loading = true;
  bool _saving = false;

  late final TextEditingController _sku;
  late final TextEditingController _name;
  late final TextEditingController _price;
  late final TextEditingController _shortDescription;
  late final TextEditingController _description;
  late final TextEditingController _weight;

  @override
  void initState() {
    super.initState();
    _sku = TextEditingController();
    _name = TextEditingController();
    _price = TextEditingController();
    _shortDescription = TextEditingController();
    _description = TextEditingController();
    _weight = TextEditingController();
    _load();
  }

  Future<void> _load() async {
    final data = await context.read<AdminProvider>().get('admin/catalog/products/edit/${widget.productId}');
    if (data is Map && mounted) {
      setState(() {
        _product = Map<String, dynamic>.from(data['product'] as Map);
        _attributes = Map<String, dynamic>.from((data['attributes'] ?? const {}) as Map);
        _inventories = decodeList(data['inventories']);
        _images = decodeList(data['images']);
        _sku.text = (_product?['sku'] ?? '') as String;
        _name.text = (_attributes['name'] ?? '') as String;
        _price.text = (_attributes['price'] ?? '0').toString();
        _shortDescription.text = (_attributes['short_description'] ?? '') as String;
        _description.text = (_attributes['description'] ?? '') as String;
        _weight.text = (_attributes['weight'] ?? '0').toString();
        _loading = false;
      });
    } else if (mounted) {
      setState(() => _loading = false);
    }
  }

  Future<void> _save() async {
    setState(() => _saving = true);
    final admin = context.read<AdminProvider>();
    try {
      await admin.put('admin/catalog/products/edit/${widget.productId}', body: {
        'sku': _sku.text,
        'attributes': {
          'name': _name.text,
          'price': num.tryParse(_price.text) ?? 0,
          'weight': num.tryParse(_weight.text) ?? 0,
          'short_description': _shortDescription.text,
          'description': _description.text,
          'status': 1,
          'visible_individually': 1,
        },
      });
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('The product has been updated successfully.')));
        context.go('/admin/products');
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
      }
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  Future<void> _uploadImage() async {
    final picked = await pickImage();
    if (picked == null || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      final uploaded = await admin.post('admin/upload', body: {
        'filename': picked.filename,
        'folder': 'product',
        'data': base64Encode(picked.bytes),
      });
      if (uploaded is! Map) return;
      final path = uploaded['path'] as String;
      final images = _images.map((image) => {'path': image['path']}).toList()
        ..add({'path': path});
      await admin.put('admin/catalog/products/edit/${widget.productId}', body: {
        'images': images,
      });
      await _load();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('The image has been uploaded.')));
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
      }
    }
  }

  Future<void> _saveInventories() async {
    final admin = context.read<AdminProvider>();
    final inventories = <String, dynamic>{};
    for (final inventory in _inventories) {
      final sourceId = inventory['inventory_source_id']?.toString();
      final controller = inventory['__controller'] as TextEditingController?;
      if (sourceId != null && controller != null) {
        inventories[sourceId] = int.tryParse(controller.text) ?? 0;
      }
    }
    await admin.put('admin/catalog/products/edit/${widget.productId}', body: {
      'inventories': inventories,
    });
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('The inventories have been updated.')));
    }
    await _load();
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 760),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                IconButton(
                  icon: const Icon(Icons.arrow_back),
                  onPressed: () => context.go('/admin/products'),
                ),
                Expanded(
                  child: Text('Edit product #${widget.productId}',
                      style: const TextStyle(
                          fontFamily: 'Poppins', fontSize: 22, fontWeight: FontWeight.w500)),
                ),
                PrimaryButton(
                  label: 'Save',
                  isLoading: _saving,
                  onPressed: _saving ? null : _save,
                ),
              ],
            ),
            const SizedBox(height: 24),
            _Section(
              title: 'General',
              children: [
                _field('SKU', _sku),
                _field('Name', _name),
                Row(children: [
                  Expanded(child: _field('Price', _price)),
                  const SizedBox(width: 12),
                  Expanded(child: _field('Weight', _weight)),
                ]),
                _field('Short description', _shortDescription, multiline: true),
                _field('Description', _description, multiline: true, lines: 5),
              ],
            ),
            const SizedBox(height: 16),
            _Section(
              title: 'Images',
              children: [
                Wrap(
                  spacing: 12,
                  runSpacing: 12,
                  children: [
                    ..._images.map((image) => Container(
                          width: 96,
                          height: 96,
                          decoration: BoxDecoration(
                            borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                            border: Border.all(color: AppColors.zinc200),
                          ),
                          child: const Icon(Icons.image_outlined,
                              color: AppColors.zinc300, size: 36),
                        )),
                    GestureDetector(
                      onTap: _uploadImage,
                      child: Container(
                        width: 96,
                        height: 96,
                        decoration: BoxDecoration(
                          borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                          color: AppColors.zinc100,
                        ),
                        child: const Icon(Icons.add_a_photo_outlined,
                            color: AppColors.zinc500, size: 28),
                      ),
                    ),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 16),
            _Section(
              title: 'Inventories',
              children: [
                ..._inventories.map((inventory) {
                  final controller = inventory.putIfAbsent(
                      '__controller',
                      () => TextEditingController(
                          text: '${inventory['qty'] ?? 0}')) as TextEditingController;
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 12),
                    child: Row(
                      children: [
                        Expanded(child: Text('${inventory['name'] ?? 'Source'}',
                            style: const TextStyle(fontFamily: 'Poppins', fontSize: 14))),
                        SizedBox(
                          width: 120,
                          child: TextField(
                            controller: controller,
                            keyboardType: TextInputType.number,
                            decoration: const InputDecoration(
                                isDense: true, fillColor: Colors.white),
                          ),
                        ),
                      ],
                    ),
                  );
                }),
                OutlinedButton(
                  onPressed: _saveInventories,
                  child: const Text('Update inventories',
                      style: TextStyle(fontFamily: 'Poppins')),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _field(String label, TextEditingController controller,
      {bool multiline = false, int lines = 2}) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
          const SizedBox(height: 4),
          TextField(
            controller: controller,
            maxLines: multiline ? lines : 1,
            decoration: const InputDecoration(
                fillColor: Colors.white, border: OutlineInputBorder()),
          ),
        ],
      ),
    );
  }
}

class _Section extends StatelessWidget {
  const _Section({required this.title, required this.children});
  final String title;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(title,
              style: const TextStyle(
                  fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500)),
          const SizedBox(height: 12),
          ...children,
        ],
      ),
    );
  }
}
