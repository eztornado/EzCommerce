import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';

/// Gestión de categorías: árbol del canal con crear/editar/eliminar.
class AdminCategoriesPage extends StatefulWidget {
  const AdminCategoriesPage({super.key});

  @override
  State<AdminCategoriesPage> createState() => _AdminCategoriesPageState();
}

class _AdminCategoriesPageState extends State<AdminCategoriesPage> {
  List<ShopCategory> _tree = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final data = await context.read<AdminProvider>().get(
      'admin/catalog/categories/tree',
    );
    if (data is List && mounted) {
      setState(() {
        _tree = data
            .map((e) => ShopCategory(Map<String, dynamic>.from(e as Map)))
            .toList();
        _loading = false;
      });
    } else if (mounted) {
      setState(() => _loading = false);
    }
  }

  Future<void> _openEditor({ShopCategory? category, int? parentId}) async {
    await showDialog(
      context: context,
      builder: (context) =>
          CategoryEditorDialog(category: category, parentId: parentId),
    );
    _load();
  }

  Future<void> _delete(int id) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete category?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      await admin.delete('admin/catalog/categories/$id');
      await _load();
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('$error'), backgroundColor: AppColors.red600),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Expanded(
                child: Text(
                  'Categories',
                  style: TextStyle(
                    fontFamily: 'Poppins',
                    fontSize: 24,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ),
              PrimaryButton(
                label: 'New Category',
                onPressed: () => _openEditor(parentId: 1),
              ),
            ],
          ),
          const SizedBox(height: 24),
          if (_loading)
            const Center(
              child: CircularProgressIndicator(color: AppColors.navyBlue),
            )
          else
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                border: Border.all(color: AppColors.zinc200),
              ),
              child: Column(
                children: [
                  const _CategoriesHeader(),
                  const Divider(height: 1, color: AppColors.zinc200),
                  ..._tree.map(
                    (category) => _CategoryTile(
                      category: category,
                      depth: 0,
                      onEdit: (item) => _openEditor(category: item),
                      onDelete: (item) => _delete(item.id ?? 0),
                      onAddChild: (item) => _openEditor(parentId: item.id),
                    ),
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}

class _CategoriesHeader extends StatelessWidget {
  const _CategoriesHeader();

  @override
  Widget build(BuildContext context) {
    const style = TextStyle(
      fontFamily: 'Poppins',
      fontSize: 12,
      fontWeight: FontWeight.w600,
      color: AppColors.zinc500,
    );
    return const Padding(
      padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      child: Row(
        children: [
          Expanded(child: Text('Name', style: style)),
          SizedBox(width: 100, child: Text('Status', style: style)),
          SizedBox(
            width: 90,
            child: Text('Position', style: style, textAlign: TextAlign.center),
          ),
          SizedBox(width: 144),
        ],
      ),
    );
  }
}

class _CategoryTile extends StatelessWidget {
  const _CategoryTile({
    required this.category,
    required this.depth,
    required this.onEdit,
    required this.onDelete,
    required this.onAddChild,
  });

  final ShopCategory category;
  final int depth;
  final void Function(ShopCategory) onEdit;
  final void Function(ShopCategory) onDelete;
  final void Function(ShopCategory) onAddChild;

  @override
  Widget build(BuildContext context) {
    final status = category.isActive ? AppColors.green600 : AppColors.zinc500;
    return Column(
      children: [
        InkWell(
          onTap: () => onEdit(category),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            child: Row(
              children: [
                Expanded(
                  child: Row(
                    children: [
                      SizedBox(width: 24.0 * depth),
                      if (depth > 0)
                        const Padding(
                          padding: EdgeInsets.only(right: 8),
                          child: Icon(
                            Icons.subdirectory_arrow_right,
                            size: 14,
                            color: AppColors.zinc400,
                          ),
                        ),
                      Flexible(
                        child: Text(
                          category.name,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            fontFamily: 'Poppins',
                            fontSize: 14,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                SizedBox(
                  width: 100,
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 10,
                        vertical: 3,
                      ),
                      decoration: BoxDecoration(
                        color: status.withValues(alpha: .1),
                        borderRadius: BorderRadius.circular(999),
                      ),
                      child: Text(
                        category.isActive ? 'Active' : 'Inactive',
                        style: TextStyle(
                          fontFamily: 'Poppins',
                          fontSize: 12,
                          fontWeight: FontWeight.w500,
                          color: status,
                        ),
                      ),
                    ),
                  ),
                ),
                SizedBox(
                  width: 90,
                  child: Text(
                    '${category.position}',
                    textAlign: TextAlign.center,
                    style: const TextStyle(
                      fontFamily: 'Poppins',
                      fontSize: 13,
                      color: AppColors.zinc600,
                    ),
                  ),
                ),
                SizedBox(
                  width: 144,
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      IconButton(
                        icon: const Icon(Icons.add, size: 18),
                        tooltip: 'Add subcategory',
                        onPressed: () => onAddChild(category),
                      ),
                      IconButton(
                        icon: const Icon(Icons.edit_outlined, size: 18),
                        tooltip: 'Edit',
                        onPressed: () => onEdit(category),
                      ),
                      IconButton(
                        icon: const Icon(
                          Icons.delete_outline,
                          size: 18,
                          color: AppColors.red600,
                        ),
                        tooltip: 'Delete',
                        onPressed: () => onDelete(category),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
        for (final child in category.children)
          _CategoryTile(
            category: child,
            depth: depth + 1,
            onEdit: onEdit,
            onDelete: onDelete,
            onAddChild: onAddChild,
          ),
      ],
    );
  }
}

class CategoryEditorDialog extends StatefulWidget {
  const CategoryEditorDialog({super.key, this.category, this.parentId});
  final ShopCategory? category;
  final int? parentId;

  @override
  State<CategoryEditorDialog> createState() => _CategoryEditorDialogState();
}

class _CategoryEditorDialogState extends State<CategoryEditorDialog> {
  late final TextEditingController _name;
  late final TextEditingController _description;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    _name = TextEditingController(text: widget.category?.name ?? '');
    _description = TextEditingController();
  }

  Future<void> _save() async {
    setState(() => _saving = true);
    try {
      if (widget.category == null) {
        final admin = context.read<AdminProvider>();
        await admin.post(
          'admin/catalog/categories/store',
          body: {
            'name': _name.text,
            'parent_id': widget.parentId ?? 1,
            'status': 1,
            'description': _description.text,
          },
        );
      } else {
        final admin = context.read<AdminProvider>();
        await admin.put(
          'admin/catalog/categories/edit/${widget.category!.id}',
          body: {
            'name': _name.text,
            'status': 1,
            'description': _description.text,
          },
        );
      }
      if (mounted) Navigator.of(context).pop();
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('$error'), backgroundColor: AppColors.red600),
        );
      }
      setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(widget.category == null ? 'New Category' : 'Edit Category'),
      content: SizedBox(
        width: 420,
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
              maxLines: 3,
              decoration: const InputDecoration(labelText: 'Description'),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        PrimaryButton(
          label: 'Save',
          isLoading: _saving,
          onPressed: _saving ? null : _save,
        ),
      ],
    );
  }
}
