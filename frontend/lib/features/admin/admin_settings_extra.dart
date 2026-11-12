import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../../../core/utils/download.dart';

/// Configuration: claves de core_config que la tienda y el motor de ventas
/// leen (oferta del topbar, envío plano, envío gratuito).
class AdminConfigurationPage extends StatefulWidget {
  const AdminConfigurationPage({super.key});

  @override
  State<AdminConfigurationPage> createState() => _AdminConfigurationPageState();
}

class _AdminConfigurationPageState extends State<AdminConfigurationPage> {
  final TextEditingController _offer = TextEditingController();
  final TextEditingController _flatRate = TextEditingController();
  final TextEditingController _flatRateType = TextEditingController();
  bool _flatRateActive = false;
  bool _freeShipping = false;
  bool _loading = true;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final general = await admin.get('admin/configuration/general.content');
    final sales = await admin.get('admin/configuration/sales.carriers');
    final rows = [...decodeList(general), ...decodeList(sales)];
    String config(String code) {
      for (final row in rows) {
        if (row['code'] == code) return '${row['value'] ?? ''}';
      }
      return '';
    }

    if (mounted) {
      setState(() {
        _offer.text = config('general.content.header_offer');
        _flatRate.text = config('sales.carriers.flatrate.default_rate');
        _flatRateType.text = config('sales.carriers.flatrate.type');
        _flatRateActive = config('sales.carriers.flatrate.active') == '1';
        _freeShipping = config('sales.carriers.free.active') == '1';
        _loading = false;
      });
    }
  }

  Future<void> _save() async {
    setState(() => _saving = true);
    final admin = context.read<AdminProvider>();
    try {
      await admin.post('admin/configuration', body: {
        'values': {
          'general.content.header_offer': _offer.text.trim(),
          'sales.carriers.flatrate.active': _flatRateActive ? '1' : '0',
          'sales.carriers.flatrate.default_rate': _flatRate.text.trim(),
          'sales.carriers.flatrate.type': _flatRateType.text.trim(),
          'sales.carriers.free.active': _freeShipping ? '1' : '0',
        }
      });
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(const SnackBar(
            content: Text('The configuration has been saved successfully.')));
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

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        const Text('Configuration',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
        const SizedBox(height: 24),
        _section('General', [
          TextField(
            controller: _offer,
            decoration: const InputDecoration(
                labelText: 'Header offer (topbar de la tienda)',
                helperText: 'Se muestra en la barra superior de la tienda'),
          ),
        ]),
        _section('Shipping', [
          SwitchListTile(
            dense: true,
            contentPadding: EdgeInsets.zero,
            title: const Text('Flat rate shipping',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
            value: _flatRateActive,
            onChanged: (value) => setState(() => _flatRateActive = value),
          ),
          Row(children: [
            Expanded(
              child: TextField(
                controller: _flatRate,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(labelText: 'Flat rate price (EUR)'),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: TextField(
                controller: _flatRateType,
                decoration: const InputDecoration(
                    labelText: 'Type (per_unit / order)',
                    helperText: 'per_unit multiplica por unidades'),
              ),
            ),
          ]),
          SwitchListTile(
            dense: true,
            contentPadding: EdgeInsets.zero,
            title: const Text('Free shipping available',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
            value: _freeShipping,
            onChanged: (value) => setState(() => _freeShipping = value),
          ),
        ]),
        const SizedBox(height: 16),
        PrimaryButton(
          label: 'Save configuration',
          isLoading: _saving,
          onPressed: _saving ? null : _save,
        ),
      ],
    );
  }

  Widget _section(String title, List<Widget> children) {
    return Container(
      margin: const EdgeInsets.only(bottom: 16),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text(title,
            style: const TextStyle(
                fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500)),
        const SizedBox(height: 12),
        ...children,
      ]),
    );
  }
}

/// Data transfer: exportaciones CSV y registro del último import.
class AdminDataTransferPage extends StatelessWidget {
  const AdminDataTransferPage({super.key});

  Future<void> _download(BuildContext context, String path, String filename) async {
    final token = ApiClient.instance.adminToken;
    if (token == null) return;
    final response = await http.get(
      Uri.parse('${ApiClient.baseUrl}/api/$path'),
      headers: {'Authorization': 'Bearer $token'},
    );
    if (response.statusCode != 200 || !context.mounted) return;
    downloadFile(filename, response.bodyBytes);
    ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('$filename downloaded.')));
  }

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        const Text('Data Transfer',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
        const SizedBox(height: 24),
        _card(context, 'Export products (CSV)', () => _download(context, 'admin/data-transfer/products/export', 'products.csv')),
        _card(context, 'Export customers (CSV)', () => _download(context, 'admin/data-transfer/export-customers', 'customers.csv')),
        _card(context, 'Export orders (CSV)', () => _download(context, 'admin/data-transfer/export-orders', 'orders.csv')),
        _card(context, 'Import products (CSV file)', () async {
          final admin = context.read<AdminProvider>();
          try {
            final result = await admin.post('admin/data-transfer/products/import');
            if (context.mounted) {
              ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                  content: Text('${result is Map ? result['message'] ?? '' : ''}')));
            }
          } catch (error) {
            if (context.mounted) {
              ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
            }
          }
        }),
      ],
    );
  }

  Widget _card(BuildContext context, String label, VoidCallback action) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Row(children: [
        Expanded(child: Text(label, style: const TextStyle(fontFamily: 'Poppins', fontSize: 14))),
        OutlinedButton(onPressed: action, child: const Text('Run')),
      ]),
    );
  }
}

/// Familias de atributos: listado con alta/edición/baja.
class AdminFamiliesPage extends StatefulWidget {
  const AdminFamiliesPage({super.key});

  @override
  State<AdminFamiliesPage> createState() => _AdminFamiliesPageState();
}

class _AdminFamiliesPageState extends State<AdminFamiliesPage> {
  List<Map<String, dynamic>> _families = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final data = await admin.get('admin/catalog/families');
    if (mounted) {
      setState(() {
        _families = decodeList(data);
        _loading = false;
      });
    }
  }

  void _showError(Object error) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
  }

  Future<void> _familyDialog([Map<String, dynamic>? family]) async {
    final code = TextEditingController(text: '${family?['code'] ?? ''}');
    final name = TextEditingController(text: '${family?['name'] ?? ''}');
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(family == null ? 'New attribute family' : 'Edit attribute family'),
        content: SizedBox(
          width: 420,
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            TextField(
              controller: code,
              enabled: family == null,
              decoration: const InputDecoration(labelText: 'Code'),
            ),
            const SizedBox(height: 12),
            TextField(controller: name, decoration: const InputDecoration(labelText: 'Name')),
          ]),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
          PrimaryButton(
            label: 'Save',
            onPressed: () => (code.text.trim().isEmpty || name.text.trim().isEmpty)
                ? null
                : Navigator.pop(context, true),
          ),
        ],
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      family == null
          ? await admin.post('admin/catalog/families/store', body: {
              'code': code.text.trim(),
              'name': name.text.trim(),
            })
          : await admin.put('admin/catalog/families/edit/${family['id']}', body: {
              'name': name.text.trim(),
            });
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _delete(Map<String, dynamic> family) async {
    final admin = context.read<AdminProvider>();
    try {
      await admin.delete('admin/catalog/families/${family['id']}');
      await _load();
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [
          const Expanded(
            child: Text('Attribute Families',
                style: TextStyle(
                    fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
          ),
          TextButton.icon(
            onPressed: () => _familyDialog(),
            icon: const Icon(Icons.add, size: 18),
            label: const Text('New Family', style: TextStyle(fontFamily: 'Poppins')),
          ),
        ]),
        const SizedBox(height: 24),
        for (final family in _families)
          Container(
            margin: const EdgeInsets.only(bottom: 12),
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(EzTheme.radiusXl),
              border: Border.all(color: AppColors.zinc200),
            ),
            child: Row(children: [
              Expanded(
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  Text('${family['name']}',
                      style: const TextStyle(
                          fontFamily: 'Poppins', fontSize: 14, fontWeight: FontWeight.w500)),
                  Text('code: ${family['code']}',
                      style: TextStyle(
                          fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                ]),
              ),
              IconButton(
                icon: const Icon(Icons.edit_outlined, size: 18),
                onPressed: (family['is_user_defined'] ?? 1) == 1
                    ? () => _familyDialog(family)
                    : null,
              ),
              IconButton(
                icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                onPressed: (family['is_user_defined'] ?? 1) == 1
                    ? () => _delete(family)
                    : null,
              ),
            ]),
          ),
      ]),
    );
  }
}
