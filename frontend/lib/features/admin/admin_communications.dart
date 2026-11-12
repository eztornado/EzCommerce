import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';

/// Marketing communications: plantillas de email, eventos y campañas con
/// envío a los clientes suscritos (mailer de log).
class AdminCommunicationsPage extends StatefulWidget {
  const AdminCommunicationsPage({super.key});

  @override
  State<AdminCommunicationsPage> createState() => _AdminCommunicationsPageState();
}

class _AdminCommunicationsPageState extends State<AdminCommunicationsPage> {
  List<Map<String, dynamic>> _templates = [];
  List<Map<String, dynamic>> _campaigns = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final templates = await admin.get('admin/marketing/communications/templates');
    final campaigns = await admin.get('admin/marketing/communications/campaigns');
    if (mounted) {
      setState(() {
        _templates = decodeList(templates);
        _campaigns = decodeList(campaigns);
        _loading = false;
      });
    }
  }

  void _showError(Object error) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
  }

  void _toast(String message) {
    if (!mounted) return;
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text(message)));
  }

  Future<void> _templateDialog([Map<String, dynamic>? template]) async {
    final name = TextEditingController(text: '${template?['name'] ?? ''}');
    final content = TextEditingController(text: '${template?['content'] ?? ''}');
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(template == null ? 'New template' : 'Edit template'),
        content: SingleChildScrollView(
          child: SizedBox(
            width: 520,
            child: Column(mainAxisSize: MainAxisSize.min, children: [
              TextField(controller: name, decoration: const InputDecoration(labelText: 'Name')),
              const SizedBox(height: 8),
              const Align(
                alignment: Alignment.centerLeft,
                child: Text('Placeholders: {{first_name}}, {{last_name}}, {{email}}',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 11, color: AppColors.zinc500)),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: content,
                maxLines: 10,
                decoration: const InputDecoration(
                    labelText: 'Content (HTML)',
                    alignLabelWithHint: true,
                    border: OutlineInputBorder()),
              ),
            ]),
          ),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
          PrimaryButton(
            label: 'Save',
            onPressed: () => name.text.trim().isEmpty ? null : Navigator.pop(context, true),
          ),
        ],
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    final body = {'name': name.text.trim(), 'status': 1, 'content': content.text};
    try {
      template == null
          ? await admin.post('admin/marketing/communications/templates/store', body: body)
          : await admin.put('admin/marketing/communications/templates/edit/${template['id']}',
              body: body);
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _campaignDialog([Map<String, dynamic>? campaign]) async {
    final name = TextEditingController(text: '${campaign?['name'] ?? ''}');
    final subject = TextEditingController(text: '${campaign?['subject'] ?? ''}');
    int? templateId = campaign?['marketing_template_id'] as int?;
    int? groupId = campaign?['customer_group_id'] as int?;
    final groups = [
      {'id': 1, 'name': 'Guest'},
      {'id': 2, 'name': 'General'},
      {'id': 3, 'name': 'Wholesale'},
    ];
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(campaign == null ? 'New campaign' : 'Edit campaign'),
          content: SizedBox(
            width: 440,
            child: Column(mainAxisSize: MainAxisSize.min, children: [
              TextField(controller: name, decoration: const InputDecoration(labelText: 'Name')),
              const SizedBox(height: 12),
              TextField(controller: subject, decoration: const InputDecoration(labelText: 'Email subject')),
              const SizedBox(height: 12),
              DropdownButtonFormField<int>(
                initialValue: templateId,
                decoration: const InputDecoration(labelText: 'Template'),
                items: _templates
                    .map((template) => DropdownMenuItem(
                          value: template['id'] as int,
                          child: Text('${template['name']}'),
                        ))
                    .toList(),
                onChanged: (value) => setDialogState(() => templateId = value),
              ),
              const SizedBox(height: 12),
              DropdownButtonFormField<int>(
                initialValue: groupId,
                decoration:
                    const InputDecoration(labelText: 'Customer group (optional)'),
                items: groups
                    .map((group) => DropdownMenuItem(
                          value: group['id'] as int,
                          child: Text('${group['name']}'),
                        ))
                    .toList(),
                onChanged: (value) => setDialogState(() => groupId = value),
              ),
            ]),
          ),
          actions: [
            TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
            PrimaryButton(
              label: 'Save',
              onPressed: () =>
                  (name.text.trim().isEmpty || templateId == null) ? null : Navigator.pop(context, true),
            ),
          ],
        ),
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    final body = {
      'name': name.text.trim(),
      'subject': subject.text.trim(),
      'marketing_template_id': templateId,
      'customer_group_id': groupId,
    };
    try {
      campaign == null
          ? await admin.post('admin/marketing/communications/campaigns/store', body: body)
          : await admin.put('admin/marketing/communications/campaigns/edit/${campaign['id']}',
              body: body);
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _send(Map<String, dynamic> campaign) async {
    final admin = context.read<AdminProvider>();
    try {
      final result = await admin.post('admin/marketing/communications/campaigns/send/${campaign['id']}');
      final message = result is Map
          ? '${result['message'] ?? 'Campaign sent.'}'
          : 'Campaign sent.';
      _toast(message);
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
        const Text('Communications',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
        const SizedBox(height: 8),
        const Text(
            'Email templates and campaigns. Sending renders the template per subscribed customer '
            'and writes it to the service log (mail = log in this deployment).',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
        const SizedBox(height: 24),
        _card(
          title: 'Templates',
          actionLabel: 'New Template',
          onAction: () => _templateDialog(),
          children: [
            for (final template in _templates)
              ListTile(
                title: Text('${template['name']}',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                subtitle: Text('${(template['content'] ?? '').toString().length} chars',
                    style: const TextStyle(
                        fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                trailing: Row(mainAxisSize: MainAxisSize.min, children: [
                  IconButton(
                    icon: const Icon(Icons.edit_outlined, size: 18),
                    onPressed: () => _templateDialog(template),
                  ),
                  IconButton(
                    icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                    onPressed: () async {
                      final admin = context.read<AdminProvider>();
                      await admin
                          .delete('admin/marketing/communications/templates/${template['id']}');
                      await _load();
                    },
                  ),
                ]),
              ),
          ],
        ),
        _card(
          title: 'Campaigns',
          actionLabel: 'New Campaign',
          onAction: () => _campaignDialog(),
          children: [
            for (final campaign in _campaigns)
              ListTile(
                title: Text('${campaign['name']}',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                subtitle: Text(
                    '${campaign['subject']} · template: ${campaign['template_name'] ?? '-'} · '
                    'group: ${campaign['group_name'] ?? 'all'}',
                    style: const TextStyle(
                        fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                trailing: Row(mainAxisSize: MainAxisSize.min, children: [
                  if ((campaign['spooling'] ?? 0) == 1)
                    const Chip(
                        label: Text('Sent',
                            style: TextStyle(fontFamily: 'Poppins', fontSize: 11)))
                  else
                    TextButton(
                      onPressed: () => _send(campaign),
                      child: const Text('Send', style: TextStyle(fontFamily: 'Poppins')),
                    ),
                  IconButton(
                    icon: const Icon(Icons.edit_outlined, size: 18),
                    onPressed: () => _campaignDialog(campaign),
                  ),
                  IconButton(
                    icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                    onPressed: () async {
                      final admin = context.read<AdminProvider>();
                      await admin
                          .delete('admin/marketing/communications/campaigns/${campaign['id']}');
                      await _load();
                    },
                  ),
                ]),
              ),
          ],
        ),
      ]),
    );
  }

  Widget _card({
    required String title,
    required String actionLabel,
    required VoidCallback onAction,
    required List<Widget> children,
  }) {
    return Container(
      margin: const EdgeInsets.only(bottom: 16),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [
          Expanded(
            child: Text(title,
                style: const TextStyle(
                    fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500)),
          ),
          TextButton.icon(
            onPressed: onAction,
            icon: const Icon(Icons.add, size: 18),
            label: Text(actionLabel, style: const TextStyle(fontFamily: 'Poppins')),
          ),
        ]),
        ...children,
      ]),
    );
  }
}
