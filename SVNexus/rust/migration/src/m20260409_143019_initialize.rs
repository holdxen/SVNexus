use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum WorkspaceHistoryGroup {
    Table,
    Id,
    Uuid,
    Name,
    Children
}

#[derive(DeriveIden)]
enum WorkspaceHistory {
    Table,
    Id,
    Uuid,
    LastUsedTime,
    Star,
    Remark,
    RepositoryRootUrl,
    // working_copy
    WorkingCopyRoot,
    WorkingCopyPath,
    Checkout,

    // repository
    RepositoryUuid,

    Kind,
}

#[derive(DeriveIden)]
pub enum LogChangedPathEntry {
    Table,
    Id,
    Path,
    Action,
    CopyFromPath,
    CopyFromRevision,
    NodeKind,
    TextModified,
    PropertiesModified,
    LogEntryId,
}

impl LogChangedPathEntry {
    async fn create<'a>(manager: &SchemaManager<'a>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Self::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Self::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Self::Path).string().not_null())
                    .col(ColumnDef::new(Self::Action).string().not_null())
                    .col(ColumnDef::new(Self::CopyFromPath).string().null())
                    .col(ColumnDef::new(Self::CopyFromRevision).integer().null())
                    .col(ColumnDef::new(Self::NodeKind).string().not_null())
                    .col(ColumnDef::new(Self::TextModified).boolean().null())
                    .col(ColumnDef::new(Self::PropertiesModified).boolean().null())
                    .col(ColumnDef::new(Self::LogEntryId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("foreign_key_log_entry_id")
                            .from(Self::Table, Self::LogEntryId)
                            .to(LogEntry::Table, LogEntry::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Property {
    Table,
    Id,
    Name,
    Value,
    LogEntryId,
}

impl Property {
    async fn create<'a>(manager: &SchemaManager<'a>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Self::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Self::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Self::Name).string().not_null())
                    .col(ColumnDef::new(Self::Value).string().not_null())
                    .col(ColumnDef::new(Self::LogEntryId).integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("foreign_key_property_id")
                            .from(Self::Table, Self::LogEntryId)
                            .to(LogEntry::Table, LogEntry::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum LogEntry {
    Table,
    Id,
    RepositoryUuid,
    Revision,
    Date,
    Author,
    Message,
    HasChildren,
    NonInheritable,
    SubtractiveMerge,
}

impl LogEntry {
    async fn create<'a>(manager: &SchemaManager<'a>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Self::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Self::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Self::RepositoryUuid).string().not_null())
                    .col(ColumnDef::new(Self::Revision).unsigned().null())
                    .col(ColumnDef::new(Self::Date).integer().null())
                    .col(ColumnDef::new(Self::Author).string().null())
                    .col(ColumnDef::new(Self::Message).string().null())
                    .col(ColumnDef::new(Self::HasChildren).boolean().not_null())
                    .col(ColumnDef::new(Self::NonInheritable).boolean().not_null())
                    .col(ColumnDef::new(Self::SubtractiveMerge).boolean().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}


#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(WorkspaceHistoryGroup::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkspaceHistoryGroup::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WorkspaceHistoryGroup::Uuid)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(WorkspaceHistoryGroup::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(WorkspaceHistoryGroup::Children)
                            .string()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WorkspaceHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkspaceHistory::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WorkspaceHistory::Uuid).string().not_null())
                    .col(ColumnDef::new(WorkspaceHistory::LastUsedTime).integer().null())
                    .col(ColumnDef::new(WorkspaceHistory::Star).boolean().not_null())
                    .col(ColumnDef::new(WorkspaceHistory::Remark).string().null())
                    .col(ColumnDef::new(WorkspaceHistory::RepositoryRootUrl).string().null())
                    .col(ColumnDef::new(WorkspaceHistory::WorkingCopyPath).string().null())
                    .col(ColumnDef::new(WorkspaceHistory::WorkingCopyRoot).string().null())
                    .col(ColumnDef::new(WorkspaceHistory::Checkout).boolean().not_null())
                    .col(ColumnDef::new(WorkspaceHistory::RepositoryUuid).string().null())
                    .col(ColumnDef::new(WorkspaceHistory::Kind).string().not_null())
                    // .col(ColumnDef::new(WorkspaceHistory::HistoryGroupId).integer().null())
                    // .foreign_key(
                    //     ForeignKey::create()
                    //         .name("foreign_key_history_group_id")
                    //         .from(WorkspaceHistory::Table, WorkspaceHistory::HistoryGroupId)
                    //         .to(WorkspaceHistoryGroup::Table, WorkspaceHistoryGroup::Id)
                    //         .on_delete(ForeignKeyAction::SetNull)
                    //         .on_update(ForeignKeyAction::Cascade),
                    // )
                    .to_owned(),
            )
            .await?;

        LogEntry::create(manager).await?;
        Property::create(manager).await?;
        LogChangedPathEntry::create(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WorkspaceHistory::Table).to_owned())
            .await?;
        Ok(())
    }
}
