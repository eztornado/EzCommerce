import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';

/// Search SEO: url rewrites, términos buscados, sinónimos, sitemap y
/// suscriptores del newsletter.
class AdminSearchSeoPage extends StatefulWidget {
  const AdminSearchSeoPage({super.key});

  @override
  State<AdminSearchSeoPage> createState() => _AdminSearchSeoPageState();
}

class _AdminSearchSeoPageState extends State<AdminSearchSeoPage>
    with SingleTickerProviderStateMixin {
  late final TabController _tabs = TabController(length: 5, vsync: this);

  @override
  void dispose() {
    _tabs.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        backgroundColor: Colors.white,
        elevation: 0,
        automaticallyImplyLeading: false,
        title: const Text('Search SEO',
            style: TextStyle(
                fontFamily: 'Poppins', fontSize: 22, fontWeight: FontWeight.w500)),
        bottom: TabBar(
          controller: _tabs,
          isScrollable: true,
          labelColor: AppColors.navyBlue,
          unselectedLabelColor: AppColors.zinc600,
          indicatorColor: AppColors.navyBlue,
          tabs: const [
            Tab(text: 'URL rewrites'),
            Tab(text: 'Search terms'),
            Tab(text: 'Synonyms'),
            Tab(text: 'Subscribers'),
            Tab(text: 'Sitemap'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabs,
        children: const [
          UrlRewritesTab(),
          SearchTermsTab(),
          SynonymsTab(),
          SubscribersTab(),
          SitemapTab(),
        ],
      ),
    );
  }
}

class UrlRewritesTab extends StatefulWidget {
  const UrlRewritesTab({super.key});

  @override
  State<UrlRewritesTab> createState() => _UrlRewritesTabState();
}

class _UrlRewritesTabState extends State<UrlRewritesTab> {
  List<Map<String, dynamic>> _rows = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final data = await admin.post('admin/seo/url-rewrites', body: {});
    if (mounted) {
      setState(() {
        final payload = data is Map ? (data['data'] ?? data) : data;
        _rows = decodeList(payload is Map ? payload['records'] ?? payload : payload);
        _loading = false;
      });
    }
  }

  void _showError(Object error) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
  }

  Future<void> _dialog([Map<String, dynamic>? rewrite]) async {
    final request = TextEditingController(text: '${rewrite?['request_path'] ?? ''}');
    final target = TextEditingController(text: '${rewrite?['target_path'] ?? ''}');
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(rewrite == null ? 'New URL rewrite' : 'Edit URL rewrite'),
        content: SizedBox(
          width: 460,
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            TextField(
              controller: request,
              decoration: const InputDecoration(
                  labelText: 'Request path (e.g. /outlet)', helperText: 'La tienda resuelve este slug'),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: target,
              decoration: const InputDecoration(
                  labelText: 'Target path (e.g. /product/leather-tote-bag)'),
            ),
          ]),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
          PrimaryButton(
            label: 'Save',
            onPressed: () => (request.text.trim().isEmpty || target.text.trim().isEmpty)
                ? null
                : Navigator.pop(context, true),
          ),
        ],
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      rewrite == null
          ? await admin.post('admin/seo/url-rewrites/store', body: {
              'request_path': request.text.trim(),
              'target_path': target.text.trim(),
            })
          : await admin.put('admin/seo/url-rewrites/edit/${rewrite['id']}', body: {
              'request_path': request.text.trim(),
              'target_path': target.text.trim(),
            });
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
    return ListView(padding: const EdgeInsets.all(24), children: [
      Row(children: [
        const Spacer(),
        TextButton.icon(
          onPressed: () => _dialog(),
          icon: const Icon(Icons.add, size: 18),
          label: const Text('New rewrite', style: TextStyle(fontFamily: 'Poppins')),
        ),
      ]),
      for (final row in _rows)
        ListTile(
          title: Text('${row['request_path']} → ${row['target_path']}',
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
          trailing: Row(mainAxisSize: MainAxisSize.min, children: [
            IconButton(
              icon: const Icon(Icons.edit_outlined, size: 18),
              onPressed: () => _dialog(row),
            ),
            IconButton(
              icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
              onPressed: () async {
                final admin = context.read<AdminProvider>();
                await admin.delete('admin/seo/url-rewrites/${row['id']}');
                await _load();
              },
            ),
          ]),
        ),
    ]);
  }
}

class SearchTermsTab extends StatefulWidget {
  const SearchTermsTab({super.key});

  @override
  State<SearchTermsTab> createState() => _SearchTermsTabState();
}

class _SearchTermsTabState extends State<SearchTermsTab> {
  List<Map<String, dynamic>> _rows = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final data = await admin.post('admin/seo/search-terms', body: {});
    if (mounted) {
      setState(() {
        final payload = data is Map ? (data['data'] ?? data) : data;
        _rows = decodeList(payload is Map ? payload['records'] ?? payload : payload);
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    return ListView(padding: const EdgeInsets.all(24), children: [
      const Text('Terms visitors searched for, with results and usage.',
          style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
      const SizedBox(height: 16),
      for (final row in _rows)
        ListTile(
          title: Text('${row['term']}',
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
          subtitle: Text('results: ${row['results']} · uses: ${row['uses']}',
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
          trailing: IconButton(
            icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
            onPressed: () async {
              final admin = context.read<AdminProvider>();
              await admin.delete('admin/seo/search-terms/${row['id']}');
              await _load();
            },
          ),
        ),
    ]);
  }
}

class SynonymsTab extends StatefulWidget {
  const SynonymsTab({super.key});

  @override
  State<SynonymsTab> createState() => _SynonymsTabState();
}

class _SynonymsTabState extends State<SynonymsTab> {
  List<Map<String, dynamic>> _rows = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final data = await admin.post('admin/seo/search-synonyms', body: {});
    if (mounted) {
      setState(() {
        final payload = data is Map ? (data['data'] ?? data) : data;
        _rows = decodeList(payload is Map ? payload['records'] ?? payload : payload);
        _loading = false;
      });
    }
  }

  void _showError(Object error) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
  }

  Future<void> _dialog([Map<String, dynamic>? synonym]) async {
    final name = TextEditingController(text: '${synonym?['name'] ?? ''}');
    final terms = TextEditingController(text: '${synonym?['terms'] ?? ''}');
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(synonym == null ? 'New synonym' : 'Edit synonym'),
        content: SizedBox(
          width: 460,
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            TextField(
              controller: name,
              decoration: const InputDecoration(
                  labelText: 'Term', helperText: 'El término que busca la gente'),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: terms,
              decoration: const InputDecoration(
                  labelText: 'Synonyms (comma separated)',
                  helperText: 'Se incluyen en la búsqueda con OR'),
            ),
          ]),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
          PrimaryButton(
            label: 'Save',
            onPressed: () => (name.text.trim().isEmpty || terms.text.trim().isEmpty)
                ? null
                : Navigator.pop(context, true),
          ),
        ],
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      synonym == null
          ? await admin.post('admin/seo/search-synonyms/store', body: {
              'name': name.text.trim(),
              'terms': terms.text.trim(),
            })
          : await admin.put('admin/seo/search-synonyms/edit/${synonym['id']}', body: {
              'name': name.text.trim(),
              'terms': terms.text.trim(),
            });
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
    return ListView(padding: const EdgeInsets.all(24), children: [
      Row(children: [
        const Spacer(),
        TextButton.icon(
          onPressed: () => _dialog(),
          icon: const Icon(Icons.add, size: 18),
          label: const Text('New synonym', style: TextStyle(fontFamily: 'Poppins')),
        ),
      ]),
      for (final row in _rows)
        ListTile(
          title: Text('${row['name']} → ${row['terms']}',
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
          trailing: Row(mainAxisSize: MainAxisSize.min, children: [
            IconButton(
              icon: const Icon(Icons.edit_outlined, size: 18),
              onPressed: () => _dialog(row),
            ),
            IconButton(
              icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
              onPressed: () async {
                final admin = context.read<AdminProvider>();
                await admin.delete('admin/seo/search-synonyms/${row['id']}');
                await _load();
              },
            ),
          ]),
        ),
    ]);
  }
}

class SubscribersTab extends StatefulWidget {
  const SubscribersTab({super.key});

  @override
  State<SubscribersTab> createState() => _SubscribersTabState();
}

class _SubscribersTabState extends State<SubscribersTab> {
  List<Map<String, dynamic>> _rows = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final data = await admin.get('admin/seo/subscribers');
    if (mounted) {
      setState(() {
        _rows = decodeList(data);
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    return ListView(padding: const EdgeInsets.all(24), children: [
      const Text('Newsletter subscribers (destinatarios de las campañas).',
          style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
      const SizedBox(height: 16),
      for (final row in _rows)
        ListTile(
          title: Text('${row['email']}',
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
          subtitle: Text('${row['first_name'] ?? ''} ${row['last_name'] ?? ''}'.trim(),
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
          trailing: TextButton(
            onPressed: () async {
              final admin = context.read<AdminProvider>();
              await admin.put('admin/seo/subscribers/${row['id']}', body: {'subscribed': 0});
              await _load();
            },
            child: const Text('Unsubscribe',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.red600)),
          ),
        ),
    ]);
  }
}

class SitemapTab extends StatelessWidget {
  const SitemapTab({super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(padding: const EdgeInsets.all(24), children: [
      const Text('Sitemap',
          style: TextStyle(fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500)),
      const SizedBox(height: 12),
      const Text(
          'El sitemap se genera dinámicamente en /api/seo/sitemap.xml '
          '(productos visibles y categorías activas). Añádelo a Search Console '
          'apuntando a https://tu-dominio/api/seo/sitemap.xml',
          style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
    ]);
  }
}
