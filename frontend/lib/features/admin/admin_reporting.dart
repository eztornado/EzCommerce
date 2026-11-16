import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../../../core/utils/download.dart';

enum _Range { last30, thisMonth, thisYear, custom }

/// Reporting: ventas/productos/clientes por rangos de fecha con export CSV.
class AdminReportingPage extends StatefulWidget {
  const AdminReportingPage({super.key});

  @override
  State<AdminReportingPage> createState() => _AdminReportingPageState();
}

class _AdminReportingPageState extends State<AdminReportingPage> {
  String _module = 'sales';
  _Range _range = _Range.last30;
  DateTime? _customStart;
  DateTime? _customEnd;
  Map<String, dynamic>? _sales;
  List<Map<String, dynamic>> _rows = [];
  bool _loading = true;

  String get _start {
    final now = DateTime.now();
    return switch (_range) {
      _Range.last30 =>
        now
            .subtract(const Duration(days: 30))
            .toIso8601String()
            .substring(0, 10),
      _Range.thisMonth => DateTime(
        now.year,
        now.month,
      ).toIso8601String().substring(0, 10),
      _Range.thisYear => DateTime(now.year).toIso8601String().substring(0, 10),
      _Range.custom => (_customStart ?? now).toIso8601String().substring(0, 10),
    };
  }

  String get _end =>
      (_range == _Range.custom && _customEnd != null
              ? _customEnd!
              : DateTime.now())
          .toIso8601String()
          .substring(0, 10);

  @override
  void initState() {
    super.initState();
    _load();
  }

  Map<String, String> get _query => {
    'start': _start,
    'end': _end,
    if (_module == 'sales') 'granularity': 'day',
  };

  Future<void> _load() async {
    setState(() => _loading = true);
    final admin = context.read<AdminProvider>();
    try {
      if (_module == 'sales') {
        final data = await admin.get('admin/reporting/sales', query: _query);
        _sales = data is Map ? Map<String, dynamic>.from(data) : null;
        _rows = [];
      } else {
        final data = await admin.get('admin/reporting/$_module', query: _query);
        _rows = decodeList(data);
        _sales = null;
      }
    } catch (_) {}
    if (mounted) setState(() => _loading = false);
  }

  Future<void> _exportCsv() async {
    final token = ApiClient.instance.adminToken;
    if (token == null) return;
    final params = {..._query, 'format': 'csv'};
    final query = Uri(queryParameters: params).query;
    final response = await http.get(
      Uri.parse('${ApiClient.baseUrl}/api/admin/reporting/$_module?$query'),
      headers: {'Authorization': 'Bearer $token'},
    );
    if (response.statusCode != 200 || !mounted) return;
    downloadFile('$_module-report.csv', response.bodyBytes);
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(const SnackBar(content: Text('Export CSV downloaded.')));
  }

  Future<void> _pickDate({bool start = true}) async {
    final initial = start
        ? (_customStart ?? DateTime.now())
        : (_customEnd ?? DateTime.now());
    final picked = await showDatePicker(
      context: context,
      initialDate: initial,
      firstDate: DateTime(2020),
      lastDate: DateTime.now().add(const Duration(days: 1)),
    );
    if (picked == null || !mounted) return;
    setState(() {
      if (start) {
        _customStart = picked;
      } else {
        _customEnd = picked;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Reporting',
            style: TextStyle(
              fontFamily: 'Poppins',
              fontSize: 24,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 24),
          Wrap(
            spacing: 16,
            runSpacing: 12,
            children: [
              SegmentedButton<String>(
                segments: const [
                  ButtonSegment(value: 'sales', label: Text('Sales')),
                  ButtonSegment(value: 'products', label: Text('Products')),
                  ButtonSegment(value: 'customers', label: Text('Customers')),
                ],
                selected: {_module},
                onSelectionChanged: (selection) {
                  setState(() => _module = selection.first);
                  _load();
                },
              ),
              DropdownButton<_Range>(
                value: _range,
                underline: const SizedBox.shrink(),
                items: const [
                  DropdownMenuItem(
                    value: _Range.last30,
                    child: Text('Last 30 days'),
                  ),
                  DropdownMenuItem(
                    value: _Range.thisMonth,
                    child: Text('This month'),
                  ),
                  DropdownMenuItem(
                    value: _Range.thisYear,
                    child: Text('This year'),
                  ),
                  DropdownMenuItem(
                    value: _Range.custom,
                    child: Text('Custom range'),
                  ),
                ],
                onChanged: (value) {
                  setState(() => _range = value ?? _Range.last30);
                  _range == _Range.custom ? null : _load();
                },
              ),
              if (_range == _Range.custom) ...[
                OutlinedButton(
                  onPressed: () async {
                    await _pickDate(start: true);
                    if (_customEnd != null) _load();
                  },
                  child: Text('From $_start'),
                ),
                OutlinedButton(
                  onPressed: () async {
                    await _pickDate(start: false);
                    if (_customStart != null) _load();
                  },
                  child: Text('To $_end'),
                ),
              ],
              OutlinedButton.icon(
                onPressed: _exportCsv,
                icon: const Icon(Icons.download_outlined, size: 16),
                label: const Text(
                  'Export CSV',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 12),
                ),
              ),
            ],
          ),
          const SizedBox(height: 24),
          if (_loading)
            const Center(
              child: CircularProgressIndicator(color: AppColors.navyBlue),
            )
          else if (_module == 'sales')
            _salesView()
          else
            _tableView(),
        ],
      ),
    );
  }

  Widget _salesView() {
    final totals = Map<String, dynamic>.from(
      (_sales?['totals'] ?? const {}) as Map,
    );
    final series = decodeList(_sales?['series']);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Wrap(
          spacing: 16,
          runSpacing: 12,
          children: [
            _statCard('Total sales', totals['total_sales']),
            _statCard('Orders', totals['total_orders']),
            _statCard('Tax collected', totals['tax_collected']),
            _statCard('Shipping', totals['shipping_collected']),
            _statCard('Refunds', totals['refunds']),
            _statCard('Coupon discounts', totals['coupon_discount']),
          ],
        ),
        const SizedBox(height: 24),
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: Colors.white,
            borderRadius: BorderRadius.circular(EzTheme.radiusXl),
            border: Border.all(color: AppColors.zinc200),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Sales over time',
                style: TextStyle(
                  fontFamily: 'Poppins',
                  fontSize: 15,
                  fontWeight: FontWeight.w500,
                ),
              ),
              const SizedBox(height: 16),
              if (series.isEmpty)
                const Text(
                  'No sales in this period.',
                  style: TextStyle(
                    fontFamily: 'Poppins',
                    fontSize: 12,
                    color: AppColors.zinc500,
                  ),
                )
              else
                EzBarChart(
                  bars: [
                    for (final point in series)
                      (
                        '${point['label']}',
                        (point['sales'] as num? ?? 0).toDouble(),
                      ),
                  ],
                ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: Colors.white,
            borderRadius: BorderRadius.circular(EzTheme.radiusXl),
            border: Border.all(color: AppColors.zinc200),
          ),
          child: Table(
            columnWidths: const {
              0: FlexColumnWidth(2),
              1: FlexColumnWidth(2),
              2: FlexColumnWidth(1),
            },
            border: TableBorder(
              horizontalInside: BorderSide(color: AppColors.zinc200),
            ),
            children: [
              const TableRow(
                children: [
                  Padding(
                    padding: EdgeInsets.all(8),
                    child: Text('Date', style: _headerStyle),
                  ),
                  Padding(
                    padding: EdgeInsets.all(8),
                    child: Text('Sales', style: _headerStyle),
                  ),
                  Padding(
                    padding: EdgeInsets.all(8),
                    child: Text('Orders', style: _headerStyle),
                  ),
                ],
              ),
              for (final point in series)
                TableRow(
                  children: [
                    Padding(
                      padding: const EdgeInsets.all(8),
                      child: Text(
                        '${point['label']}',
                        style: const TextStyle(
                          fontFamily: 'Poppins',
                          fontSize: 12,
                        ),
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.all(8),
                      child: Text(
                        '€${(point['sales'] as num? ?? 0).toStringAsFixed(2)}',
                        style: const TextStyle(
                          fontFamily: 'Poppins',
                          fontSize: 12,
                        ),
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.all(8),
                      child: Text(
                        '${point['orders']}',
                        style: const TextStyle(
                          fontFamily: 'Poppins',
                          fontSize: 12,
                        ),
                      ),
                    ),
                  ],
                ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _tableView() {
    final isProducts = _module == 'products';
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: _rows.isEmpty
          ? const Text(
              'No data in this period.',
              style: TextStyle(
                fontFamily: 'Poppins',
                fontSize: 12,
                color: AppColors.zinc500,
              ),
            )
          : Table(
              columnWidths: const {
                0: FlexColumnWidth(3),
                1: FlexColumnWidth(2),
                2: FlexColumnWidth(1),
                3: FlexColumnWidth(2),
              },
              border: TableBorder(
                horizontalInside: BorderSide(color: AppColors.zinc200),
              ),
              children: [
                TableRow(
                  children: [
                    Padding(
                      padding: const EdgeInsets.all(8),
                      child: Text(
                        isProducts ? 'Product' : 'Customer',
                        style: _headerStyle,
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.all(8),
                      child: Text(
                        isProducts ? 'SKU' : 'Email',
                        style: _headerStyle,
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.all(8),
                      child: Text(
                        isProducts ? 'Qty' : 'Orders',
                        style: _headerStyle,
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.all(8),
                      child: Text('Revenue', style: _headerStyle),
                    ),
                  ],
                ),
                for (final row in _rows)
                  TableRow(
                    children: [
                      Padding(
                        padding: const EdgeInsets.all(8),
                        child: Text(
                          isProducts
                              ? '${row['name']}'
                              : '${row['first_name']} ${row['last_name']}'
                                    .trim(),
                          style: const TextStyle(
                            fontFamily: 'Poppins',
                            fontSize: 12,
                          ),
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.all(8),
                        child: Text(
                          isProducts ? '${row['sku']}' : '${row['email']}',
                          style: const TextStyle(
                            fontFamily: 'Poppins',
                            fontSize: 12,
                          ),
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.all(8),
                        child: Text(
                          '${row[isProducts ? 'qty' : 'orders']}',
                          style: const TextStyle(
                            fontFamily: 'Poppins',
                            fontSize: 12,
                          ),
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.all(8),
                        child: Text(
                          '€${((row['total'] as num? ?? 0)).toStringAsFixed(2)}',
                          style: const TextStyle(
                            fontFamily: 'Poppins',
                            fontSize: 12,
                          ),
                        ),
                      ),
                    ],
                  ),
              ],
            ),
    );
  }

  Widget _statCard(String label, dynamic value) {
    return Container(
      padding: const EdgeInsets.all(16),
      width: 180,
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: const TextStyle(
              fontFamily: 'Poppins',
              fontSize: 11,
              color: AppColors.zinc500,
            ),
          ),
          const SizedBox(height: 6),
          Text(
            value is num ? value.toStringAsFixed(2) : '$value',
            style: const TextStyle(
              fontFamily: 'Poppins',
              fontSize: 20,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
}

const _headerStyle = TextStyle(
  fontFamily: 'Poppins',
  fontSize: 12,
  fontWeight: FontWeight.w600,
);

/// Gráfica de barras simple sin dependencias externas.
/// Gráfica de área de las ventas (línea navy + relleno degradado), con
/// líneas de referencia y etiquetas de importe. Se usa en dashboard y
/// reporting para cualquier serie temporal.
class EzBarChart extends StatelessWidget {
  const EzBarChart({super.key, required this.bars});

  final List<(String, double)> bars;

  @override
  Widget build(BuildContext context) {
    final max = bars.fold(0.0, (max, bar) => bar.$2 > max ? bar.$2 : max);
    if (max <= 0) return const SizedBox.shrink();

    const chartHeight = 190.0;
    final step = bars.length <= 7 ? 1 : (bars.length / 6).ceil();

    return Column(
      children: [
        SizedBox(
          height: chartHeight,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              SizedBox(
                width: 58,
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    for (var i = 4; i >= 0; i--)
                      Padding(
                        padding: const EdgeInsets.only(right: 10),
                        child: Text(
                          _compact(max * i / 4),
                          style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 10,
                            color: AppColors.zinc400,
                          ),
                        ),
                      ),
                  ],
                ),
              ),
              Expanded(
                child: LayoutBuilder(
                  builder: (context, constraints) => CustomPaint(
                    size: Size(constraints.maxWidth, chartHeight),
                    painter: _SalesChartPainter(
                      values: [for (final bar in bars) bar.$2],
                      max: max,
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 8),
        Padding(
          padding: const EdgeInsets.only(left: 58),
          child: Row(
            children: [
              for (var i = 0; i < bars.length; i++)
                Expanded(
                  child: i % step == 0 || i == bars.length - 1
                      ? Text(
                          _label(bars[i].$1),
                          textAlign: TextAlign.center,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 9,
                            color: AppColors.zinc400,
                          ),
                        )
                      : const SizedBox.shrink(),
                ),
            ],
          ),
        ),
      ],
    );
  }

  String _compact(double value) {
    if (value >= 1000) return '€${(value / 1000).toStringAsFixed(1)}k';
    return '€${value.toStringAsFixed(0)}';
  }

  String _label(String value) {
    if (value.length == 10) return value.substring(5);
    return value;
  }
}

class _SalesChartPainter extends CustomPainter {
  _SalesChartPainter({required this.values, required this.max});

  final List<double> values;
  final double max;

  @override
  void paint(Canvas canvas, Size size) {
    final grid = Paint()
      ..color = AppColors.zinc200
      ..strokeWidth = 1;
    for (var i = 0; i <= 4; i++) {
      final y = size.height * i / 4;
      canvas.drawLine(Offset(0, y), Offset(size.width, y), grid);
    }

    if (values.isEmpty) return;
    final points = <Offset>[];
    for (var i = 0; i < values.length; i++) {
      final x = values.length == 1
          ? size.width / 2
          : size.width * i / (values.length - 1);
      final y = size.height - (values[i] / max) * (size.height - 16) - 8;
      points.add(Offset(x, y));
    }

    final area = Path()..moveTo(points.first.dx, size.height);
    for (final point in points) {
      area.lineTo(point.dx, point.dy);
    }
    area.lineTo(points.last.dx, size.height);
    area.close();
    canvas.drawPath(
      area,
      Paint()
        ..shader = const LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [Color(0x33060C3B), Color(0x00060C3B)],
        ).createShader(Offset.zero & size),
    );

    final line = Path()..moveTo(points.first.dx, points.first.dy);
    for (var i = 1; i < points.length; i++) {
      line.lineTo(points[i].dx, points[i].dy);
    }
    canvas.drawPath(
      line,
      Paint()
        ..color = AppColors.navyBlue
        ..strokeWidth = 2.5
        ..style = PaintingStyle.stroke
        ..strokeJoin = StrokeJoin.round
        ..strokeCap = StrokeCap.round,
    );

    final dotFill = Paint()..color = Colors.white;
    final dotBorder = Paint()
      ..color = AppColors.navyBlue
      ..strokeWidth = 2
      ..style = PaintingStyle.stroke;
    for (final point in points) {
      canvas.drawCircle(point, 4, dotFill);
      canvas.drawCircle(point, 4, dotBorder);
    }
  }

  @override
  bool shouldRepaint(covariant _SalesChartPainter oldDelegate) =>
      oldDelegate.values != values || oldDelegate.max != max;
}
