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
- Materialized prices (`product_flat`, price and inventory indexes) in the
  style of Bagisto
- Localization by language and currency conversion with exchange rates
- Bookings by slots for booking-type products
- SQL migrations with dual dialect (SQLite/MySQL) and idempotent seeders
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
`ADMIN_EMAIL`, `ADMIN_PASSWORD`, `SEED_DEMO_DATA`, `FRONTEND_URL`.

## Deployment on Coolify

1. Create the project from the Git repository choosing **Docker Compose**
   (`docker-compose.yml` at the root)
2. Define the required variables (`JWT_SECRET`, `DB_PASSWORD`,
   `DB_ROOT_PASSWORD`, `ADMIN_EMAIL`, `ADMIN_PASSWORD`) and the recommended
   ones (`APP_URL`, `FRONTEND_URL` for CORS, `SEED_DEMO_DATA=false`)
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
- Precios materializados (`product_flat`, índices de precio e inventario) al
  estilo de Bagisto
- Localización por idioma y conversión de moneda con tasas de cambio
- Reservas (bookings) por slots para productos de tipo booking
- Migraciones SQL con doble dialecto (SQLite/MySQL) y seeders idempotentes
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
`ADMIN_EMAIL`, `ADMIN_PASSWORD`, `SEED_DEMO_DATA`, `FRONTEND_URL`.

## Despliegue en Coolify

1. Crear el proyecto desde el repositorio Git eligiendo **Docker Compose**
   (`docker-compose.yml` en la raíz)
2. Definir las variables obligatorias (`JWT_SECRET`, `DB_PASSWORD`,
   `DB_ROOT_PASSWORD`, `ADMIN_EMAIL`, `ADMIN_PASSWORD`) y las recomendadas
   (`APP_URL`, `FRONTEND_URL` para CORS, `SEED_DEMO_DATA=false`)
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
- 顧客アカウント：プロフィール、注文（詳細、キャンセル、再注文）、住所、ウィッシュリスト、商品レビュー
- 完全な認証機能：登録、ログイン、パスワードリセット
- ヘッダーの言語とチャネル通貨セレクター

### 管理パネル（React）

- 売上メトリクス、注文、顧客、過去30日間のグラフを含むダッシュボード
- 検索、フィルター、ソート、ページネーション、一括アクション、CSVエクスポート機能付き汎用DataGrid
- 商品：タイプ別作成（シンプル、バーチャル、設定可能、グループ化、バンドル、ダウンロード可能）、日付範囲による特別価格、リンク商品、バリエーション、倉庫別在庫、画像、商品コピーを含むタブ付きエディター
- カテゴリ（階層ツリー）、ファミリーとグループ付き属性
- 注文の完全なライフサイクル：請求書、配送（マルチ倉庫）、返金・キャンセル、数量と在庫引下げに基づく派生ステータス
- 顧客：注文、住所、レビュー、ウィッシュリストを含むプロフィール；顧客グループとレビューモデレーション
- マーケティング：クーポン（パーセンテージ、固定額、X個購入でY個プレゼント、比例配分）付きカートルールと価格再インデックス付きカタログルール
- コミュニケーション：メールテンプレート、イベント、購読者への送信キャンペーン
- 検索SEO：URL書き換え、検索用語とシノニム
- CMS、ローカライゼーション（チャネル、通貨、言語）、ACL付きユーザーとロール
- 日付範囲別レポート、CSVエクスポート

### バックエンドAPI（Rust）

- JWT（Laravelスタイルのクレーム、トークンブラックリスト、リフレッシュ）によるストアと管理用JSON API
- 合計計算、ゾーン別税金、スケジュール送料と無料送料、在庫予約を含むカートエンジン
- Bagistoスタイルの物化された価格（`product_flat`、価格・在庫インデックス）
- 言語別ローカライゼーションと為替レート付き通貨変換
- ブッキングタイプ商品のスロット別予約
- SQLite/MySQLの二重ダイアレクト対応SQLマイグレーションと冪等なシーダー
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
`ADMIN_EMAIL`、`ADMIN_PASSWORD`、`SEED_DEMO_DATA`、`FRONTEND_URL`。

## Coolifyでのデプロイ

1. Gitリポジトリからプロジェクトを作成し、**Docker Compose**を選択
   （ルートに`docker-compose.yml`）
2. 必須変数（`JWT_SECRET`、`DB_PASSWORD`、`DB_ROOT_PASSWORD`、
   `ADMIN_EMAIL`、`ADMIN_PASSWORD`）と推奨変数（`APP_URL`、CORS用の
   `FRONTEND_URL`、`SEED_DEMO_DATA=false`）を定義
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

## ライセンス

内部使用。再利用する前にリポジトリ所有者にお問い合わせください。
