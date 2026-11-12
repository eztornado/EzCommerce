import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import 'admin_shell.dart' show EzDataGrid;

/// Ventas: grids propios de facturas, envíos y reembolsos (estilo Bagisto,
/// con acciones hacia el pedido padre).
class AdminInvoicesPage extends StatelessWidget {
  const AdminInvoicesPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        EzDataGrid(
          gridPath: 'admin/sales/invoices',
          title: 'Invoices',
          extraActions: (row) => [
            if (row['order_id'] != null)
              IconButton(
                icon: const Icon(Icons.receipt_long_outlined, size: 18),
                tooltip: 'View order',
                onPressed: () => context.go('/admin/orders'),
              ),
          ],
        ),
      ],
    );
  }
}

class AdminShipmentsPage extends StatelessWidget {
  const AdminShipmentsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return EzDataGrid(
      gridPath: 'admin/sales/shipments',
      title: 'Shipments',
    );
  }
}

class AdminRefundsPage extends StatelessWidget {
  const AdminRefundsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return EzDataGrid(
      gridPath: 'admin/sales/refunds',
      title: 'Refunds',
    );
  }
}
