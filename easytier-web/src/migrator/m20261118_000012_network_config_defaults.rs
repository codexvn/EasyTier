use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE networks ADD COLUMN config_defaults TEXT NOT NULL DEFAULT '{\"values\":{},\"set_fields\":[],\"clear_fields\":[]}'; \
                 ALTER TABLE network_members ADD COLUMN override_patch TEXT; \
                 ALTER TABLE network_members ADD COLUMN override_inherit_fields TEXT NOT NULL DEFAULT '[]'",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE network_members DROP COLUMN override_inherit_fields; \
                 ALTER TABLE network_members DROP COLUMN override_patch; \
                 ALTER TABLE networks DROP COLUMN config_defaults",
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, Statement};

    #[tokio::test]
    async fn migration_keeps_existing_member_snapshot_and_adds_empty_defaults() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        db.execute_unprepared(
            "CREATE TABLE networks (id TEXT PRIMARY KEY); \
             CREATE TABLE network_members (id TEXT PRIMARY KEY, override_config TEXT); \
             INSERT INTO networks VALUES ('network'); \
             INSERT INTO network_members VALUES ('member', '{\"enable_udp_fec\":true}')",
        )
        .await
        .unwrap();
        let manager = SchemaManager::new(&db);
        Migration.up(&manager).await.unwrap();
        let network = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "SELECT config_defaults FROM networks WHERE id = 'network'".to_owned(),
            ))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            network.try_get::<String>("", "config_defaults").unwrap(),
            "{\"values\":{},\"set_fields\":[],\"clear_fields\":[]}"
        );
        let member = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "SELECT override_config, override_patch, override_inherit_fields FROM network_members WHERE id = 'member'".to_owned(),
            ))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            member.try_get::<String>("", "override_config").unwrap(),
            "{\"enable_udp_fec\":true}"
        );
        assert!(
            member
                .try_get::<Option<String>>("", "override_patch")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            member
                .try_get::<String>("", "override_inherit_fields")
                .unwrap(),
            "[]"
        );
        Migration.down(&manager).await.unwrap();
    }
}
