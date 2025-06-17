use sqlx_migrator::{Info, Migrate, Migrator, Plan};

struct CreateConfigConfigMigration;

sqlx_migrator::sqlite_migration!(
    CreateConfigConfigMigration,
    "config",
    "create_config",
    sqlx_migrator::vec_box![],
    sqlx_migrator::vec_box![(
        "CREATE TABLE config (
            key TEXT PRIMARY KEY,
            value BLOB
        )", //up
        "DROP TABLE config" //down
    )]
);

struct CreateConfigWalletMigration;

sqlx_migrator::sqlite_migration!(
    CreateConfigWalletMigration,
    "config",
    "create_wallets",
    sqlx_migrator::vec_box![],
    sqlx_migrator::vec_box![(
        "CREATE TABLE wallets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            secret_key BLOB NOT NULL,
            scan_config TEXT NOT NULL,
            address TEXT NOT NULL,
            balance TEXT NOT NULL
        )", //up
        "DROP TABLE config" //down
    )]
);

impl super::Config {
    pub async fn migrate_tables(&self) -> anyhow::Result<()> {
        let mut migrator = Migrator::default();
        // Adding migration can fail if another migration with same app and name and different values gets added
        // Adding migrations add its parents, replaces and not before as well
        migrator.add_migration(Box::new(CreateConfigConfigMigration))?;
        migrator.add_migration(Box::new(CreateConfigWalletMigration))?;

        let mut conn = self.db.acquire().await?;
        // use apply all to apply all pending migration
        migrator.run(&mut *conn, &Plan::apply_all()).await?;

        Ok(())
    }
}
