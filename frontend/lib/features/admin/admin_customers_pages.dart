import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';

/// Grupos de clientes: listado con alta/edición/baja (los de sistema
/// protegidos).
class AdminGroupsPage extends StatefulWidget {
  const AdminGroupsPage({super.key});

  @override
  State<AdminGroupsPage> createState() => _AdminGroupsPageState();
}

class _AdminGroupsPageState extends State<AdminGroupsPage> {
  List<Map<String, dynamic>> _groups = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final data = await admin.get('admin/customers/groups');
    if (mounted) {
      setState(() {
        _groups = decodeList(data);
        _loading = false;
      });
    }
  }

  void _showError(Object error) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
  }

  Future<void> _groupDialog([Map<String, dynamic>? group]) async {
    final code = TextEditingController(text: '${group?['code'] ?? ''}');
    final name = TextEditingController(text: '${group?['name'] ?? ''}');
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(group == null ? 'New customer group' : 'Edit customer group'),
        content: SizedBox(
          width: 420,
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            TextField(
              controller: code,
              enabled: group == null,
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
      group == null
          ? await admin.post('admin/customers/groups/store', body: {
              'code': code.text.trim(),
              'name': name.text.trim(),
            })
          : await admin.put('admin/customers/groups/edit/${group['id']}', body: {
              'name': name.text.trim(),
            });
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _delete(Map<String, dynamic> group) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete group?'),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
          TextButton(onPressed: () => Navigator.pop(context, true), child: const Text('Delete')),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      await admin.delete('admin/customers/groups/${group['id']}');
      await _load();
    } catch (error) {
      _showError(error);
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
            child: Text('Customer Groups',
                style: TextStyle(
                    fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
          ),
          TextButton.icon(
            onPressed: () => _groupDialog(),
            icon: const Icon(Icons.add, size: 18),
            label: const Text('New Group', style: TextStyle(fontFamily: 'Poppins')),
          ),
        ]),
        const SizedBox(height: 24),
        for (final group in _groups)
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
                  Text('${group['name']}',
                      style: const TextStyle(
                          fontFamily: 'Poppins', fontSize: 14, fontWeight: FontWeight.w500)),
                  Text('code: ${group['code']}',
                      style: TextStyle(
                          fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                ]),
              ),
              IconButton(
                icon: const Icon(Icons.edit_outlined, size: 18),
                onPressed: (group['is_user_defined'] ?? 1) == 1
                    ? () => _groupDialog(group)
                    : null,
              ),
              IconButton(
                icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                onPressed: (group['is_user_defined'] ?? 1) == 1
                    ? () => _delete(group)
                    : null,
              ),
            ]),
          ),
      ]),
    );
  }
}

/// Moderación de reseñas: aprobar / rechazar / pendiente.
class AdminReviewsPage extends StatefulWidget {
  const AdminReviewsPage({super.key});

  @override
  State<AdminReviewsPage> createState() => _AdminReviewsPageState();
}

class _AdminReviewsPageState extends State<AdminReviewsPage> {
  List<Map<String, dynamic>> _reviews = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final data = await admin.post('admin/customers/reviews', body: {});
    if (mounted) {
      setState(() {
        final payload = data is Map ? (data['data'] ?? data) : data;
        _reviews = decodeList(payload is Map ? payload['records'] ?? payload : payload);
        _loading = false;
      });
    }
  }

  Future<void> _setStatus(Map<String, dynamic> review, String status) async {
    final admin = context.read<AdminProvider>();
    try {
      await admin.put('admin/customers/reviews/${review['id']}/status', body: {
        'status': status,
      });
      await _load();
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
      }
    }
  }

  Color _statusColor(String status) => switch (status) {
        'approved' => Colors.green.shade700,
        'rejected' => AppColors.red600,
        _ => AppColors.zinc600,
      };

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        const Text('Reviews',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
        const SizedBox(height: 8),
        const Text('Only approved reviews are visible in the store.',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
        const SizedBox(height: 24),
        for (final review in _reviews)
          Container(
            margin: const EdgeInsets.only(bottom: 12),
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(EzTheme.radiusXl),
              border: Border.all(color: AppColors.zinc200),
            ),
            child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
              Expanded(
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  Row(children: [
                    Text('${review['title']}',
                        style: const TextStyle(
                            fontFamily: 'Poppins',
                            fontSize: 14,
                            fontWeight: FontWeight.w500)),
                    const SizedBox(width: 8),
                    Text('★' * ((review['rating'] ?? 0) as int),
                        style: const TextStyle(color: Colors.orange, fontSize: 12)),
                  ]),
                  const SizedBox(height: 4),
                  Text('${review['comment']}',
                      style: const TextStyle(fontFamily: 'Poppins', fontSize: 12)),
                  const SizedBox(height: 4),
                  Text('by ${review['name']} · ${review['created_at']}',
                      style: TextStyle(
                          fontFamily: 'Poppins', fontSize: 11, color: AppColors.zinc500)),
                ]),
              ),
              const SizedBox(width: 12),
              Column(crossAxisAlignment: CrossAxisAlignment.end, children: [
                Text('${review['status']}',
                    style: TextStyle(
                        fontFamily: 'Poppins',
                        fontSize: 12,
                        color: _statusColor('${review['status']}'))),
                const SizedBox(height: 8),
                Row(mainAxisSize: MainAxisSize.min, children: [
                  TextButton(
                    onPressed:
                        '${review['status']}' == 'approved' ? null : () => _setStatus(review, 'approved'),
                    child: const Text('Approve', style: TextStyle(fontFamily: 'Poppins', fontSize: 12)),
                  ),
                  TextButton(
                    onPressed:
                        '${review['status']}' == 'rejected' ? null : () => _setStatus(review, 'rejected'),
                    child: const Text('Reject',
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.red600)),
                  ),
                ]),
              ]),
            ]),
          ),
      ]),
    );
  }
}
