import 'package:flutter/material.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/theme/ez_theme.dart';

/// Atributo filtrable del catálogo (de /api/categories/attributes).
class FilterableAttribute {
  FilterableAttribute(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get code => (json['code'] ?? '') as String;
  String get label => (json['admin_name'] ?? json['code'] ?? '') as String;
}

class FilterOption {
  FilterOption(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get name => (json['admin_name'] ?? '') as String;
}

/// Sidebar de filtros del listado (desktop) / bottom-sheet (móvil):
/// accordions por atributo con checkboxes navyBlue + filtro de precio.
class CatalogFilters extends StatefulWidget {
  const CatalogFilters({super.key, required this.onApply, this.categoryId});

  final int? categoryId;
  final void Function(Map<String, String> filters) onApply;

  @override
  State<CatalogFilters> createState() => _CatalogFiltersState();
}

class _CatalogFiltersState extends State<CatalogFilters> {
  List<FilterableAttribute> _attributes = [];
  final Map<String, List<FilterOption>> _options = {};
  final Set<String> _selected = {}; // "code:optionId"
  double _maxPrice = 0;
  double _priceLimit = 0;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final attrs = await apiGet('categories/attributes',
          query: {if (widget.categoryId != null) 'category_id': '${widget.categoryId}'});
      if (attrs is! List) return;
      _attributes =
          attrs.map((e) => FilterableAttribute(Map<String, dynamic>.from(e as Map))).toList();

      for (final attribute in _attributes) {
        if (attribute.code == 'price' || attribute.code == 'new') continue;
        final options = await apiGet(
            'categories/attributes/${attribute.id}/options');
        if (options is List) {
          _options[attribute.code] =
              options.map((e) => FilterOption(Map<String, dynamic>.from(e as Map))).toList();
        }
      }

      final price = await apiGet(
          'categories/max-price/${widget.categoryId ?? 1}');
      if (price is Map && price['max_price'] is num) {
        _maxPrice = (price['max_price'] as num).toDouble();
        _priceLimit = _maxPrice;
      }
    } catch (_) {}
    if (mounted) setState(() => _loading = false);
  }

  Map<String, String> _currentFilters() {
    final filters = <String, String>{};
    for (final entry in _selected) {
      final parts = entry.split(':');
      final code = parts[0];
      final optionId = parts[1];
      filters.update(code, (value) => '$value,$optionId', ifAbsent: () => optionId);
    }
    if (_priceLimit > 0 && _priceLimit < _maxPrice) {
      filters['price_max'] = _priceLimit.round().toString();
    }
    return filters;
  }

  void _apply() => widget.onApply(_currentFilters());

  void _clear() {
    setState(() {
      _selected.clear();
      _priceLimit = _maxPrice;
    });
    widget.onApply({});
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const SizedBox(
        height: 200,
        child: Center(child: CircularProgressIndicator(color: AppColors.navyBlue)),
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            const Expanded(
              child: Text('Filters',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 17, fontWeight: FontWeight.w600)),
            ),
            GestureDetector(
              onTap: _clear,
              child: const Text('Clear All',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 12, fontWeight: FontWeight.w500,
                      color: AppColors.navyBlue)),
            ),
          ],
        ),
        const Divider(height: 20),
        // Precio
        if (_maxPrice > 0) ...[
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Row(
              children: [
                const Text('Price up to',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                const Spacer(),
                Text('${_priceLimit.round()} €',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 14,
                        fontWeight: FontWeight.w600, color: AppColors.navyBlue)),
              ],
            ),
          ),
          Slider(
            value: _priceLimit,
            max: _maxPrice,
            divisions: _maxPrice > 10 ? (_maxPrice / 10).round() : null,
            activeColor: AppColors.navyBlue,
            onChanged: (value) => setState(() => _priceLimit = value),
            onChangeEnd: (_) => _apply(),
          ),
          const Divider(),
        ],
        // Atributos en accordions
        ..._options.entries.map((entry) {
          return Container(
            decoration: const BoxDecoration(
                border: Border(bottom: BorderSide(color: AppColors.zinc200))),
            child: ExpansionTile(
              tilePadding: EdgeInsets.zero,
              childrenPadding: const EdgeInsets.only(bottom: 8),
              title: Text(entry.key,
                  style: const TextStyle(fontFamily: 'Poppins', fontSize: 15)),
              children: entry.value.map((option) {
                final key = '${entry.key}:${option.id}';
                return CheckboxListTile(
                  dense: true,
                  controlAffinity: ListTileControlAffinity.leading,
                  activeColor: AppColors.navyBlue,
                  contentPadding: EdgeInsets.zero,
                  title: Text(option.name,
                      style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                  value: _selected.contains(key),
                  onChanged: (checked) {
                    setState(() {
                      checked == true ? _selected.add(key) : _selected.remove(key);
                    });
                    _apply();
                  },
                );
              }).toList(),
            ),
          );
        }),
      ],
    );
  }
}

/// Barra inferior fija móvil: Filter (drawer) | Sort (bottom-sheet).
class MobileFilterBar extends StatelessWidget {
  const MobileFilterBar({
    super.key,
    required this.categoryId,
    required this.currentSort,
    required this.onSort,
    required this.onFilters,
  });

  final int? categoryId;
  final String currentSort;
  final ValueChanged<String> onSort;
  final void Function(Map<String, String> filters) onFilters;

  void _openFilters(BuildContext context) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.white,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(EzTheme.radius2Xl)),
      ),
      builder: (context) => SafeArea(
        child: Container(
          padding: const EdgeInsets.all(20),
          constraints: BoxConstraints(
              maxHeight: MediaQuery.of(context).size.height * 0.75),
          child: SingleChildScrollView(
            child: CatalogFilters(categoryId: categoryId, onApply: (filters) {
              Navigator.of(context).pop();
              onFilters(filters);
            }),
          ),
        ),
      ),
    );
  }

  void _openSort(BuildContext context) {
    showModalBottomSheet(
      context: context,
      backgroundColor: Colors.white,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(EzTheme.radius2Xl)),
      ),
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Padding(
              padding: EdgeInsets.all(16),
              child: Text('Sort by',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w600)),
            ),
            ...{
              'newest': 'Newest first',
              'price_asc': 'Price: Low to High',
              'price_desc': 'Price: High to Low',
              'name_asc': 'Name: A-Z',
            }.entries.map((entry) => ListTile(
                  title: Text(entry.value,
                      style: TextStyle(
                          fontFamily: 'Poppins',
                          fontSize: 14,
                          color: entry.key == currentSort
                              ? AppColors.navyBlue
                              : AppColors.gray900)),
                  trailing: entry.key == currentSort
                      ? const Icon(Icons.check, color: AppColors.navyBlue, size: 18)
                      : null,
                  onTap: () {
                    Navigator.of(context).pop();
                    onSort(entry.key);
                  },
                )),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Colors.white,
        border: Border(top: BorderSide(color: AppColors.zinc200)),
        boxShadow: [
          BoxShadow(color: Colors.black.withValues(alpha: 0.06), blurRadius: 8, offset: const Offset(0, -2)),
        ],
      ),
      child: Row(
        children: [
          Expanded(
            child: TextButton.icon(
              onPressed: () => _openFilters(context),
              icon: const Icon(Icons.filter_list, size: 18),
              label: const Text('Filter',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 14)),
            ),
          ),
          Container(width: 1, height: 24, color: AppColors.zinc200),
          Expanded(
            child: TextButton.icon(
              onPressed: () => _openSort(context),
              icon: const Icon(Icons.sort, size: 18),
              label: const Text('Sort',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 14)),
            ),
          ),
        ],
      ),
    );
  }
}
