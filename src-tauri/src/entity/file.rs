//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, Eq)]
#[sea_orm(table_name = "file")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub pid: String,
    pub wid: String,
    pub size: i32,
    pub create_time: i64,
    pub update_time: i64,
    pub state: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DataTransModel {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub pid: String,
    pub parent_file_name: Option<String>,
    pub wid: String,
    pub workspace_name: String,
    pub size: i32,
    pub create_time: i64,
    pub update_time: i64,
    pub state: i32,
}