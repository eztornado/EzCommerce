import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../../../core/utils/pick_image.dart';

/// Editor de producto del admin: atributos base (nombre, precio,
/// descripciones), categorías, inventario por almacén, imágenes (subida
/// base64) y variantes de configurables.
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
  List<Map<String, dynamic>> _superAttributes = [];
  List<Map<String, dynamic>> _variants = [];
  List<Map<String, dynamic>> _categoriesTree = [];
  Set<int> _selectedCategories = {};
  bool _loading = true;
  bool _saving = false;

  late final TextEditingController _sku;
  late final TextEditingController _name;
  late final TextEditingController _price;
  late final TextEditingController _shortDescription;
  late final TextEditingController _description;
  late final TextEditingController _weight;

  bool get _isConfigurable => _product?['type'] == 'configurable';

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
    final admin = context.read<AdminProvider>();
    final data = await admin.get('admin/catalog/products/edit/${widget.productId}');
    final tree = await admin.get('admin/catalog/categories/tree');
    if (data is Map && mounted) {
      setState(() {
        _product = Map<String, dynamic>.from(data['product'] as Map);
        _attributes = Map<String, dynamic>.from((data['attributes'] ?? const {}) as Map);
        _inventories = decodeList(data['inventories']);
        _images = decodeList(data['images']);
        _superAttributes = decodeList(data['super_attributes']);
        _variants = decodeList(data['variants']);
        _categoriesTree = decodeList(tree);
        _selectedCategories = decodeList(data['categories'])
            .map((row) => (row['category_id'] ?? row['id']) as int)
            .whereType<int>()
            .toSet();
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
        'categories': _selectedCategories.toList(),
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

  Future<void> _deleteImage(int index) async {
    final admin = context.read<AdminProvider>();
    final images = _images.map((image) => {'path': image['path']}).toList()
      ..removeAt(index);
    await admin.put('admin/catalog/products/edit/${widget.productId}', body: {
      'images': images,
    });
    await _load();
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

  /// Resumen de la variante: valores de los super attributes (por código).
  String _variantLabel(Map<String, dynamic> variant) {
    final attributes = Map<String, dynamic>.from((variant['attributes'] ?? const {}) as Map);
    final parts = <String>[];
    for (final superAttribute in _superAttributes) {
      final code = '${superAttribute['code']}';
      final value = attributes[code];
      if (value == null) continue;
      // El valor puede ser el id de opción: traducir con las opciones.
      String label = '$value';
      final options = decodeList(superAttribute['options']);
      for (final option in options) {
        if (option['id'] == value) label = '${option['admin_name']}';
      }
      parts.add(label);
    }
    return parts.isEmpty ? '${variant['sku']}' : parts.join(' / ');
  }

  Future<void> _createVariant() async {
    if (_superAttributes.isEmpty) return;
    final sku = TextEditingController();
    final price = TextEditingController(text: _price.text);
    final qty = TextEditingController(text: '10');
    final optionSelections = <String, int?>{
      for (final superAttribute in _superAttributes)
        '${superAttribute['code']}': null,
    };
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: const Text('New variant'),
          content: SingleChildScrollView(
            child: SizedBox(
              width: 420,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextField(
                    controller: sku,
                    decoration: const InputDecoration(labelText: 'SKU'),
                  ),
                  const SizedBox(height: 12),
                  for (final superAttribute in _superAttributes)
                    Padding(
                      padding: const EdgeInsets.only(bottom: 12),
                      child: DropdownButtonFormField<int>(
                        initialValue: optionSelections['${superAttribute['code']}'],
                        decoration: InputDecoration(
                            labelText: '${superAttribute['admin_name'] ?? superAttribute['code']}'),
                        items: decodeList(superAttribute['options'])
                            .map((option) => DropdownMenuItem(
                                  value: option['id'] as int,
                                  child: Text('${option['admin_name']}'),
                                ))
                            .toList(),
                        onChanged: (value) => setDialogState(() =>
                            optionSelections['${superAttribute['code']}'] = value),
                      ),
                    ),
                  Row(children: [
                    Expanded(
                        child: TextField(
                            controller: price,
                            keyboardType: TextInputType.number,
                            decoration: const InputDecoration(labelText: 'Price'))),
                    const SizedBox(width: 12),
                    Expanded(
                        child: TextField(
                            controller: qty,
                            keyboardType: TextInputType.number,
                            decoration: const InputDecoration(labelText: 'Qty'))),
                  ]),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(onPressed: () => Navigator.of(context).pop(false), child: const Text('Cancel')),
            PrimaryButton(
              label: 'Create',
              onPressed: () {
                if (sku.text.trim().isEmpty ||
                    optionSelections.values.any((value) => value == null)) {
                  return;
                }
                Navigator.of(context).pop(true);
              },
            ),
          ],
        ),
      ),
    );
    if (saved != true || !mounted) return;

    final admin = context.read<AdminProvider>();
    try {
      await admin.post('admin/catalog/products/edit/${widget.productId}/variants', body: {
        'sku': sku.text.trim(),
        'attributes': {
          'name': '${_name.text} ${optionSelections.values.map((v) => v).join(' ')}'.trim(),
          'price': num.tryParse(price.text) ?? 0,
          'weight': num.tryParse(_weight.text) ?? 0,
          for (final entry in optionSelections.entries) entry.key: entry.value,
        },
        'qty': int.tryParse(qty.text) ?? 0,
      });
      await _load();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('The variant has been created successfully.')));
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
      }
    }
  }

  Future<void> _deleteVariant(int variantId) async {
    final admin = context.read<AdminProvider>();
    await admin.delete('admin/catalog/products/variants/$variantId');
    await _load();
  }

  /// Campos del árbol de categorías excluyendo la raíz (id 1).
  List<Map<String, dynamic>> _flattenCategories(
      List<Map<String, dynamic>> nodes, bool isRoot) {
    final result = <Map<String, dynamic>>[];
    for (final node in nodes) {
      final id = node['id'] as int?;
      final children = decodeList(node['children']);
      if (!isRoot && id != null) result.add(node);
      result.addAll(_flattenCategories(children, false));
    }
    return result;
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    final categoryNodes = _flattenCategories(_categoriesTree, true);
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
                    for (final (index, image) in _images.indexed)
                      Stack(children: [
                        Container(
                          width: 96,
                          height: 96,
                          decoration: BoxDecoration(
                            borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                            border: Border.all(color: AppColors.zinc200),
                          ),
                          child: ClipRRect(
                            borderRadius: BorderRadius.circular(EzTheme.radiusLg - 1),
                            child: Image.network(
                              apiUrlRelative('${image['path']}') ?? '',
                              fit: BoxFit.cover,
                              errorBuilder: (_, _, _) => const Icon(Icons.image_outlined,
                                  color: AppColors.zinc300, size: 36),
                            ),
                          ),
                        ),
                        Positioned(
                          right: -8,
                          top: -8,
                          child: IconButton(
                            icon: const Icon(Icons.remove_circle, color: AppColors.red600, size: 22),
                            onPressed: () => _deleteImage(index),
                          ),
                        ),
                      ]),
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
              title: 'Categories',
              children: [
                if (categoryNodes.isEmpty)
                  const Text('No categories yet.',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc500))
                else
                  Wrap(
                    spacing: 16,
                    runSpacing: 8,
                    children: [
                      for (final category in categoryNodes)
                        SizedBox(
                          width: 220,
                          child: CheckboxListTile(
                            dense: true,
                            contentPadding: EdgeInsets.zero,
                            controlAffinity: ListTileControlAffinity.leading,
                            title: Text('${category['name'] ?? category['slug']}',
                                style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                            value: _selectedCategories.contains(category['id']),
                            onChanged: (checked) => setState(() {
                              final id = category['id'] as int;
                              checked == true
                                  ? _selectedCategories.add(id)
                                  : _selectedCategories.remove(id);
                            }),
                          ),
                        ),
                    ],
                  ),
              ],
            ),
            if (_isConfigurable) ...[
              const SizedBox(height: 16),
              _Section(
                title: 'Variants',
                children: [
                  if (_variants.isEmpty)
                    const Text('No variants yet.',
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc500))
                  else
                    ..._variants.map((variant) => Padding(
                          padding: const EdgeInsets.only(bottom: 8),
                          child: Row(
                            children: [
                              Expanded(
                                child: Text(
                                    '${_variantLabel(variant)}  ·  ${variant['sku']}  ·  '
                                    '${(Map<String, dynamic>.from((variant['attributes'] ?? const {}) as Map))['price'] ?? 0}',
                                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                              ),
                              Text('Qty: ${variant['qty'] ?? 0}',
                                  style: const TextStyle(
                                      fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                              IconButton(
                                icon: const Icon(Icons.delete_outline, size: 18),
                                tooltip: 'Delete variant',
                                onPressed: () => _deleteVariant(variant['id'] as int),
                              ),
                            ],
                          ),
                        )),
                  const SizedBox(height: 8),
                  OutlinedButton.icon(
                    onPressed: _createVariant,
                    icon: const Icon(Icons.add),
                    label: const Text('New variant', style: TextStyle(fontFamily: 'Poppins')),
                  ),
                ],
              ),
            ],
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
