use larust_http::Router;

#[tokio::main]
async fn main() -> Result<(), larust_core::AppError> {
    // Larust's `Application` loads `.env` for the HTTP server, but the
    // migrate/seed commands run before it is built. Load it up-front so
    // `cargo run -- migrate` honours `DB_CONNECTION` and the rest of `.env`.
    let _ = dotenvy::from_path(ezcommerce_api::paths().env());
    let command = std::env::args().nth(1);

    if command.as_deref() == Some("migrate") {
        ezcommerce_api::connect_database().await?;
        ezcommerce_api::database::migrate().await?;
        return Ok(());
    }

    if command.as_deref() == Some("migrate:fresh") {
        ezcommerce_api::connect_database().await?;
        ezcommerce_api::database::migrate_fresh().await?;
        if std::env::args().any(|arg| arg == "--seed") {
            ezcommerce_api::seeders::run().await?;
        }
        return Ok(());
    }

    if command.as_deref() == Some("db:seed") {
        ezcommerce_api::connect_database().await?;
        ezcommerce_api::seeders::run().await?;
        return Ok(());
    }

    if command.as_deref() == Some("queue:work") {
        ezcommerce_api::connect_database().await?;
        let registry = larust_support::queue::JobRegistry::new()
            .register::<larust_support::mail::MailJob>();
        return larust_support::queue::work(registry).await;
    }

    if command.as_deref() == Some("schedule:work") {
        ezcommerce_api::connect_database().await?;
        return larust_support::schedule::work(ezcommerce_api::routes::console::schedule()).await;
    }

    if command.as_deref() == Some("route:list") {
        let app = ezcommerce_api::application()?;
        let route = ezcommerce_api::router(&app);
        print_routes(&route);
        return Ok(());
    }

    if command.as_deref() == Some("command:list") {
        for info in ezcommerce_api::routes::console::commands().list() {
            println!("{:<24} {}", info.name, info.description);
        }
        return Ok(());
    }

    if let Some(name) = command.as_deref() {
        ezcommerce_api::connect_database().await?;
        let args: Vec<String> = std::env::args().skip(2).collect();
        if ezcommerce_api::routes::console::commands()
            .dispatch(name, &args)
            .await?
        {
            return Ok(());
        }
        eprintln!("error: unknown command {name:?} (see `command:list`)");
        std::process::exit(1);
    }

    ezcommerce_api::serve().await
}

fn print_routes(route: &Router) {
    for info in route.routes() {
        println!(
            "{:<7} {:<44} {}",
            info.method,
            info.path,
            info.name.as_deref().unwrap_or("")
        );
    }
}
