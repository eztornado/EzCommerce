import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import 'admin_shell.dart' show EzDataGrid;

/// CMS: datagrid de páginas + editor (título, url_key, html, meta).
class AdminCmsPage extends StatelessWidget {
  const AdminCmsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        EzDataGrid(
          gridPath: 'admin/cms/pages',
          title: 'CMS Pages',
          extraActions: (row) => [
            IconButton(
              icon: const Icon(Icons.edit_outlined, size: 18),
              tooltip: 'Edit',
              onPressed: () async {
                final admin = context.read<AdminProvider>();
                final data = await admin.get('admin/cms/pages/view/${row['id']}');
                if (data is Map && context.mounted) {
                  showDialog(
                    context: context,
                    builder: (context) => CmsPageEditorDialog(
                      pageId: row['id'] as int?,
                      translation:
                          Map<String, dynamic>.from((data['translation'] ?? const {}) as Map),
                    ),
                  );
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
            label: const Text('New Page'),
            onPressed: () => showDialog(
              context: context,
              builder: (context) => CmsPageEditorDialog(),
            ),
          ),
        ),
      ],
    );
  }
}

class CmsPageEditorDialog extends StatefulWidget {
  const CmsPageEditorDialog({super.key, this.pageId, this.translation = const {}});
  final int? pageId;
  final Map<String, dynamic> translation;

  @override
  State<CmsPageEditorDialog> createState() => _CmsPageEditorDialogState();
}

class _CmsPageEditorDialogState extends State<CmsPageEditorDialog> {
  late final TextEditingController _title;
  late final TextEditingController _urlKey;
  late final TextEditingController _html;
  late final TextEditingController _metaTitle;
  late final TextEditingController _metaDescription;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    _title = TextEditingController(text: '${widget.translation['page_title'] ?? ''}');
    _urlKey = TextEditingController(text: '${widget.translation['url_key'] ?? ''}');
    _html = TextEditingController(text: '${widget.translation['html_content'] ?? ''}');
    _metaTitle = TextEditingController(text: '${widget.translation['meta_title'] ?? ''}');
    _metaDescription = TextEditingController(text: '${widget.translation['meta_description'] ?? ''}');
  }

  Future<void> _save() async {
    setState(() => _saving = true);
    final admin = context.read<AdminProvider>();
    final body = {
      'page_title': _title.text,
      'url_key': _urlKey.text,
      'html_content': _html.text,
      'meta_title': _metaTitle.text,
      'meta_description': _metaDescription.text,
    };
    try {
      if (widget.pageId == null) {
        await admin.post('admin/cms/pages/store', body: body);
      } else {
        await admin.put('admin/cms/pages/edit/${widget.pageId}', body: body);
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
      title: Text(widget.pageId == null ? 'New CMS Page' : 'Edit CMS Page'),
      content: SingleChildScrollView(
        child: SizedBox(
          width: 520,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: _title,
                decoration: const InputDecoration(labelText: 'Page Title'),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: _urlKey,
                decoration: const InputDecoration(
                    labelText: 'URL Key', helperText: 'Se sirve en /api/cms/page/{url_key}'),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: _html,
                maxLines: 8,
                decoration: const InputDecoration(
                    labelText: 'HTML Content',
                    alignLabelWithHint: true,
                    border: OutlineInputBorder()),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: _metaTitle,
                decoration: const InputDecoration(labelText: 'Meta Title'),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: _metaDescription,
                maxLines: 2,
                decoration: const InputDecoration(labelText: 'Meta Description'),
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
