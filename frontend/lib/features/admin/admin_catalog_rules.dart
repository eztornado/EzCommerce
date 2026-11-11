import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import 'admin_shell.dart' show EzDataGrid;

/// Reglas de catálogo: datagrid + editor (acción sobre el precio mostrado).
class AdminCatalogRulesPage extends StatelessWidget {
  const AdminCatalogRulesPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        EzDataGrid(gridPath: 'admin/marketing/catalog-rules', title: 'Catalog Rules'),
        Positioned(
          right: 24,
          bottom: 24,
          child: FloatingActionButton.extended(
            backgroundColor: AppColors.navyBlue,
            foregroundColor: Colors.white,
            icon: const Icon(Icons.add),
            label: const Text('New Rule'),
            onPressed: () => showDialog(
              context: context,
              builder: (context) => const CatalogRuleEditorDialog(),
            ),
          ),
        ),
      ],
    );
  }
}

class CatalogRuleEditorDialog extends StatefulWidget {
  const CatalogRuleEditorDialog({super.key, this.rule});
  final Map<String, dynamic>? rule;

  @override
  State<CatalogRuleEditorDialog> createState() => _CatalogRuleEditorDialogState();
}

class _CatalogRuleEditorDialogState extends State<CatalogRuleEditorDialog> {
  late final TextEditingController _name;
  late final TextEditingController _description;
  late final TextEditingController _discountAmount;
  String _actionType = 'by_percent';
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    final rule = widget.rule ?? const {};
    _name = TextEditingController(text: '${rule['name'] ?? ''}');
    _description = TextEditingController(text: '${rule['description'] ?? ''}');
    _discountAmount = TextEditingController(text: '${rule['discount_amount'] ?? 15}');
    _actionType = '${rule['action_type'] ?? 'by_percent'}';
  }

  Future<void> _save() async {
    setState(() => _saving = true);
    final admin = context.read<AdminProvider>();
    final body = {
      'name': _name.text,
      'description': _description.text,
      'status': 1,
      'action_type': _actionType,
      'discount_amount': num.tryParse(_discountAmount.text) ?? 0,
      'sort': 0,
    };
    try {
      if (widget.rule == null) {
        await admin.post('admin/marketing/catalog-rules/store', body: body);
      } else {
        await admin.put('admin/marketing/catalog-rules/edit/${widget.rule!['id']}', body: body);
      }
      if (mounted) Navigator.of(context).pop();
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
      }
      setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(widget.rule == null ? 'New Catalog Rule' : 'Edit Catalog Rule'),
      content: SizedBox(
        width: 440,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: _name,
              decoration: const InputDecoration(labelText: 'Name'),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _description,
              maxLines: 2,
              decoration: const InputDecoration(labelText: 'Description'),
            ),
            const SizedBox(height: 12),
            Row(children: [
              Expanded(
                child: DropdownButtonFormField<String>(
                  initialValue: _actionType,
                  decoration: const InputDecoration(labelText: 'Action'),
                  items: const [
                    DropdownMenuItem(value: 'by_percent', child: Text('- Percentage')),
                    DropdownMenuItem(value: 'to_percent', child: Text('Set to % of price')),
                    DropdownMenuItem(value: 'by_fixed', child: Text('- Fixed amount')),
                    DropdownMenuItem(value: 'to_fixed', child: Text('Set to fixed price')),
                  ],
                  onChanged: (value) => setState(() => _actionType = value ?? 'by_percent'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: TextField(
                  controller: _discountAmount,
                  keyboardType: TextInputType.number,
                  decoration: const InputDecoration(labelText: 'Amount'),
                ),
              ),
            ]),
          ],
        ),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
        PrimaryButton(label: 'Save', isLoading: _saving, onPressed: _saving ? null : _save),
      ],
    );
  }
}
