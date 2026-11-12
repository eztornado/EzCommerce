import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../../../core/utils/pick_image.dart';

const _sectionTypes = <(String, String)>[
  ('image_carousel', 'Image carousel'),
  ('product_carousel', 'Product carousel'),
  ('category_carousel', 'Category carousel'),
  ('static_content', 'Static content'),
  ('services_content', 'Services strip'),
  ('footer_links', 'Footer links'),
];

/// Apariencia: secciones del home (carruseles, banners, servicios) con
/// estado publicado/borrador, orden y editor por tipo.
class AdminAppearancePage extends StatefulWidget {
  const AdminAppearancePage({super.key});

  @override
  State<AdminAppearancePage> createState() => _AdminAppearancePageState();
}

class _AdminAppearancePageState extends State<AdminAppearancePage> {
  List<Map<String, dynamic>> _sections = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final data = await admin.get('admin/appearance/sections/list/en');
    if (mounted) {
      setState(() {
        _sections = decodeList(data);
        _loading = false;
      });
    }
  }

  void _showError(Object error) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
  }

  Future<void> _toggleStatus(Map<String, dynamic> section) async {
    final admin = context.read<AdminProvider>();
    try {
      await admin.post('admin/appearance/sections/status/${section['id']}', body: {
        'status': (section['status'] ?? 1) == 1 ? 0 : 1,
      });
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _move(int index, int delta) async {
    final target = index + delta;
    if (target < 0 || target >= _sections.length) return;
    final rows = [..._sections];
    final section = rows.removeAt(index);
    rows.insert(target, section);
    setState(() => _sections = rows);
    final admin = context.read<AdminProvider>();
    try {
      await admin.post('admin/appearance/sections/reorder', body: {
        'order': [for (final row in rows) row['id']],
      });
    } catch (error) {
      _showError(error);
      await _load();
    }
  }

  Future<void> _delete(Map<String, dynamic> section) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete section?'),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
          TextButton(onPressed: () => Navigator.pop(context, true), child: const Text('Delete')),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      await admin.delete('admin/appearance/sections/${section['id']}');
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  String _typeLabel(String type) =>
      _sectionTypes.where((entry) => entry.$1 == type).firstOrNull?.$2 ?? type;

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    return Stack(
      children: [
        SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 860),
            child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              const Text('Appearance',
                  style:
                      TextStyle(fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
              const SizedBox(height: 8),
              const Text('Home sections. Draft sections are not shown in the store.',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc500)),
              const SizedBox(height: 24),
              for (final (index, section) in _sections.indexed)
                Container(
                  margin: const EdgeInsets.only(bottom: 12),
                  padding: const EdgeInsets.all(16),
                  decoration: BoxDecoration(
                    color: Colors.white,
                    borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                    border: Border.all(color: AppColors.zinc200),
                  ),
                  child: Row(children: [
                    Column(children: [
                      IconButton(
                        icon: const Icon(Icons.arrow_upward, size: 18),
                        onPressed: index > 0 ? () => _move(index, -1) : null,
                      ),
                      IconButton(
                        icon: const Icon(Icons.arrow_downward, size: 18),
                        onPressed: index < _sections.length - 1 ? () => _move(index, 1) : null,
                      ),
                    ]),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                        Text('${section['name'] ?? _typeLabel('${section['type']}')}',
                            style: const TextStyle(
                                fontFamily: 'Poppins', fontSize: 14, fontWeight: FontWeight.w500)),
                        const SizedBox(height: 2),
                        Text(_typeLabel('${section['type']}'),
                            style: const TextStyle(
                                fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                      ]),
                    ),
                    Chip(
                      label: Text(
                        (section['status'] ?? 1) == 1 ? 'Published' : 'Draft',
                        style: TextStyle(
                            fontFamily: 'Poppins',
                            fontSize: 11,
                            color: (section['status'] ?? 1) == 1 ? Colors.white : AppColors.zinc600),
                      ),
                      backgroundColor:
                          (section['status'] ?? 1) == 1 ? AppColors.navyBlue : AppColors.zinc200,
                    ),
                    Switch(
                      value: (section['status'] ?? 1) == 1,
                      activeColor: AppColors.navyBlue, // ignore: deprecated_member_use
                      onChanged: (_) => _toggleStatus(section),
                    ),
                    IconButton(
                      icon: const Icon(Icons.edit_outlined, size: 18),
                      tooltip: 'Edit',
                      onPressed: () => showDialog(
                        context: context,
                        builder: (context) => SectionEditorDialog(section: section),
                      ).then((_) => _load()),
                    ),
                    IconButton(
                      icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                      tooltip: 'Delete',
                      onPressed: () => _delete(section),
                    ),
                  ]),
                ),
            ]),
          ),
        ),
        Positioned(
          right: 24,
          bottom: 24,
          child: FloatingActionButton.extended(
            backgroundColor: AppColors.navyBlue,
            foregroundColor: Colors.white,
            icon: const Icon(Icons.add),
            label: const Text('New Section'),
            onPressed: () => showDialog(
              context: context,
              builder: (context) => const SectionEditorDialog(),
            ).then((_) => _load()),
          ),
        ),
      ],
    );
  }
}

/// Editor de sección: nombre, tipo y opciones estructuradas según el tipo
/// (fallback JSON para servicios/enlaces del pie).
class SectionEditorDialog extends StatefulWidget {
  const SectionEditorDialog({super.key, this.section});
  final Map<String, dynamic>? section;

  @override
  State<SectionEditorDialog> createState() => _SectionEditorDialogState();
}

class _SectionEditorDialogState extends State<SectionEditorDialog> {
  late final TextEditingController _name;
  late String _type;
  late final Map<String, dynamic> _options;
  final List<TextEditingController> _slideTitles = [];
  final List<TextEditingController> _slideLinks = [];
  final List<String> _slideImages = [];
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    final section = widget.section ?? const {};
    _name = TextEditingController(text: '${section['name'] ?? ''}');
    _type = '${section['type'] ?? 'image_carousel'}';
    _options = Map<String, dynamic>.from((section['options'] ?? const {}) as Map);
    for (final slide in decodeList(_options['slides'])) {
      _slideTitles.add(TextEditingController(text: '${slide['title'] ?? ''}'));
      _slideLinks.add(TextEditingController(text: '${slide['link'] ?? ''}'));
      _slideImages.add('${slide['image'] ?? ''}');
    }
    if (_type == 'image_carousel' && _slideTitles.isEmpty) {
      _addSlide();
    }
  }

  void _addSlide() {
    setState(() {
      _slideTitles.add(TextEditingController());
      _slideLinks.add(TextEditingController());
      _slideImages.add('');
    });
  }

  Future<void> _uploadSlideImage(int index) async {
    final picked = await pickImage();
    if (picked == null || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      final uploaded = await admin.post('admin/upload', body: {
        'filename': picked.filename,
        'folder': 'theme',
        'data': base64Encode(picked.bytes),
      });
      if (uploaded is Map && mounted) {
        setState(() => _slideImages[index] = '${uploaded['path']}');
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
      }
    }
  }

  Map<String, dynamic> _buildOptions() {
    switch (_type) {
      case 'image_carousel':
        return {
          'slides': [
            for (final (index, _) in _slideTitles.indexed)
              {
                'title': _slideTitles[index].text,
                'link': _slideLinks[index].text,
                'image': _slideImages[index],
              }
          ],
        };
      case 'product_carousel':
      case 'category_carousel':
        return {
          'title': _options['title'],
          'filters': _options['filters'],
          'limit': _options['limit'],
        };
      default:
        return _options;
    }
  }

  Future<void> _save() async {
    setState(() => _saving = true);
    final admin = context.read<AdminProvider>();
    final body = {
      'type': _type,
      'name': _name.text.trim(),
      'status': (widget.section?['status'] ?? 1) as int,
      'sort': widget.section?['sort_order'] ?? 0,
      'locale': 'en',
      'options': _buildOptions(),
    };
    try {
      widget.section == null
          ? await admin.post('admin/appearance/sections/store', body: body)
          : await admin.put('admin/appearance/sections/edit/${widget.section!['id']}', body: body);
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
      title: Text(widget.section == null ? 'New section' : 'Edit section'),
      content: SingleChildScrollView(
        child: SizedBox(
          width: 520,
          child: Column(mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start, children: [
            TextField(controller: _name, decoration: const InputDecoration(labelText: 'Name')),
            const SizedBox(height: 12),
            DropdownButtonFormField<String>(
              initialValue: _type,
              decoration: const InputDecoration(labelText: 'Type'),
              items: _sectionTypes
                  .map((type) => DropdownMenuItem(value: type.$1, child: Text(type.$2)))
                  .toList(),
              onChanged: (value) => setState(() => _type = value ?? 'image_carousel'),
            ),
            const SizedBox(height: 16),
            if (_type == 'image_carousel') ...[
              Row(children: [
                const Expanded(
                  child: Text('Slides',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                ),
                TextButton.icon(
                  onPressed: _addSlide,
                  icon: const Icon(Icons.add, size: 16),
                  label: const Text('Add slide', style: TextStyle(fontFamily: 'Poppins')),
                ),
              ]),
              for (final (index, _) in _slideTitles.indexed)
                Container(
                  margin: const EdgeInsets.only(bottom: 10),
                  padding: const EdgeInsets.all(10),
                  decoration: BoxDecoration(
                    border: Border.all(color: AppColors.zinc200),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Column(children: [
                    Row(children: [
                      Expanded(
                        child: TextField(
                          controller: _slideTitles[index],
                          decoration: const InputDecoration(
                              isDense: true, labelText: 'Title', border: OutlineInputBorder()),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        child: TextField(
                          controller: _slideLinks[index],
                          decoration: const InputDecoration(
                              isDense: true, labelText: 'Link', border: OutlineInputBorder()),
                        ),
                      ),
                    ]),
                    const SizedBox(height: 8),
                    Row(children: [
                      Expanded(
                        child: Text(_slideImages[index].isEmpty
                            ? 'No image'
                            : _slideImages[index],
                            style: const TextStyle(
                                fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500),
                            overflow: TextOverflow.ellipsis),
                      ),
                      TextButton(
                        onPressed: () => _uploadSlideImage(index),
                        child: const Text('Upload', style: TextStyle(fontFamily: 'Poppins')),
                      ),
                      IconButton(
                        icon: const Icon(Icons.remove_circle_outline, size: 18),
                        onPressed: () => setState(() {
                          _slideTitles.removeAt(index);
                          _slideLinks.removeAt(index);
                          _slideImages.removeAt(index);
                        }),
                      ),
                    ]),
                  ]),
                ),
            ] else
              _genericOptionsFields(),
          ]),
        ),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
        PrimaryButton(label: 'Save', isLoading: _saving, onPressed: _saving ? null : _save),
      ],
    );
  }

  /// Campos según tipo para los demás tipos; JSON crudo como fallback.
  Widget _genericOptionsFields() {
    final fields = <Widget>[];
    if (_type == 'product_carousel' || _type == 'category_carousel') {
      final title = TextEditingController(text: '${_options['title'] ?? ''}');
      final limit = TextEditingController(text: '${_options['limit'] ?? 8}');
      final parent = TextEditingController(
          text: '${((_options['filters'] ?? const {}) as Map)['parent_id'] ?? 1}');
      final sort = TextEditingController(
          text: '${((_options['filters'] ?? const {}) as Map)['sort'] ?? ''}');
      fields.addAll([
        TextField(
          controller: title,
          decoration: const InputDecoration(labelText: 'Title'),
          onChanged: (value) => _options['title'] = value,
        ),
        const SizedBox(height: 12),
        Row(children: [
          Expanded(
            child: TextField(
              controller: limit,
              keyboardType: TextInputType.number,
              decoration: const InputDecoration(labelText: 'Limit'),
              onChanged: (value) => _options['limit'] = int.tryParse(value) ?? 8,
            ),
          ),
          const SizedBox(width: 12),
          if (_type == 'category_carousel')
            Expanded(
              child: TextField(
                controller: parent,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(labelText: 'Parent category id'),
                onChanged: (value) => _ensureFilters()['parent_id'] = int.tryParse(value) ?? 1,
              ),
            )
          else
            Expanded(
              child: TextField(
                controller: sort,
                decoration:
                    const InputDecoration(labelText: 'Sort (newest, featured...)'),
                onChanged: (value) => _ensureFilters()['sort'] = value,
              ),
            ),
        ]),
        const SizedBox(height: 12),
      ]);
    } else {
      final json = TextEditingController(
          text: const JsonEncoder.withIndent('  ').convert(_options));
      fields.add(TextField(
        controller: json,
        maxLines: 8,
        decoration: const InputDecoration(
            labelText: 'Options (JSON)',
            alignLabelWithHint: true,
            border: OutlineInputBorder()),
        onChanged: (value) {
          try {
            final parsed = jsonDecode(value);
            if (parsed is Map<String, dynamic>) _options..clear()..addAll(parsed);
          } catch (_) {}
        },
      ));
    }
    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: fields);
  }

  Map<String, dynamic> _ensureFilters() {
    final filters = _options['filters'];
    if (filters is Map<String, dynamic>) return filters;
    final created = <String, dynamic>{...?filters is Map ? filters.cast<String, dynamic>() : null};
    _options['filters'] = created;
    return created;
  }
}
