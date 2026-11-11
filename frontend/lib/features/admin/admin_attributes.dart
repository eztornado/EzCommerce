import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import 'admin_shell.dart' show EzDataGrid;

const _attributeTypes = <(String, String)>[
  ('text', 'Text'),
  ('textarea', 'Textarea'),
  ('price', 'Price'),
  ('number', 'Number'),
  ('date', 'Date'),
  ('datetime', 'Datetime'),
  ('boolean', 'Boolean'),
  ('select', 'Select'),
  ('multiselect', 'Multiselect'),
  ('checkbox', 'Checkbox'),
];

const _optionTypes = {'select', 'multiselect', 'checkbox'};

/// Atributos del catálogo: datagrid + editor (tipo, flags y opciones).
class AdminAttributesPage extends StatelessWidget {
  const AdminAttributesPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        EzDataGrid(
          gridPath: 'admin/catalog/attributes',
          title: 'Attributes',
          extraActions: (row) => [
            IconButton(
              icon: const Icon(Icons.edit_outlined, size: 18),
              tooltip: 'Edit',
              onPressed: () => showDialog(
                context: context,
                builder: (context) => AttributeEditorDialog(attributeId: row['id'] as int),
              ),
            ),
            if ((row['is_user_defined'] ?? 1) == 1)
              IconButton(
                icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                tooltip: 'Delete',
                onPressed: () async {
                  final admin = context.read<AdminProvider>();
                  final confirmed = await showDialog<bool>(
                    context: context,
                    builder: (context) => AlertDialog(
                      title: const Text('Delete attribute?'),
                      actions: [
                        TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
                        TextButton(onPressed: () => Navigator.pop(context, true), child: const Text('Delete')),
                      ],
                    ),
                  );
                  if (confirmed == true && context.mounted) {
                    try {
                      await admin.delete('admin/catalog/attributes/${row['id']}');
                    } catch (error) {
                      if (context.mounted) {
                        ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
                      }
                    }
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
            label: const Text('New Attribute'),
            onPressed: () => showDialog(
              context: context,
              builder: (context) => const AttributeEditorDialog(),
            ),
          ),
        ),
      ],
    );
  }
}

/// Editor de atributo en diálogo: código, nombre, tipo, flags y opciones.
class AttributeEditorDialog extends StatefulWidget {
  const AttributeEditorDialog({super.key, this.attributeId});
  final int? attributeId;

  @override
  State<AttributeEditorDialog> createState() => _AttributeEditorDialogState();
}

class _AttributeEditorDialogState extends State<AttributeEditorDialog> {
  final TextEditingController _code = TextEditingController();
  final TextEditingController _name = TextEditingController();
  String _type = 'text';
  final Map<String, bool> _flags = {
    'is_required': false,
    'is_visible_on_front': false,
    'is_configurable': false,
    'is_filterable': false,
    'value_per_locale': false,
    'value_per_channel': false,
  };
  final List<TextEditingController> _options = [];
  bool _loading = false;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    if (widget.attributeId != null) _load();
  }

  Future<void> _load() async {
    setState(() => _loading = true);
    final admin = context.read<AdminProvider>();
    final data = await admin.get('admin/catalog/attributes/edit/${widget.attributeId}');
    if (data is Map && mounted) {
      final attribute = Map<String, dynamic>.from((data['attribute'] ?? const {}) as Map);
      setState(() {
        _code.text = '${attribute['code'] ?? ''}';
        _name.text = '${attribute['admin_name'] ?? ''}';
        _type = '${attribute['type'] ?? 'text'}';
        for (final key in _flags.keys) {
          _flags[key] = (attribute[key] ?? 0) == 1;
        }
        for (final option in decodeList(data['options'])) {
          final controller = TextEditingController(text: '${option['admin_name'] ?? ''}');
          _options.add(controller);
        }
        _loading = false;
      });
    } else if (mounted) {
      setState(() => _loading = false);
    }
  }

  Future<void> _save() async {
    if (_code.text.trim().isEmpty || _name.text.trim().isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(
          content: const Text('Code and Name are required.'),
          backgroundColor: AppColors.red600));
      return;
    }
    setState(() => _saving = true);
    final admin = context.read<AdminProvider>();
    final body = {
      'code': _code.text.trim(),
      'admin_name': _name.text.trim(),
      'type': _type,
      for (final entry in _flags.entries) entry.key: entry.value ? 1 : 0,
      if (_optionTypes.contains(_type))
        'options': _options.map((option) => {'admin_name': option.text.trim()}).toList(),
    };
    try {
      widget.attributeId == null
          ? await admin.post('admin/catalog/attributes/store', body: body)
          : await admin.put('admin/catalog/attributes/edit/${widget.attributeId}', body: body);
      if (mounted) Navigator.of(context).pop();
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
    return AlertDialog(
      title: Text(widget.attributeId == null ? 'New attribute' : 'Edit attribute'),
      content: _loading
          ? const Center(child: CircularProgressIndicator(color: AppColors.navyBlue))
          : SingleChildScrollView(
              child: SizedBox(
                width: 460,
                child: Column(mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start, children: [
                  TextField(
                    controller: _code,
                    enabled: widget.attributeId == null,
                    decoration: const InputDecoration(labelText: 'Code (e.g. material)'),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                      controller: _name,
                      decoration: const InputDecoration(labelText: 'Name')),
                  const SizedBox(height: 12),
                  DropdownButtonFormField<String>(
                    initialValue: _type,
                    decoration: const InputDecoration(labelText: 'Type'),
                    items: _attributeTypes
                        .map((type) => DropdownMenuItem(value: type.$1, child: Text(type.$2)))
                        .toList(),
                    onChanged: (value) => setState(() => _type = value ?? 'text'),
                  ),
                  const SizedBox(height: 8),
                  Wrap(
                    spacing: 4,
                    runSpacing: -8,
                    children: [
                      for (final (label, key) in const [
                        ('Required', 'is_required'),
                        ('Visible on front', 'is_visible_on_front'),
                        ('Configurable', 'is_configurable'),
                        ('Filterable', 'is_filterable'),
                        ('Per locale', 'value_per_locale'),
                        ('Per channel', 'value_per_channel'),
                      ])
                        SizedBox(
                          width: 230,
                          child: CheckboxListTile(
                            dense: true,
                            controlAffinity: ListTileControlAffinity.leading,
                            contentPadding: EdgeInsets.zero,
                            title: Text(label,
                                style: const TextStyle(fontFamily: 'Poppins', fontSize: 12)),
                            value: _flags[key],
                            onChanged: (value) =>
                                setState(() => _flags[key] = value ?? false),
                          ),
                        ),
                    ],
                  ),
                  if (_optionTypes.contains(_type)) ...[
                    const SizedBox(height: 8),
                    Row(children: [
                      const Expanded(
                        child: Text('Options',
                            style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                      ),
                      TextButton.icon(
                        onPressed: () =>
                            setState(() => _options.add(TextEditingController())),
                        icon: const Icon(Icons.add, size: 16),
                        label: const Text('Add', style: TextStyle(fontFamily: 'Poppins')),
                      ),
                    ]),
                    for (final (index, option) in _options.indexed)
                      Padding(
                        padding: const EdgeInsets.only(bottom: 8),
                        child: Row(children: [
                          Expanded(
                            child: TextField(
                              controller: option,
                              decoration: const InputDecoration(
                                  isDense: true, labelText: 'Option'),
                            ),
                          ),
                          IconButton(
                            icon: const Icon(Icons.remove_circle_outline, size: 18),
                            onPressed: () =>
                                setState(() => _options.removeAt(index)),
                          ),
                        ]),
                      ),
                  ],
                ]),
              ),
            ),
      actions: [
        TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
        PrimaryButton(label: 'Save', isLoading: _saving, onPressed: _saving ? null : _save),
      ],
    );
  }
}
