import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';

/// Atributos fijos del carrito y del artículo disponibles como condiciones
/// (el motor del backend los evalúa: `cart::`, `cart_item::`, `product::`).
const _builtinAttributes = <(String, String, String)>[
  // (attribute, label, kind: number | text | guest)
  ('cart::base_sub_total', 'Cart subtotal', 'number'),
  ('cart::base_grand_total', 'Cart total', 'number'),
  ('cart::items_qty', 'Items quantity', 'number'),
  ('cart::customer_id', 'Only guests', 'guest'),
  ('cart_item::quantity', 'Item quantity', 'number'),
  ('cart_item::base_price', 'Item price', 'number'),
  ('cart_item::sku', 'Item SKU', 'text'),
];

const _numberOperators = ['==', '!=', '>=', '<=', '>', '<'];
const _textOperators = ['==', '!=', '{}', '!{}'];
const _selectOperators = ['==', '!='];

class ConditionRow {
  ConditionRow({required this.attribute, required this.operator, this.value});
  final String attribute;
  final String operator;
  final dynamic value;

  Map<String, dynamic> toPayload() => {'attribute': attribute, 'operator': operator, 'value': value};

  static ConditionRow fromCondition(Map<String, dynamic> payload) => ConditionRow(
        attribute: '${payload['attribute'] ?? ''}',
        operator: '${payload['operator'] ?? '=='}',
        value: payload['value'],
      );

  static ConditionRow fromPayload(Map<String, dynamic> payload) => ConditionRow(
        attribute: '${payload['attribute'] ?? ''}',
        operator: '${payload['operator'] ?? '=='}',
        value: payload['value'],
      );
}

/// Editor visual de condiciones de reglas de carrito: filas
/// atributo + operador + valor con union ALL/ANY. Los atributos de producto
/// (EAV) se cargan del API; los atributos desconocidos se conservan sin editar.
class CartConditionsEditor extends StatefulWidget {
  const CartConditionsEditor({
    super.key,
    required this.conditionType,
    required this.conditions,
    required this.onChanged,
  });

  final int conditionType;
  final List<Map<String, dynamic>> conditions;
  final void Function(int conditionType, List<Map<String, dynamic>> conditions) onChanged;

  @override
  State<CartConditionsEditor> createState() => _CartConditionsEditorState();
}

class _CartConditionsEditorState extends State<CartConditionsEditor> {
  List<Map<String, dynamic>> _attributes = [];
  final Map<String, List<Map<String, dynamic>>> _optionsCache = {};
  List<_EditableRow> _rows = [];
  bool _loading = true;

  List<Map<String, dynamic>> get _conditions => List.from(widget.conditions);

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final data = await admin.get('admin/catalog/attributes/all');
    if (!mounted) return;
    setState(() {
      _attributes = decodeList(data);
      _rows = [
        for (final condition in widget.conditions) _EditableRow.fromCondition(condition),
      ];
      _loading = false;
    });
    // Precarga de opciones para las filas select existentes.
    for (final row in _rows) {
      if (row.kind == 'select') _loadOptions(row.attribute);
    }
  }

  Future<void> _loadOptions(String attribute) async {
    if (_optionsCache.containsKey(attribute)) return;
    final match = _attributes.where((entry) => 'product::${entry['code']}' == attribute).firstOrNull;
    if (match == null) return;
    final admin = context.read<AdminProvider>();
    final data = await admin.get('admin/catalog/attributes/edit/${match['id']}');
    if (data is Map && mounted) {
      setState(() => _optionsCache[attribute] = decodeList(data['options']));
    }
  }

  void _emit() {
    final payload = <Map<String, dynamic>>[
      for (final row in _rows) ...[
        if (row.attribute.isEmpty)
          if (row.unknownCondition != null) row.unknownCondition!.toPayload(),
        if (row.attribute.isNotEmpty)
          ConditionRow(
            attribute: row.attribute,
            operator: row.operator,
            value: row.kind == 'guest'
                ? 0
                : row.kind == 'select'
                    ? row.optionValue
                    : row.kind == 'number'
                        ? (num.tryParse(row.valueController.text) ?? 0)
                        : row.valueController.text,
          ).toPayload(),
      ],
    ];
    widget.onChanged(widget.conditionType, payload);
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(
          child: SizedBox(
              width: 18,
              height: 18,
              child: CircularProgressIndicator(color: AppColors.navyBlue, strokeWidth: 2)));
    }
    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      Row(children: [
        const Text('Conditions match',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
        const SizedBox(width: 12),
        ChoiceChip(
          label: const Text('ALL', style: TextStyle(fontFamily: 'Poppins', fontSize: 12)),
          selected: widget.conditionType == 1,
          selectedColor: AppColors.navyBlue,
          labelStyle: TextStyle(
              fontFamily: 'Poppins', color: widget.conditionType == 1 ? Colors.white : AppColors.zinc600),
          onSelected: (_) {
            widget.onChanged(1, _conditions);
          },
        ),
        const SizedBox(width: 8),
        ChoiceChip(
          label: const Text('ANY', style: TextStyle(fontFamily: 'Poppins', fontSize: 12)),
          selected: widget.conditionType == 2,
          selectedColor: AppColors.navyBlue,
          labelStyle: TextStyle(
              fontFamily: 'Poppins', color: widget.conditionType == 2 ? Colors.white : AppColors.zinc600),
          onSelected: (_) {
            widget.onChanged(2, _conditions);
          },
        ),
      ]),
      const SizedBox(height: 12),
      if (_rows.isEmpty)
        const Text('No conditions: the rule applies to everything.',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500))
      else
        for (final (index, row) in _rows.indexed)
          _rowTile(index, row),
      const SizedBox(height: 4),
      TextButton.icon(
        onPressed: () => setState(() {
          _rows.add(_EditableRow());
        }),
        icon: const Icon(Icons.add, size: 16),
        label: const Text('Add condition', style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
      ),
    ]);
  }

  Widget _rowTile(int index, _EditableRow row) {
    final operators = switch (row.kind) {
      'number' => _numberOperators,
      'guest' => ['=='],
      'select' => _selectOperators,
      'text' => _textOperators,
      _ => const <String>[],
    };
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        border: Border.all(color: AppColors.zinc200),
        borderRadius: BorderRadius.circular(10),
        color: Colors.white,
      ),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [
          Expanded(
            child: DropdownButtonFormField<String>(
              isDense: true,
              initialValue: row.attribute.isEmpty ? null : row.attribute,
              decoration: const InputDecoration(
                  isDense: true, labelText: 'Attribute', border: OutlineInputBorder()),
              items: [
                for (final (attribute, label, _) in _builtinAttributes)
                  DropdownMenuItem(value: attribute, child: Text(label,
                      style: const TextStyle(fontFamily: 'Poppins', fontSize: 12))),
                for (final attribute in _attributes)
                  DropdownMenuItem(
                      value: 'product::${attribute['code']}',
                      child: Text('Product: ${attribute['admin_name'] ?? attribute['code']}',
                          style: const TextStyle(fontFamily: 'Poppins', fontSize: 12))),
              ],
              onChanged: (value) {
                setState(() {
                  row.attribute = value ?? '';
                  row.operator = '==';
                  row.optionValue = null;
                  row.valueController.clear();
                  if (row.kind == 'select') _loadOptions(row.attribute);
                });
                _emit();
              },
            ),
          ),
          const SizedBox(width: 8),
          if (row.kind != 'guest')
            SizedBox(
              width: 86,
              child: DropdownButtonFormField<String>(
                isDense: true,
                initialValue: operators.contains(row.operator) ? row.operator : operators.firstOrNull,
                decoration: const InputDecoration(
                    isDense: true, labelText: 'Op', border: OutlineInputBorder()),
                items: operators
                    .map((operator) => DropdownMenuItem(
                        value: operator,
                        child: Text(operator, style: const TextStyle(fontFamily: 'Poppins', fontSize: 12))))
                    .toList(),
                onChanged: (value) {
                  setState(() => row.operator = value ?? '==');
                  _emit();
                },
              ),
            ),
          const SizedBox(width: 8),
          IconButton(
            icon: const Icon(Icons.remove_circle_outline, size: 18, color: AppColors.red600),
            onPressed: () {
              setState(() => _rows.removeAt(index));
              _emit();
            },
          ),
        ]),
        const SizedBox(height: 8),
        if (row.kind == 'guest')
          Text('Guest customers only (customer_id = 0).',
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500))
        else if (row.kind == 'select')
          DropdownButtonFormField<int>(
            isDense: true,
            initialValue: row.optionValue,
            decoration: const InputDecoration(
                isDense: true, labelText: 'Value', border: OutlineInputBorder()),
            items: (_optionsCache[row.attribute] ?? const [])
                .map((option) => DropdownMenuItem(
                    value: option['id'] as int?,
                    child: Text('${option['admin_name']}',
                        style: const TextStyle(fontFamily: 'Poppins', fontSize: 12))))
                .toList(),
            onChanged: (value) {
              setState(() => row.optionValue = value);
              _emit();
            },
          )
        else if (row.kind != 'unknown')
          TextField(
            controller: row.valueController,
            keyboardType:
                row.kind == 'number' ? TextInputType.number : TextInputType.text,
            decoration: InputDecoration(
                isDense: true,
                labelText: row.kind == 'number' ? 'Value' : 'Value (contains supported)',
                border: const OutlineInputBorder()),
            onChanged: (_) => _emit(),
          )
        else
          Text('Unrecognised condition: kept as-is when saving.',
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
      ]),
    );
  }
}

class _EditableRow {
  _EditableRow();

  String attribute = '';
  String operator = '==';
  final TextEditingController valueController = TextEditingController();
  int? optionValue;
  ConditionRow? unknownCondition;

  String get kind {
    final builtin = _builtinAttributes.where((entry) => entry.$1 == attribute).firstOrNull;
    if (builtin != null) return builtin.$3;
    return '';
  }

  factory _EditableRow.fromCondition(Map<String, dynamic> condition) {
    final row = _EditableRow();
    final payload = ConditionRow.fromCondition(condition);
    row.attribute = payload.attribute;
    row.operator = payload.operator;
    if (row.kind == 'number') {
      row.valueController.text = '${payload.value ?? ''}';
    } else if (row.kind == 'select') {
      row.optionValue = payload.value is int ? payload.value as int : null;
    } else if (row.kind == 'text') {
      row.valueController.text = '${payload.value ?? ''}';
    } else if (row.kind != 'guest') {
      row.unknownCondition = payload;
      row.attribute = '';
    }
    return row;
  }
}
