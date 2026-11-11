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

## Despliegue en Coolify

El repo está preparado para desplegarse dockerizado con **Docker Compose**:

1. Crear el proyecto en Coolify desde el repositorio Git eligiendo **Docker Compose**
   (`docker-compose.yml` en la raíz).
2. Definir en *Environment Variables* (a nivel de compose o del servicio):
   - Obligatorias: `JWT_SECRET`, `DB_PASSWORD`, `DB_ROOT_PASSWORD`, `ADMIN_EMAIL`, `ADMIN_PASSWORD`
   - Recomendadas: `APP_URL` y `FRONTEND_URL`/`FRONTEND_WWW_URL` (dominio real para CORS),
     `SEED_DEMO_DATA=false` en producción
3. Asignar dominios en Coolify:
   - Servicio `web` (nginx + SPA Flutter) → dominio público de la tienda; hace proxy de
     `/api` y `/storage` hacia `api:34187` (mismo origen, sin CORS)
   - Servicio `api` → dominio del API (si se expone aparte); healthcheck en `/up`
4. Volúmenes persistentes: `db_data` (MariaDB) y `api_storage` (`/app/public/storage`,
   imágenes de productos — montarlo como volumen en Coolify para no perderlas al recrear)
5. Desplegar: el contenedor `api` ejecuta `migrate` + `db:seed` antes de servir
   (`SEED_DEMO_DATA=false` por defecto en producción)

Notas:
- El SPA se construye con `API_BASE_URL=SAME_ORIGIN` (rutas `/api` relativas al propio
  dominio a través del proxy de nginx); si el API se sirve en otro dominio, cambiar el
  build-arg `API_BASE_URL` por su URL.
- Cambiar siempre `JWT_SECRET` (por defecto: clave de ejemplo) y las credenciales
  `ADMIN_*` en el primer arranque.
