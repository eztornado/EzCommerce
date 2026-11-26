# EzCommerce

Full-stack e-commerce platform: a responsive online store and an admin panel,
built with a **Rust API backend** (Larust framework) and a **React web
frontend with Mantine**.

Designed to be deployed with Docker (Docker Compose / Coolify) using
**MySQL/MariaDB** in production and SQLite as the development database.

## Features

### Storefront (React, responsive web)

- Configurable home page with sections (image carousels, products and
  categories, content banners)
- Catalog with search, attribute and price filters, sorting
- Product detail page with gallery, configurable variants with color swatches
  and grouped products
- Guest cart (signed cart token) and registered customer cart, coupons and
  shipping estimation
- One-page checkout: address, shipping method, payment method, review and
  confirmation
- Stripe Checkout (hosted redirect) for card and wallet payments
- Customer account: profile, orders (detail, cancel, reorder), addresses,
  wishlist, product reviews
- Full authentication: register, login, password reset
- Language and channel currency selector in the header

### Admin Panel (React)

- Dashboard with sales metrics, orders, customers and a 30-day chart
- Generic DataGrid with search, filters, sorting, pagination, bulk actions and
  CSV export
- Products: creation by type (simple, virtual, configurable, grouped, bundle,
  downloadable), tabbed editor with special prices by date range, linked
  products, variants, inventory by warehouse, images and product copy
- Categories (hierarchical tree), attributes with families and groups
- Orders with full lifecycle: invoices, shipments (multi-warehouse), refunds and
  cancellation, with derived states from quantities and stock deduction
- Stripe configuration, payment status, card details, captures, refunds,
  disputes and per-order transaction history
- Customers: profile with orders, addresses, reviews and wishlist; customer
  groups and review moderation
- Marketing: cart rules with coupons (percentage, fixed amount, buy X get Y,
  proportional split) and catalog rules with price reindexing
- Communications: email templates, events and campaigns with sending to
  subscribers
- Search SEO: URL rewrites, search terms and synonyms
- CMS, localization (channels, currencies, languages), users and roles with ACL
- Reporting by date ranges with CSV export

### Backend API (Rust)

- JSON API for store and admin with JWT (Laravel-style claims, token blacklist
  and refresh)
- Cart engine with total calculation, taxes by zone, scheduled and free
  shipping rates, and inventory reservation
- Stripe Checkout Sessions with signed webhooks, manual capture, refunds,
  disputes, transaction log and automatic invoice creation
- Materialized prices (`product_flat`, price and inventory indexes) in the
  style of Bagisto
- Localization by language and currency conversion with exchange rates
- Bookings by slots for booking-type products
- SQL migrations with dual dialect (SQLite/MySQL) and idempotent seeders
- OpenAPI 3.0 document, Swagger UI and a JWT-protected MCP tool server
- Service health at `/up` and image storage with static serving

## Repository Structure

```
backend/         Rust API (Larust): controllers, cart/sales engine,
                 SQL migrations and seeders
frontend-react/  React web app (Mantine): store and admin panel
```

## Requirements

- Stable Rust (backend)
- Node 20+ (frontend)
- MariaDB/MySQL 10.4+ in production; SQLite for local development

## Demo Data

With `SEED_DEMO_DATA=true` the seeder generates a complete package to test
the platform: 11 catalog products (including a configurable with three color
variants), home sections, coupons (10% automatic and `WELCOME10`), catalog
rule, customers, orders with invoices and shipments (real derived states:
pending, processing, completed), approved reviews and wishlist. The seeder is
idempotent: re-running it does not duplicate data.

Demo credentials:

| Role | Identifier | Password |
|---|---|---|
| Admin | value of `ADMIN_EMAIL` (default `admin@example.com`) | value of `ADMIN_PASSWORD` (default `admin123`) |
| Customer | `customer@example.com` | `customer123` |
| Customer 2 | `marc@example.com` | `customer123` |
| Customer 3 | `laura@example.com` | `customer123` |

## Getting Started (Development)

Backend:

```bash
cd backend
cp .env.example .env
cargo run -- migrate:fresh --seed   # full schema + initial data
cargo run                           # API at http://127.0.0.1:34187
```

Frontend:

```bash
cd frontend-react
npm install
npm run dev   # Vite with proxy /api -> http://127.0.0.1:34187
```

The API URL is configured with `VITE_API_BASE` in `frontend-react` (empty by
default: relative `/api` routes on the same domain, with Vite proxy in
development). Admin credentials and other parameters are defined in
`backend/.env` (see `.env.example`).

## Stripe Payments (Redirect Checkout)

Stripe uses **hosted Checkout with redirection**: the customer is sent to
Stripe, pays there and comes back to the storefront. The order is marked as
paid only after Stripe confirms the charge through a signed webhook (the
return page also verifies the Checkout Session as a fallback).

### 1. Apply the migrations

```bash
cd backend
cargo run -- migrate        # payment_status, transaction fields and webhook log
```

If you are creating the database from scratch, `migrate:fresh --seed` also
includes these tables and columns.

### 2. Configure the keys

The recommended way is environment variables (never commit them):

```env
STRIPE_SECRET_KEY=sk_test_...
STRIPE_PUBLISHABLE_KEY=pk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
```

They can also be stored from **Admin → Configuration → Payment methods →
Stripe**; environment variables take precedence over `core_config`. Activate
the gateway and save the configuration. Stripe will only appear at checkout
when it is active and a secret key is available.

### 3. Configure the webhook

In the Stripe Dashboard, go to **Developers → Webhooks** and add:

```text
https://your-domain.com/api/webhooks/stripe
```

Select at least these events:

- `checkout.session.completed`
- `checkout.session.async_payment_succeeded`
- `checkout.session.async_payment_failed`
- `checkout.session.expired`
- `payment_intent.succeeded`
- `payment_intent.payment_failed`
- `payment_intent.amount_capturable_updated`
- `charge.dispute.created`
- `charge.dispute.closed`
- `charge.refund.updated`

For local development, use the Stripe CLI:

```bash
stripe listen --forward-to localhost:34187/api/webhooks/stripe
# copy the printed whsec_... value into STRIPE_WEBHOOK_SECRET
```

### 4. Test the flow

Use the test card `4242 4242 4242 4242`, any future expiry date and any CVC.

1. The order is created with `pending` / `pending_payment`.
2. The browser is redirected to Stripe Checkout.
3. When the payment succeeds, the webhook records the transaction, marks the
   order as `paid` and issues the invoice automatically.
4. If the customer cancels or the Checkout Session expires (~31 minutes), the
   order is canceled, stock is released and the cart is restored.

### Refunds, manual capture and disputes

- **Refunds**: use the regular Refund action in the order. EzCommerce calls
  `POST /v1/refunds` first and only then records the local refund, storing the
  Stripe id (`re_...`) and the gateway state. Partial refunds are supported.
  If Stripe rejects the refund, no local refund is created. Pending refunds are
  updated when Stripe sends `charge.refund.updated`.
- **Manual capture**: set **Capture method** to *Manual (authorize only)* in
  the Stripe settings. The order stays `authorized` after checkout and the
  order view shows a **Capture payment** action. The invoice is issued when
  the payment is captured.
- **Disputes/chargebacks**: `charge.dispute.created` and
  `charge.dispute.closed` update the order payment status
  (`disputed` / `paid` when won / `refunded` when lost), store the dispute id,
  reason and status, and add an entry to the order transactions.
- **Itemized receipt**: Checkout Sessions are created with one line per
  product plus tax and shipping; order discounts are applied as a one-off
  Stripe coupon. If the amounts cannot be reconciled exactly, EzCommerce falls
  back to a single line with the order total to guarantee the charge matches
  the order.

### Behavior and security

- Webhook signatures are verified (HMAC-SHA256, 5-minute tolerance) and events
  are deduplicated by event id.
- Unpaid or merely authorized Stripe orders cannot be shipped or refunded.
- Paid Stripe orders must be refunded rather than cancelled directly; the
  Cancel action is hidden while the charge is captured.
- A return to the success page verifies the Checkout Session, so the flow also
  works in local environments where the webhook has not arrived yet.

## Deployment with Docker Compose

```bash
JWT_SECRET=<secret> DB_PASSWORD=<password> docker compose up -d
```

Services:

- `api` — Rust binary; runs migrations and seeders before serving
  (healthcheck at `/up`, volume `api_storage` for uploaded images)
- `db` — MariaDB 11 with persistent volume
- `web` — nginx serving the compiled React SPA and proxying `/api` and
  `/storage` to the API (same origin, no CORS)

Main variables (see `docker-compose.yml`): `JWT_SECRET`, `DB_*`,
`ADMIN_EMAIL`, `ADMIN_PASSWORD`, `SEED_DEMO_DATA`, `FRONTEND_URL` and,
optionally, `STRIPE_SECRET_KEY`, `STRIPE_PUBLISHABLE_KEY` and
`STRIPE_WEBHOOK_SECRET`.

## Deployment on Coolify

1. Create the project from the Git repository choosing **Docker Compose**
   (`docker-compose.yml` at the root)
2. Define the required variables (`JWT_SECRET`, `DB_PASSWORD`,
   `DB_ROOT_PASSWORD`, `ADMIN_EMAIL`, `ADMIN_PASSWORD`) and the recommended
   ones (`APP_URL`, `FRONTEND_URL` for CORS, `SEED_DEMO_DATA=false`). For
   Stripe, add `STRIPE_SECRET_KEY`, `STRIPE_PUBLISHABLE_KEY` and
   `STRIPE_WEBHOOK_SECRET`
3. Assign domains in Coolify: `web` service → public store domain (the nginx
   service proxies `/api` and `/storage` to the API, no CORS); the `api`
   service does not need a public domain (healthcheck `/up`)
4. Mount persistent volumes: `db_data` (database) and `api_storage`
   (product images)
5. Deploy: the API migrates and seeds automatically on first startup

Services use `expose` instead of published `ports`: Coolify manages access
through its domains and proxy. If direct port access is desired (e.g., for
testing without a domain), add the desired mapping in the service UI or
replace `expose` with `ports` in the compose (`"8080:80"` for the store,
`"34187:34187"` for the API).

The SPA is built with `API_BASE_URL=SAME_ORIGIN` (relative `/api` calls on
the same domain). If the API is published on another domain, change that
build-arg.

## CI

GitHub Actions (`.github/workflows/ci.yml`) validates on every push:

- Backend: `cargo check` + migrations and seeders on SQLite
- Frontend: `npm install` + `npm run build` (TypeScript + Vite)
- Docker image builds (API and web)

## API

JSON API for store (`/api/...`) and admin (`/api/admin/...`) with JWT
authentication via `Authorization: Bearer` header. Errors follow the Laravel
format (`{message}` and 422 validation with `{message, errors}`); paginated
collections use the standard form `{data, links, meta}`.

### API documentation and MCP

- Swagger UI: `http://<host>/api/docs` (public). `/docs` redirects there.
- OpenAPI 3.0 JSON: `http://<host>/api/openapi.json`.
- MCP server: `POST /api/mcp`, JSON-RPC 2.0 over HTTP, authenticated with the
  same admin JWT as the admin API (`Authorization: Bearer <token>`). Supports
  `initialize`, `tools/list` and `tools/call`; it is documented in the OpenAPI
  document with its tools and JWT security scheme.

MCP tools (full schemas are returned by `tools/list` and shown in the OpenAPI
`x-mcp-tools` extension):

- Catalog: `list_products`, `get_product`, `list_categories`, `get_category`,
  `list_attributes`, `list_attribute_families`, `get_product_stock`,
  `low_stock_products`, `create_product`, `update_product`, `delete_product`,
  `set_product_stock`, `create_category`, `update_category`, `delete_category`.
- Sales: `list_orders`, `get_order`, `create_invoice`, `cancel_order`,
  `list_invoices`, `get_invoice`, `list_shipments`, `get_shipment`,
  `create_shipment`, `create_refund`, `add_order_comment`,
  `list_transactions`, `list_refunds`, `sales_summary`.
- Customers: `list_customers`, `get_customer`, `create_customer`,
  `update_customer`.
- Marketing and content: `list_cart_rules`, `get_cart_rule`,
  `list_catalog_rules`, `get_catalog_rule`, `list_cms_pages`, `get_cms_page`,
  `list_reviews`, `create_cms_page`, `update_cms_page`, `delete_cms_page`.
- Store settings: `list_channels`, `list_locales`, `list_currencies`,
  `list_tax_rates`, `list_inventory_sources`, `save_configuration`.

```bash
curl -X POST http://<host>/api/mcp \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

## License

Internal use. Contact the repository owner before reusing.

---

# EzCommerce

Plataforma de comercio electrónico de código completo: una tienda online
responsive y un panel de administración, construidos con un **backend API en
Rust** (framework Larust) y un **frontend web en React con Mantine**.

Diseñado para desplegarse dockerizado (Docker Compose / Coolify) con
**MySQL/MariaDB** en producción y SQLite como base de datos de desarrollo.

## Características

### Tienda (React, responsive web)

- Página de inicio configurable por secciones (carruseles de imágenes,
  productos y categorías, banners de contenido)
- Catálogo con búsqueda, filtros por atributos y precio, ordenación
- Ficha de producto con galería, variantes configurables con swatches de
  color y productos agrupados
- Carrito para invitados (token de carrito firmado) y clientes registrados,
  cupones y estimación de envío
- Checkout de una página: dirección, método de envío, método de pago,
  revisión y confirmación
- Checkout de Stripe (redirección alojada) para pagos con tarjeta y wallets
- Cuenta de cliente: perfil, pedidos (detalle, cancelación, reordenar),
  direcciones, wishlist, reseñas de productos
- Autenticación completa: registro, login, restablecimiento de contraseña
- Selector de idioma y moneda del canal en la cabecera

### Panel de administración (React)

- Dashboard con métricas de ventas, pedidos, clientes y gráfica de los
  últimos 30 días
- DataGrid genérico con búsqueda, filtros, ordenación, paginación, acciones
  masivas y exportación CSV
- Productos: creación por tipos (simple, virtual, configurable, agrupado,
  bundle, descargable), editor por pestañas con precios especiales por
  ventana de fechas, linked products, variantes, inventario por almacén,
  imágenes y copia de producto
- Categorías (árbol jerárquico), atributos con familias y grupos
- Pedidos con ciclo completo: facturas, envíos (multi-almacén), reembolsos y
  cancelación, con estados derivados de las cantidades y descuento de stock
- Configuración de Stripe, estado del pago, datos de tarjeta, capturas,
  reembolsos, disputas e historial de transacciones por pedido
- Clientes: ficha con pedidos, direcciones, reseñas y wishlist; grupos de
  clientes y moderación de reseñas
- Marketing: reglas de carrito con cupones (porcentaje, importe fijo, compra
  X consigue Y, reparto proporcional) y reglas de catálogo con reindexado de
  precios
- Communications: plantillas de email, eventos y campañas con envío a los
  suscriptores
- Search SEO: URL rewrites, términos y sinónimos de búsqueda
- CMS, localización (canales, monedas, idiomas), usuarios y roles con ACL
- Reporting por rangos de fecha con export CSV

### Backend API (Rust)

- API JSON para tienda y administración con JWT (claims estilo Laravel,
  blacklist de tokens y refresco)
- Motor de carrito con cálculo de totales, impuestos por zona, tarifas de
  envío planificadas y gratuitas, y reserva de inventario
- Sesiones de Stripe Checkout con webhooks firmados, captura manual,
  reembolsos, disputas, registro de transacciones y emisión automática de
  facturas
- Precios materializados (`product_flat`, índices de precio e inventario) al
  estilo de Bagisto
- Localización por idioma y conversión de moneda con tasas de cambio
- Reservas (bookings) por slots para productos de tipo booking
- Migraciones SQL con doble dialecto (SQLite/MySQL) y seeders idempotentes
- Documento OpenAPI 3.0, Swagger UI y servidor de herramientas MCP protegido
  por JWT
- Salud del servicio en `/up` y almacenamiento de imágenes con servir estático

## Estructura del repositorio

```
backend/         API en Rust (Larust): controladores, motor de carrito/ventas,
                 migraciones SQL y seeders
frontend-react/  App web React (Mantine): tienda y panel de administración
```

## Requisitos

- Rust estable (backend)
- Node 20+ (frontend)
- MariaDB/MySQL 10.4+ en producción; SQLite para desarrollo local

## Datos de ejemplo

Con `SEED_DEMO_DATA=true` el seeder genera un paquete completo para probar
la plataforma: 11 productos de catálogo (incluido un configurable con tres
variantes de color), secciones de home, cupones (10% automático y
`WELCOME10`), regla de catálogo, clientes, pedidos con facturas y envíos
(estados derivados reales: pendiente, en proceso, completado), reseñas
aprobadas y wishlist. El seeder es idempotente: re-ejecutarlo no duplica
datos.

Credenciales de los datos demo:

| Rol | Identificador | Contraseña |
|---|---|---|
| Administrador | valor de `ADMIN_EMAIL` (por defecto `admin@example.com`) | valor de `ADMIN_PASSWORD` (por defecto `admin123`) |
| Cliente | `customer@example.com` | `customer123` |
| Cliente 2 | `marc@example.com` | `customer123` |
| Cliente 3 | `laura@example.com` | `customer123` |

## Puesta en marcha (desarrollo)

Backend:

```bash
cd backend
cp .env.example .env
cargo run -- migrate:fresh --seed   # esquema completo + datos iniciales
cargo run                           # API en http://127.0.0.1:34187
```

Frontend:

```bash
cd frontend-react
npm install
npm run dev   # Vite con proxy /api -> http://127.0.0.1:34187
```

La URL del API se configura con `VITE_API_BASE` en `frontend-react` (vacía
por defecto: rutas `/api` relativas al propio dominio, con proxy de Vite en
desarrollo). Las credenciales del administrador y el resto de parámetros se
definen en `backend/.env` (ver `.env.example`).

## Pagos con Stripe (Checkout con redirección)

Stripe utiliza **Checkout alojado con redirección**: el cliente sale a Stripe,
paga allí y vuelve a la tienda. El pedido solo se marca como pagado cuando
Stripe confirma el cobro mediante un webhook firmado; la página de retorno
también verifica la Checkout Session como respaldo.

### 1. Aplicar las migraciones

```bash
cd backend
cargo run -- migrate        # payment_status, campos de transacción y log de webhooks
```

Si creas la base de datos desde cero, `migrate:fresh --seed` ya incluye estas
tablas y columnas.

### 2. Configurar las claves

La forma recomendada es mediante variables de entorno (no las subas al
repositorio):

```env
STRIPE_SECRET_KEY=sk_test_...
STRIPE_PUBLISHABLE_KEY=pk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
```

También se pueden guardar desde **Admin → Configuración → Métodos de pago →
Stripe**; las variables de entorno tienen prioridad sobre `core_config`.
Activa la pasarela y guarda la configuración. Stripe solo aparecerá en el
checkout cuando esté activo y exista una clave secreta.

### 3. Configurar el webhook

En el Dashboard de Stripe, entra en **Developers → Webhooks** y añade:

```text
https://tu-dominio.com/api/webhooks/stripe
```

Selecciona al menos estos eventos:

- `checkout.session.completed`
- `checkout.session.async_payment_succeeded`
- `checkout.session.async_payment_failed`
- `checkout.session.expired`
- `payment_intent.succeeded`
- `payment_intent.payment_failed`
- `payment_intent.amount_capturable_updated`
- `charge.dispute.created`
- `charge.dispute.closed`
- `charge.refund.updated`

Para desarrollo local, usa la CLI de Stripe:

```bash
stripe listen --forward-to localhost:34187/api/webhooks/stripe
# copia el valor whsec_... que muestra en STRIPE_WEBHOOK_SECRET
```

### 4. Probar el flujo

Usa la tarjeta de prueba `4242 4242 4242 4242`, cualquier fecha futura y
cualquier CVC.

1. El pedido se crea como `pending` / `pending_payment`.
2. El navegador se redirige a Stripe Checkout.
3. Cuando el pago se completa, el webhook registra la transacción, marca el
   pedido como `paid` y emite la factura automáticamente.
4. Si el cliente cancela o la sesión expira (~31 minutos), el pedido se
   cancela, se libera el stock y se restaura el carrito.

### Reembolsos, captura manual y disputas

- **Reembolsos**: usa la acción Refund habitual del pedido. EzCommerce llama
  primero a `POST /v1/refunds` y solo después registra el reembolso local,
  guardando el id de Stripe (`re_...`) y el estado real de la pasarela. Se
  admiten reembolsos parciales. Si Stripe rechaza el reembolso, no se crea
  ningún reembolso local. Los reembolsos pendientes se actualizan cuando
  Stripe envía `charge.refund.updated`.
- **Captura manual**: selecciona **Capture method → Manual (authorize only)**
  en la configuración de Stripe. El pedido queda en `authorized` tras el
  checkout y la vista del pedido muestra la acción **Capture payment**. La
  factura se emite al capturar el pago.
- **Disputas/chargebacks**: `charge.dispute.created` y
  `charge.dispute.closed` actualizan el estado del pago
  (`disputed` / `paid` si se gana / `refunded` si se pierde), guardan id,
  motivo y estado de la disputa, y añaden una entrada al historial de
  transacciones del pedido.
- **Recibo itemizado**: las Checkout Sessions se crean con una línea por
  producto más impuestos y envío; los descuentos del pedido se aplican como
  cupón de un solo uso de Stripe. Si los importes no cuadran exactamente,
  EzCommerce usa una sola línea con el total del pedido para garantizar que el
  cobro coincide.

### Comportamiento y seguridad

- Las firmas de los webhooks se verifican (HMAC-SHA256, tolerancia de 5
  minutos) y los eventos se deduplican por id.
- Los pedidos Stripe impagados o solo autorizados no se pueden enviar ni
  reembolsar.
- Los pedidos Stripe pagados deben reembolsarse en lugar de cancelarse
  directamente; la acción Cancelar permanece oculta mientras el cobro esté
  capturado.
- La vuelta a la página de éxito verifica la Checkout Session, así que el flujo
  también funciona en local cuando el webhook todavía no ha llegado.

## Despliegue con Docker Compose

```bash
JWT_SECRET=<secreto> DB_PASSWORD=<contraseña> docker compose up -d
```

Servicios:

- `api` — binario Rust; ejecuta migraciones y seeders antes de servir
  (healthcheck en `/up`, volumen `api_storage` para las imágenes subidas)
- `db` — MariaDB 11 con volumen persistente
- `web` — nginx que sirve el SPA React compilado y hace proxy de `/api` y
  `/storage` hacia el API (mismo origen, sin CORS)

Variables principales (ver `docker-compose.yml`): `JWT_SECRET`, `DB_*`,
`ADMIN_EMAIL`, `ADMIN_PASSWORD`, `SEED_DEMO_DATA`, `FRONTEND_URL` y,
opcionalmente, `STRIPE_SECRET_KEY`, `STRIPE_PUBLISHABLE_KEY` y
`STRIPE_WEBHOOK_SECRET`.

## Despliegue en Coolify

1. Crear el proyecto desde el repositorio Git eligiendo **Docker Compose**
   (`docker-compose.yml` en la raíz)
2. Definir las variables obligatorias (`JWT_SECRET`, `DB_PASSWORD`,
   `DB_ROOT_PASSWORD`, `ADMIN_EMAIL`, `ADMIN_PASSWORD`) y las recomendadas
   (`APP_URL`, `FRONTEND_URL` para CORS, `SEED_DEMO_DATA=false`). Para Stripe,
   añadir `STRIPE_SECRET_KEY`, `STRIPE_PUBLISHABLE_KEY` y
   `STRIPE_WEBHOOK_SECRET`
3. Asignar dominios en Coolify: servicio `web` → dominio público de la tienda
   (el nginx del servicio hace proxy de `/api` y `/storage` hacia el API, sin
   CORS); el servicio `api` no necesita dominio público (healthcheck `/up`)
4. Montar volúmenes persistentes: `db_data` (base de datos) y `api_storage`
   (imágenes de producto)
5. Desplegar: el API migra y siembra automáticamente en el primer arranque

Los servicios usan `expose` en lugar de `ports` publicados: Coolify gestiona
el acceso mediante sus dominios y proxy. Si se quiere acceso directo por
puerto (por ejemplo para pruebas sin dominio), añadir en la UI del servicio el
mapeo deseado o sustituir `expose` por `ports` en el compose
(`"8080:80"` para la tienda, `"34187:34187"` para el API).

El SPA se construye con `API_BASE_URL=SAME_ORIGIN` (llamadas `/api` relativas
al propio dominio). Si el API se publica en otro dominio, cambiar ese build-arg.

## CI

GitHub Actions (`.github/workflows/ci.yml`) valida en cada push:

- Backend: `cargo check` + migraciones y seeders sobre SQLite
- Frontend: `npm install` + `npm run build` (TypeScript + Vite)
- Construcción de las imágenes Docker (API y web)

## API

API JSON para tienda (`/api/...`) y administración (`/api/admin/...`) con
autenticación JWT por cabecera `Authorization: Bearer`. Los errores siguen el
formato Laravel (`{message}` y validación 422 con `{message, errors}`); las
colecciones paginadas usan la forma estándar `{data, links, meta}`.

### Documentación del API y MCP

- Swagger UI: `http://<host>/api/docs` (pública). `/docs` redirige allí.
- OpenAPI 3.0 JSON: `http://<host>/api/openapi.json`.
- Servidor MCP: `POST /api/mcp`, JSON-RPC 2.0 sobre HTTP, autenticado con el
  mismo JWT de administración que el API (`Authorization: Bearer <token>`).
  Soporta `initialize`, `tools/list` y `tools/call`; aparece documentado en el
  OpenAPI con sus herramientas y el esquema de seguridad JWT.

Herramientas MCP (los esquemas completos se devuelven en `tools/list` y se
muestran en la extensión `x-mcp-tools` del OpenAPI):

- Catálogo: `list_products`, `get_product`, `list_categories`, `get_category`,
  `list_attributes`, `list_attribute_families`, `get_product_stock`,
  `low_stock_products`, `create_product`, `update_product`, `delete_product`,
  `set_product_stock`, `create_category`, `update_category`, `delete_category`.
- Ventas: `list_orders`, `get_order`, `create_invoice`, `cancel_order`,
  `list_invoices`, `get_invoice`, `list_shipments`, `get_shipment`,
  `create_shipment`, `create_refund`, `add_order_comment`,
  `list_transactions`, `list_refunds`, `sales_summary`.
- Clientes: `list_customers`, `get_customer`, `create_customer`,
  `update_customer`.
- Marketing y contenido: `list_cart_rules`, `get_cart_rule`,
  `list_catalog_rules`, `get_catalog_rule`, `list_cms_pages`, `get_cms_page`,
  `list_reviews`, `create_cms_page`, `update_cms_page`, `delete_cms_page`.
- Configuración: `list_channels`, `list_locales`, `list_currencies`,
  `list_tax_rates`, `list_inventory_sources`, `save_configuration`.

```bash
curl -X POST http://<host>/api/mcp \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

## Licencia

Uso interno. Consulta con el propietario del repositorio antes de reutilizar.

---

# EzCommerce

フルスタックのeコマースプラットフォーム：**Rust APIバックエンド**（Larustフレームワーク）と**Reactウェブフロントエンド（Mantine）**で構築された、レスポンシブなオンラインストアと管理パネル。

Docker（Docker Compose / Coolify）でデプロイ可能で、本番環境では**MySQL/MariaDB**、開発環境ではSQLiteを使用します。

## 機能

### ストアフロント（React、レスポンシブウェブ）

- セクション（画像カルーセル、商品・カテゴリ、コンテンツバナー）で構成可能なホーム画面
- 検索、属性・価格フィルター、ソート機能付きカタログ
- ギャラリー、カラーバッチ付き設定可能なバリエーション、グループ化商品を含む商品詳細ページ
- ゲストカート（署名付きカートトークン）と登録顧客カート、クーポン、送料見積もり
- ワンページチェックアウト：住所、配送方法、支払い方法、確認・完了
- Stripe Checkout（ホスト型リダイレクト）によるカード・ウォレット決済
- 顧客アカウント：プロフィール、注文（詳細、キャンセル、再注文）、住所、ウィッシュリスト、商品レビュー
- 完全な認証機能：登録、ログイン、パスワードリセット
- ヘッダーの言語とチャネル通貨セレクター

### 管理パネル（React）

- 売上メトリクス、注文、顧客、過去30日間のグラフを含むダッシュボード
- 検索、フィルター、ソート、ページネーション、一括アクション、CSVエクスポート機能付き汎用DataGrid
- 商品：タイプ別作成（シンプル、バーチャル、設定可能、グループ化、バンドル、ダウンロード可能）、日付範囲による特別価格、リンク商品、バリエーション、倉庫別在庫、画像、商品コピーを含むタブ付きエディター
- カテゴリ（階層ツリー）、ファミリーとグループ付き属性
- 注文の完全なライフサイクル：請求書、配送（マルチ倉庫）、返金・キャンセル、数量と在庫引下げに基づく派生ステータス
- Stripe設定、支払い状況、カード情報、キャプチャ、返金、異議申し立て、注文ごとの取引履歴
- 顧客：注文、住所、レビュー、ウィッシュリストを含むプロフィール；顧客グループとレビューモデレーション
- マーケティング：クーポン（パーセンテージ、固定額、X個購入でY個プレゼント、比例配分）付きカートルールと価格再インデックス付きカタログルール
- コミュニケーション：メールテンプレート、イベント、購読者への送信キャンペーン
- 検索SEO：URL書き換え、検索用語とシノニム
- CMS、ローカライゼーション（チャネル、通貨、言語）、ACL付きユーザーとロール
- 日付範囲別レポート、CSVエクスポート

### バックエンドAPI（Rust）

- JWT（Laravelスタイルのクレーム、トークンブラックリスト、リフレッシュ）によるストアと管理用JSON API
- 合計計算、ゾーン別税金、スケジュール送料と無料送料、在庫予約を含むカートエンジン
- 署名付きWebhook、手動キャプチャ、返金、異議申し立て、取引ログ、自動請求書発行を備えたStripe Checkoutセッション
- Bagistoスタイルの物化された価格（`product_flat`、価格・在庫インデックス）
- 言語別ローカライゼーションと為替レート付き通貨変換
- ブッキングタイプ商品のスロット別予約
- SQLite/MySQLの二重ダイアレクト対応SQLマイグレーションと冪等なシーダー
- OpenAPI 3.0ドキュメント、Swagger UI、JWTで保護されたMCPツールサーバー
- `/up`でのサービスヘルスチェックと静的配信付き画像ストレージ

## リポジトリ構造

```
backend/         Rust API（Larust）：コントローラー、カート/セールスエンジン、
                 SQLマイグレーションとシーダー
frontend-react/  Reactウェブアプリ（Mantine）：ストアと管理パネル
```

## 要件

- Stable Rust（バックエンド）
- Node 20+（フロントエンド）
- MariaDB/MySQL 10.4+（本番環境）；SQLite（ローカル開発）

## デモデータ

`SEED_DEMO_DATA=true`を設定すると、プラットフォームのテスト用完全パッケージが生成されます：11個のカタログ商品（3色のバリエーションを含む設定可能商品）、ホームセクション、クーポン（10%自動適用と`WELCOME10`）、カタログルール、顧客、請求書と配送付き注文（実際の派生ステータス：保留中、処理中、完了）、承認済みレビュー、ウィッシュリスト。シーダーは冪等であり、再実行してもデータが重複しません。

デモ認証情報：

| ロール | 識別子 | パスワード |
|---|---|---|
| 管理者 | `ADMIN_EMAIL`の値（デフォルト `admin@example.com`） | `ADMIN_PASSWORD`の値（デフォルト `admin123`） |
| 顧客 | `customer@example.com` | `customer123` |
| 顧客2 | `marc@example.com` | `customer123` |
| 顧客3 | `laura@example.com` | `customer123` |

## スタートガイド（開発）

バックエンド：

```bash
cd backend
cp .env.example .env
cargo run -- migrate:fresh --seed   # 完全なスキーマ + 初期データ
cargo run                           # APIが http://127.0.0.1:34187 で起動
```

フロントエンド：

```bash
cd frontend-react
npm install
npm run dev   # Vite、プロキシ /api -> http://127.0.0.1:34187
```

API URLは`frontend-react`の`VITE_API_BASE`で設定します（デフォルトは空：同じドメインの相対`/api`ルート、開発中はViteプロキシ）。管理者認証情報とその他のパラメータは`backend/.env`で定義します（`.env.example`を参照）。

## Stripe決済（リダイレクトCheckout）

Stripeは**ホスト型Checkoutのリダイレクト**方式で動作します。顧客はStripeへ移動して支払い、ストアに戻ります。注文は、署名付きWebhookでStripeが支払いを確認した時点でのみ支払い済みになります（戻りページでもCheckout Sessionを検証します）。

### 1. マイグレーションの適用

```bash
cd backend
cargo run -- migrate        # payment_status、取引項目、Webhookログを追加
```

データベースを新規作成する場合、`migrate:fresh --seed`にもこれらのテーブルとカラムが含まれます。

### 2. キーの設定

推奨される方法は環境変数です（リポジトリにコミットしないでください）：

```env
STRIPE_SECRET_KEY=sk_test_...
STRIPE_PUBLISHABLE_KEY=pk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
```

**管理パネル → 設定 → 支払い方法 → Stripe**からも保存できます。環境変数は`core_config`より優先されます。ゲートウェイを有効にして設定を保存してください。Stripeは有効かつシークレットキーが存在する場合にのみチェックアウトに表示されます。

### 3. Webhookの設定

Stripeダッシュボードの**Developers → Webhooks**で以下を追加します：

```text
https://your-domain.com/api/webhooks/stripe
```

少なくとも次のイベントを選択してください：

- `checkout.session.completed`
- `checkout.session.async_payment_succeeded`
- `checkout.session.async_payment_failed`
- `checkout.session.expired`
- `payment_intent.succeeded`
- `payment_intent.payment_failed`
- `payment_intent.amount_capturable_updated`
- `charge.dispute.created`
- `charge.dispute.closed`
- `charge.refund.updated`

ローカル開発ではStripe CLIを使用します：

```bash
stripe listen --forward-to localhost:34187/api/webhooks/stripe
# 表示されたwhsec_...をSTRIPE_WEBHOOK_SECRETに設定
```

### 4. フローのテスト

テストカード`4242 4242 4242 4242`、任意の将来の有効期限、任意のCVCを使用します。

1. 注文は`pending` / `pending_payment`で作成されます。
2. ブラウザはStripe Checkoutへリダイレクトされます。
3. 支払いが完了すると、Webhookが取引を記録し、注文を`paid`にして請求書を自動発行します。
4. 顧客がキャンセルした場合、またはセッションが期限切れ（約31分）になった場合、注文はキャンセルされ、在庫が解放され、カートが復元されます。

### 返金、手動キャプチャ、異議申し立て

- **返金**：注文の通常のRefundアクションを使用します。EzCommerceはまず`POST /v1/refunds`を呼び、その後でローカル返金を記録し、Stripe ID（`re_...`）と実際のステータスを保存します。部分返金にも対応しています。Stripeが返金を拒否した場合、ローカル返金は作成されません。保留中の返金は、Stripeが`charge.refund.updated`を送信したときに更新されます。
- **手動キャプチャ**：Stripe設定で**Capture method → Manual (authorize only)**を選択します。チェックアウト後、注文は`authorized`のままになり、注文画面に**Capture payment**アクションが表示されます。請求書はキャプチャ時に発行されます。
- **異議申し立て/チャージバック**：`charge.dispute.created`と`charge.dispute.closed`が支払いステータス（`disputed` / 勝訴時は`paid` / 敗訴時は`refunded`）を更新し、異議ID、理由、ステータスを保存して、注文の取引履歴にエントリを追加します。
- **明細付き領収書**：Checkout Sessionは商品ごとに1行、さらに税金と送料で作成されます。注文の割引は1回限りのStripeクーポンとして適用されます。金額が正確に一致しない場合は、請求額が注文と一致するよう、注文合計の1行にフォールバックします。

### 動作とセキュリティ

- Webhook署名は検証され（HMAC-SHA256、5分の許容範囲）、イベントはIDで重複排除されます。
- 未払いまたは承認のみのStripe注文は配送も返金もできません。
- 支払い済みのStripe注文は直接キャンセルせず返金する必要があります。キャプチャ済みの間はキャンセルアクションは非表示です。
- 成功ページへの戻り時にCheckout Sessionを検証するため、Webhookがまだ届いていないローカル環境でもフローが動作します。

## Docker Composeでのデプロイ

```bash
JWT_SECRET=<秘密> DB_PASSWORD=<パスワード> docker compose up -d
```

サービス：

- `api` — Rustバイナリ；サービング前にマイグレーションとシーダーを実行
  （`/up`でヘルスチェック、アップロード画像用ボリューム`api_storage`）
- `db` — 永続ボリューム付きMariaDB 11
- `web` — コンパイル済みReact SPAを配信し、`/api`と`/storage`をAPIにプロキシするnginx
  （同じオリジン、CORS不要）

主要変数（`docker-compose.yml`を参照）：`JWT_SECRET`、`DB_*`、
`ADMIN_EMAIL`、`ADMIN_PASSWORD`、`SEED_DEMO_DATA`、`FRONTEND_URL`、
および任意で`STRIPE_SECRET_KEY`、`STRIPE_PUBLISHABLE_KEY`、
`STRIPE_WEBHOOK_SECRET`。

## Coolifyでのデプロイ

1. Gitリポジトリからプロジェクトを作成し、**Docker Compose**を選択
   （ルートに`docker-compose.yml`）
2. 必須変数（`JWT_SECRET`、`DB_PASSWORD`、`DB_ROOT_PASSWORD`、
   `ADMIN_EMAIL`、`ADMIN_PASSWORD`）と推奨変数（`APP_URL`、CORS用の
   `FRONTEND_URL`、`SEED_DEMO_DATA=false`）を定義。Stripeを使う場合は
   `STRIPE_SECRET_KEY`、`STRIPE_PUBLISHABLE_KEY`、
   `STRIPE_WEBHOOK_SECRET`も追加
3. Coolifyでドメインを割り当て：`web`サービス → ストアの公開ドメイン
   （nginxサービスが`/api`と`/storage`をAPIにプロキシ、CORS不要）；
   `api`サービスは公開ドメイン不要（ヘルスチェック`/up`）
4. 永続ボリュームをマウント：`db_data`（データベース）と`api_storage`
   （商品画像）
5. デプロイ：APIは初回起動時に自動的にマイグレーションとシーディングを実行

サービスは公開`ports`の代わりに`expose`を使用します：Coolifyはドメインと
プロキシ経由でアクセスを管理します。ポート直接アクセスが必要な場合（ドメイン
なしのテストなど）、サービスUIでマッピングを追加するか、composeで`expose`を
`ports`に置き換えます（ストアは`"8080:80"`、APIは`"34187:34187"`）。

SPAは`API_BASE_URL=SAME_ORIGIN`でビルドされます（同じドメインの相対`/api`
呼び出し）。APIを別のドメインで公開する場合は、そのビルド引数を変更します。

## CI

GitHub Actions（`.github/workflows/ci.yml`）は各プッシュ時に検証します：

- バックエンド：`cargo check` + SQLite上のマイグレーションとシーダー
- フロントエンド：`npm install` + `npm run build`（TypeScript + Vite）
- Dockerイメージのビルド（APIとウェブ）

## API

`Authorization: Bearer`ヘッダーによるJWT認証付き、ストア用（`/api/...`）と
管理用（`/api/admin/...`）のJSON API。エラーはLaravel形式（`{message}`と
422バリデーションの`{message, errors}`）に従います；ページネーション付き
コレクションは標準形式`{data, links, meta}`を使用します。

### APIドキュメントとMCP

- Swagger UI：`http://<host>/api/docs`（公開）。`/docs`はそこへリダイレクトします。
- OpenAPI 3.0 JSON：`http://<host>/api/openapi.json`。
- MCPサーバー：`POST /api/mcp`、HTTP上のJSON-RPC 2.0。管理APIと同じ管理者JWTで認証します（`Authorization: Bearer <token>`）。`initialize`、`tools/list`、`tools/call`に対応し、OpenAPIドキュメントにツールとJWTセキュリティスキームが記載されます。

MCPツール（完全なスキーマは`tools/list`とOpenAPIの`x-mcp-tools`で確認できます）：

- カタログ：`list_products`、`get_product`、`list_categories`、`get_category`、
  `list_attributes`、`list_attribute_families`、`get_product_stock`、
  `low_stock_products`、`create_product`、`update_product`、`delete_product`、
  `set_product_stock`、`create_category`、`update_category`、`delete_category`。
- 販売：`list_orders`、`get_order`、`create_invoice`、`cancel_order`、
  `list_invoices`、`get_invoice`、`list_shipments`、`get_shipment`、
  `create_shipment`、`create_refund`、`add_order_comment`、
  `list_transactions`、`list_refunds`、`sales_summary`。
- 顧客：`list_customers`、`get_customer`、`create_customer`、
  `update_customer`。
- マーケティングとコンテンツ：`list_cart_rules`、`get_cart_rule`、
  `list_catalog_rules`、`get_catalog_rule`、`list_cms_pages`、`get_cms_page`、
  `list_reviews`、`create_cms_page`、`update_cms_page`、`delete_cms_page`。
- ストア設定：`list_channels`、`list_locales`、`list_currencies`、
  `list_tax_rates`、`list_inventory_sources`、`save_configuration`。

```bash
curl -X POST http://<host>/api/mcp \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

## ライセンス

内部使用。再利用する前にリポジトリ所有者にお問い合わせください。
