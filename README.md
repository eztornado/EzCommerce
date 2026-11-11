# EzCommerce

Port de [Bagisto](https://github.com/bagisto/bagisto) (bagisto-base) a un stack propio:
**backend Rust con el framework Larust** y **frontend Flutter** (web como plataforma principal,
android/ios para móvil). Mantengo **MySQL/MariaDB** como base de datos de producción y SQLite
para desarrollo local.

El alcance, decisiones de arquitectura y estado por módulo se gestiona en GitHub Issues
(épica #1 y sus issues hijos #2–#15).

## Estructura

```
backend/    API Rust (Larust 0.10): routes/api.rs, app/Http/Controllers, app/Support,
            database/migrations (esquema Bagisto completo en .sql + shim MySQL)
frontend/   App Flutter (web + android + ios): shop + panel admin
```

## Puesta en marcha (desarrollo local)

Backend (SQLite por defecto):

```bash
cd backend
cp .env.example .env
cargo run -- migrate:fresh --seed   # esquema completo + datos (admin, catálogo demo)
cargo run                            # API en http://127.0.0.1:34187
```

Producción MySQL/MariaDB: configurar `DB_CONNECTION=mysql` + `DB_HOST/PORT/DATABASE/USERNAME/PASSWORD`
en `backend/.env` y ejecutar `cargo run -- migrate` (el runner reescribe el dialecto SQL
automáticamente).

Frontend (Flutter web):

```bash
cd frontend
flutter pub get
flutter run -d chrome            # o: flutter build web
```

La URL del API se configura en `frontend/lib/core/api/api_client.dart` (`ApiClient.baseUrl`).

## Credenciales por defecto (seeders)

- Admin: `ADMIN_EMAIL` / `ADMIN_PASSWORD` del `.env` (por defecto `admin@example.com` / `admin123`).
- Panel admin Flutter: `/#/admin/login` → área `/admin`.

## Autenticación

JWT (`Authorization: Bearer`) con dos principales: `admin` (tabla `admins`) y `customer`
(tabla `customers`). Los carritos de invitado usan el header `X-Cart-Token` (token firmado
de carrito), sustituyendo las sesiones de cookie de Bagisto.

## Contrato de la API

- Shop: `/api/...` con la forma de Bagisto (`{data, message}`, paginado `{data, links, meta}`).
- Admin: `/api/admin/...` con el contrato DataGrid (`{columns, actions, mass_actions, records, meta}`).
- Errores: `{message}` y validación 422 `{message, errors}`.

### Módulos implementados

| Módulo | Endpoints clave |
|---|---|
| Core (canal/locale/moneda) | `/api/core/context`, `/countries`, `/states`, `/currencies`, `/home` |
| Catálogo | `/api/products`, `/api/products/{id}`, `/api/categories/tree`, `/api/categories/attributes`, reseñas |
| Clientes | `/api/customer/{login,register,get,profile}`, direcciones, wishlist, compare, pedidos |
| Carrito + checkout | `/api/checkout/cart`, `/api/checkout/onepage/{addresses,shipping-methods,payment-methods,orders}` |
| Ventas (admin) | pedidos/facturas/envíos/reembolsos con prorrateo, estados derivados y stock |
| Marketing | `/api/admin/marketing/{cart-rules,catalog-rules}` + cupones + generación masiva + reindex |
| CMS | `/api/cms/page/{url_key}` público, CRUD admin con traducciones |
| SEO | url rewrites, search terms/synonyms, `/api/seo/sitemap.xml` |
| Reporting | `/api/admin/reporting/{sales,products,customers}` con granularidad |
| Data transfer | `/api/admin/data-transfer/products/{export,import}` CSV |
| RMA / GDPR | cliente (crear/listar/solicitar/export datos) + admin (estados, anonimización) |
| Apariencia | CRUD de `theme_sections` con traducciones, reorder y estado |

## CI/CD

GitHub Actions (`.github/workflows/ci.yml`): backend (`cargo check` + migraciones/seed en
SQLite), frontend (`flutter analyze` + `build web`) y validación de las imágenes Docker.
`docker compose up -d` despliega API (migra+siembra al arrancar), MariaDB y nginx con el SPA.
