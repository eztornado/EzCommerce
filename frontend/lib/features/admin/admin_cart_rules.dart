import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import 'admin_shell.dart' show EzDataGrid;
import 'conditions_editor.dart';

/// Reglas de carrito (marketing/promotions): datagrid + editor con acción,
/// cupón y condiciones visuales (subtotal, cantidades, atributos...).
class AdminCartRulesPage extends StatelessWidget {
  const AdminCartRulesPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        EzDataGrid(
          gridPath: 'admin/marketing/cart-rules',
          title: 'Cart Rules',
          extraActions: (row) => [
            IconButton(
              icon: const Icon(Icons.edit_outlined, size: 18),
              tooltip: 'Edit',
              onPressed: () async {
                final admin = context.read<AdminProvider>();
                final data = await admin.get('admin/marketing/cart-rules/view/${row['id']}');
                if (data is! Map || !context.mounted) return;
                final payload = Map<String, dynamic>.from((data['data'] ?? const {}) as Map);
                final rule = Map<String, dynamic>.from((payload['rule'] ?? const {}) as Map);
                if ((rule['coupon_type'] ?? 1) == 2) {
                  final primary = decodeList(payload['coupons'])
                      .where((coupon) => (coupon['is_primary'] ?? 0) == 1)
                      .firstOrNull;
                  if (primary != null) rule['coupon_code'] = primary['code'];
                }
                if (context.mounted) {
                  showDialog(
                    context: context,
                    builder: (context) => CartRuleEditorDialog(rule: rule),
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
                  await admin.delete('admin/marketing/cart-rules/${row['id']}');
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
            onPressed: () => _openEditor(context),
          ),
        ),
      ],
    );
  }

  void _openEditor(BuildContext context, [Map<String, dynamic>? rule]) {
    showDialog(
      context: context,
      builder: (context) => CartRuleEditorDialog(rule: rule),
    );
  }
}

class CartRuleEditorDialog extends StatefulWidget {
  const CartRuleEditorDialog({super.key, this.rule});
  final Map<String, dynamic>? rule;

  @override
  State<CartRuleEditorDialog> createState() => _CartRuleEditorDialogState();
}

class _CartRuleEditorDialogState extends State<CartRuleEditorDialog> {
  late final TextEditingController _name;
  late final TextEditingController _description;
  late final TextEditingController _couponCode;
  late final TextEditingController _discountAmount;
  late final TextEditingController _usagePerCustomer;
  late final TextEditingController _usesPerCoupon;
  String _actionType = 'by_percent';
  int _couponType = 1;
  bool _applyToShipping = false;
  bool _freeShipping = false;
  int _conditionType = 1;
  List<Map<String, dynamic>> _conditions = [];
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    final rule = widget.rule ?? const {};
    _name = TextEditingController(text: '${rule['name'] ?? ''}');
    _description = TextEditingController(text: '${rule['description'] ?? ''}');
    _couponCode = TextEditingController(text: '${rule['coupon_code'] ?? ''}');
    _discountAmount = TextEditingController(text: '${rule['discount_amount'] ?? 10}');
    _usagePerCustomer = TextEditingController(text: '${rule['usage_per_customer'] ?? 0}');
    _usesPerCoupon = TextEditingController(text: '${rule['uses_per_coupon'] ?? 0}');
    _actionType = '${rule['action_type'] ?? 'by_percent'}';
    _couponType = (rule['coupon_type'] ?? 1) as int;
    _applyToShipping = (rule['apply_to_shipping'] ?? 0) == 1;
    _freeShipping = (rule['free_shipping'] ?? 0) == 1;
    _conditionType = (rule['condition_type'] ?? 1) as int;
    _conditions = decodeList(rule['conditions']);
  }

  Future<void> _save() async {
    setState(() => _saving = true);
    final admin = context.read<AdminProvider>();
    final body = {
      'name': _name.text,
      'description': _description.text,
      'status': 1,
      'coupon_type': _couponType,
      'use_auto_generation': 0,
      'coupon_code': _couponType == 2 ? _couponCode.text : null,
      'usage_per_customer': int.tryParse(_usagePerCustomer.text) ?? 0,
      'uses_per_coupon': int.tryParse(_usesPerCoupon.text) ?? 0,
      'condition_type': _conditionType,
      'conditions': _conditions,
      'action_type': _actionType,
      'discount_amount': num.tryParse(_discountAmount.text) ?? 0,
      'apply_to_shipping': _applyToShipping ? 1 : 0,
      'free_shipping': _freeShipping ? 1 : 0,
      'sort': 0,
    };
    try {
      if (widget.rule == null) {
        await admin.post('admin/marketing/cart-rules/store', body: body);
      } else {
        await admin.put('admin/marketing/cart-rules/edit/${widget.rule!['id']}', body: body);
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
      title: Text(widget.rule == null ? 'New Cart Rule' : 'Edit Cart Rule'),
      content: SingleChildScrollView(
        child: SizedBox(
          width: 480,
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
                      DropdownMenuItem(value: 'by_percent', child: Text('Percentage discount')),
                      DropdownMenuItem(value: 'by_fixed', child: Text('Fixed amount per item')),
                      DropdownMenuItem(value: 'cart_fixed', child: Text('Fixed amount for the cart')),
                      DropdownMenuItem(value: 'buy_x_get_y', child: Text('Buy X get Y free')),
                    ],
                    onChanged: (value) => setState(() => _actionType = value ?? 'by_percent'),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: TextField(
                    controller: _discountAmount,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(labelText: 'Discount amount'),
                  ),
                ),
              ]),
              const SizedBox(height: 12),
              Row(children: [
                Switch(
                  value: _applyToShipping,
                  activeColor: AppColors.navyBlue, // ignore: deprecated_member_use
                  onChanged: (value) => setState(() => _applyToShipping = value),
                ),
                const Text('Apply to shipping',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                const SizedBox(width: 16),
                Switch(
                  value: _freeShipping,
                  activeColor: AppColors.navyBlue, // ignore: deprecated_member_use
                  onChanged: (value) => setState(() => _freeShipping = value),
                ),
                const Text('Free shipping',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
              ]),
              const Divider(height: 24),
              Row(children: [
                Switch(
                  value: _couponType == 2,
                  activeColor: AppColors.navyBlue, // ignore: deprecated_member_use
                  onChanged: (value) => setState(() => _couponType = value ? 2 : 1),
                ),
                const Text('Requires coupon',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
              ]),
              if (_couponType == 2) ...[
                TextField(
                  controller: _couponCode,
                  decoration: const InputDecoration(
                      labelText: 'Coupon code (e.g. WELCOME10)'),
                ),
                const SizedBox(height: 12),
                Row(children: [
                  Expanded(
                    child: TextField(
                      controller: _usagePerCustomer,
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(
                          labelText: 'Uses per customer (0 = unlimited)'),
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: TextField(
                      controller: _usesPerCoupon,
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(
                          labelText: 'Total usage limit (0 = unlimited)'),
                    ),
                  ),
                ]),
              ],
              const Divider(height: 24),
              Text('Conditions',
                  style: const TextStyle(
                      fontFamily: 'Poppins', fontSize: 15, fontWeight: FontWeight.w500)),
              const SizedBox(height: 12),
              CartConditionsEditor(
                conditionType: _conditionType,
                conditions: _conditions,
                onChanged: (conditionType, conditions) {
                  setState(() {
                    _conditionType = conditionType;
                    _conditions = conditions;
                  });
                },
              ),
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
