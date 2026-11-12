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
        EzDataGrid(
          gridPath: 'admin/marketing/catalog-rules',
          title: 'Catalog Rules',
          extraActions: (row) => [
            IconButton(
              icon: const Icon(Icons.edit_outlined, size: 18),
              tooltip: 'Edit',
              onPressed: () async {
                final admin = context.read<AdminProvider>();
                final data = await admin.get('admin/marketing/catalog-rules/view/${row['id']}');
                if (data is! Map || !context.mounted) return;
                final rule = Map<String, dynamic>.from((data['data'] ?? const {}) as Map);
                if (context.mounted) {
                  showDialog(
                    context: context,
                    builder: (context) => CatalogRuleEditorDialog(rule: rule),
                  );
                }
              },
            ),
            IconButton(
              icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
              tooltip: 'Delete',
              onPressed: () async {
                final admin = context.read<AdminProvider>();
                final confirmed = await showDialog<bool>(
                  context: context,
                  builder: (context) => AlertDialog(
                    title: const Text('Delete rule?'),
                    actions: [
                      TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
                      TextButton(onPressed: () => Navigator.pop(context, true), child: const Text('Delete')),
                    ],
                  ),
                );
                if (confirmed == true && context.mounted) {
                  await admin.delete('admin/marketing/catalog-rules/${row['id']}');
                }
              },
            ),
          ],
        ),
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
  List<Map<String, dynamic>> _categoriesTree = [];
  Set<int> _selectedCategories = {};
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    final rule = widget.rule ?? const {};
    _name = TextEditingController(text: '${rule['name'] ?? ''}');
    _description = TextEditingController(text: '${rule['description'] ?? ''}');
    _discountAmount = TextEditingController(text: '${rule['discount_amount'] ?? 15}');
    _actionType = '${rule['action_type'] ?? 'by_percent'}';
    // Condición existente product::categories -> categorías seleccionadas.
    for (final condition in decodeList(rule['conditions'])) {
      if (condition['attribute'] == 'product::categories') {
        _selectedCategories = {
          ...decodeIds(condition['value']),
        };
      }
    }
    _loadCategories();
  }

  Future<void> _loadCategories() async {
    final admin = context.read<AdminProvider>();
    final data = await admin.get('admin/catalog/categories/tree');
    if (data is List && mounted) {
      setState(() => _categoriesTree = decodeList(data));
    } else if (data is Map && mounted) {
      setState(() => _categoriesTree = decodeList(data['data']));
    }
  }

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

  Future<void> _save() async {
    setState(() => _saving = true);
    final admin = context.read<AdminProvider>();
    final body = {
      'name': _name.text,
      'description': _description.text,
      'status': 1,
      'action_type': _actionType,
      'discount_amount': num.tryParse(_discountAmount.text) ?? 0,
      'conditions': _selectedCategories.isEmpty
          ? []
          : [
              {'attribute': 'product::categories', 'value': _selectedCategories.toList()},
            ],
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
    final categoryNodes = _flattenCategories(_categoriesTree, true);
    return AlertDialog(
      title: Text(widget.rule == null ? 'New Catalog Rule' : 'Edit Catalog Rule'),
      content: SingleChildScrollView(
        child: SizedBox(
          width: 440,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
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
              const SizedBox(height: 16),
              const Text('Limit to categories',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
              const SizedBox(height: 4),
              Wrap(
                spacing: 8,
                runSpacing: 4,
                children: [
                  for (final category in categoryNodes)
                    FilterChip(
                      label: Text('${category['name'] ?? category['slug']}',
                          style: const TextStyle(fontFamily: 'Poppins', fontSize: 12)),
                      selected: _selectedCategories.contains(category['id']),
                      onSelected: (selected) => setState(() {
                        final id = category['id'] as int;
                        selected
                            ? _selectedCategories.add(id)
                            : _selectedCategories.remove(id);
                      }),
                    ),
                ],
              ),
              if (categoryNodes.isEmpty)
                const Text('No categories yet.',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
        PrimaryButton(label: 'Save', isLoading: _saving, onPressed: _saving ? null : _save),
      ],
    );
  }
}
