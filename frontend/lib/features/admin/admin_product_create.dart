import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';

const _productTypes = <(String, String)>[
  ('simple', 'Simple'),
  ('virtual', 'Virtual'),
  ('configurable', 'Configurable'),
  ('grouped', 'Grouped'),
  ('bundle', 'Bundle'),
  ('downloadable', 'Downloadable'),
];

/// Creación de producto (estilo Bagisto): tipo + SKU + datos mínimos; al
/// guardar redirige al editor completo.
class AdminProductCreatePage extends StatefulWidget {
  const AdminProductCreatePage({super.key});

  @override
  State<AdminProductCreatePage> createState() => _AdminProductCreatePageState();
}

class _AdminProductCreatePageState extends State<AdminProductCreatePage> {
  String _type = 'simple';
  final TextEditingController _sku = TextEditingController();
  final TextEditingController _name = TextEditingController();
  final TextEditingController _price = TextEditingController();
  List<Map<String, dynamic>> _configurableAttributes = [];
  final Set<int> _selectedSuperAttributes = {};
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    _loadConfigurableAttributes();
  }

  Future<void> _loadConfigurableAttributes() async {
    final data = await context.read<AdminProvider>().get('admin/catalog/attributes/configurable');
    if (data is Map && mounted) {
      setState(() => _configurableAttributes = decodeList(data['data']));
    }
  }

  static String _urlKey(String name) {
    final slug = name
        .toLowerCase()
        .trim()
        .replaceAll(RegExp(r'[^a-z0-9]+'), '-')
        .replaceAll(RegExp(r'^-+|-+$'), '');
    return slug.isEmpty ? 'product-${DateTime.now().millisecondsSinceEpoch}' : slug;
  }

  Future<void> _save() async {
    if (_sku.text.trim().isEmpty || _name.text.trim().isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(
          content: const Text('SKU and Name are required.'),
          backgroundColor: AppColors.red600));
      return;
    }
    if (_type == 'configurable' && _selectedSuperAttributes.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(
          content: const Text('Select at least one configurable attribute.'),
          backgroundColor: AppColors.red600));
      return;
    }
    setState(() => _saving = true);
    final admin = context.read<AdminProvider>();
    try {
      final created = await admin.post('admin/catalog/products/create', body: {
        'sku': _sku.text.trim(),
        'type': _type,
        if (_type == 'configurable') 'super_attributes': _selectedSuperAttributes.toList(),
      });
      final productId = created is Map ? (created['data']?['id'] as int?) : null;
      if (productId == null) {
        throw Exception('Unexpected response from the API.');
      }
      await admin.put('admin/catalog/products/edit/$productId', body: {
        'attributes': {
          'name': _name.text.trim(),
          'price': num.tryParse(_price.text) ?? 0,
          'weight': 0,
          'status': 1,
          'visible_individually': 1,
          'manage_stock': 1,
          'url_key': _urlKey(_name.text.trim()),
        },
      });
      if (mounted) context.go('/admin/products/edit/$productId');
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
        setState(() => _saving = false);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 640),
        child: Container(
          padding: const EdgeInsets.all(24),
          decoration: BoxDecoration(
            color: Colors.white,
            borderRadius: BorderRadius.circular(EzTheme.radiusXl),
            border: Border.all(color: AppColors.zinc200),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(children: [
                IconButton(
                  icon: const Icon(Icons.arrow_back),
                  onPressed: () => context.go('/admin/products'),
                ),
                const Expanded(
                  child: Text('New product',
                      style: TextStyle(
                          fontFamily: 'Poppins', fontSize: 22, fontWeight: FontWeight.w500)),
                ),
              ]),
              const SizedBox(height: 20),
              const Text('Product type',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
              const SizedBox(height: 8),
              Wrap(
                spacing: 12,
                runSpacing: 12,
                children: [
                  for (final (value, label) in _productTypes)
                    ChoiceChip(
                      label: Text(label, style: const TextStyle(fontFamily: 'Poppins')),
                      selected: _type == value,
                      selectedColor: AppColors.navyBlue,
                      labelStyle: TextStyle(
                          fontFamily: 'Poppins',
                          color: _type == value ? Colors.white : AppColors.zinc600),
                      onSelected: (_) => setState(() => _type = value),
                    ),
                ],
              ),
              const SizedBox(height: 20),
              if (_type == 'configurable') ...[
                const Text('Configurable attributes',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                const SizedBox(height: 8),
                Wrap(
                  spacing: 12,
                  runSpacing: 8,
                  children: [
                    for (final attribute in _configurableAttributes)
                      FilterChip(
                        label: Text('${attribute['admin_name'] ?? attribute['code']}',
                            style: const TextStyle(fontFamily: 'Poppins')),
                        selected: _selectedSuperAttributes.contains(attribute['id']),
                        selectedColor: AppColors.navyBlue,
                        checkmarkColor: Colors.white,
                        labelStyle: TextStyle(
                            fontFamily: 'Poppins',
                            color: _selectedSuperAttributes.contains(attribute['id'])
                                ? Colors.white
                                : AppColors.zinc600),
                        onSelected: (selected) => setState(() {
                          final id = attribute['id'] as int;
                          selected
                              ? _selectedSuperAttributes.add(id)
                              : _selectedSuperAttributes.remove(id);
                        }),
                      ),
                  ],
                ),
                const SizedBox(height: 20),
              ],
              _field('SKU', _sku),
              _field('Name', _name),
              _field('Price', _price, number: true),
              const SizedBox(height: 8),
              PrimaryButton(
                label: 'Create product',
                isLoading: _saving,
                onPressed: _saving ? null : _save,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _field(String label, TextEditingController controller,
      {bool number = false}) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
          const SizedBox(height: 4),
          TextField(
            controller: controller,
            keyboardType: number ? TextInputType.number : null,
            decoration: const InputDecoration(
                fillColor: Colors.white, border: OutlineInputBorder()),
          ),
        ],
      ),
    );
  }
}
