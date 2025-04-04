// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::route::route_cmd;
use crate::stream::stream_cmd;
use anyhow::anyhow;
use app::api::{file, Api};
use app::dao::file_dao::FileService;
use app::dao::workspace_dao::WorkspaceService;
use app::entity::workspace::Model;
use app::service::user_service::{
    create, get_access_codes, get_user_info, get_user_info_by_name, login, logout, refresh_token,
    register, LoginBody, RegisterBody, UserInfo,
};
use app::service::workspace_service::create_workspace;
use app::service::{file_service, user_service, workspace_service};
use app::util::db_util::init_connection;
use app::{
    entity, AppResponse, AppState, Config, FileEntry, FileRequest, CONFIG_PATH, DATA_DB_NAME,
    DATA_PATH, DEFAULT_WORKSPACE, DIR_TYPE, FILE_PATH, FILE_TYPE, RESPONSE_CODE_ERROR,
    RESPONSE_CODE_SUCCESS, ROOT_PATH, WORKSPACE_PATH,
};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use clap::Parser;
use config::FileFormat;
use futures::future::err;
use log::{error, info};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, ExecResult, Schema};
use serde_json::{to_value, Value};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};
use std::time::SystemTime;
use std::{env, fs};
use tauri::api::http::{ClientBuilder, HttpRequestBuilder, ResponseType};
use tauri::api::path::home_dir;
use tauri::State;

mod route;

mod stream;

static DEFAULT_CONFIG: &str = include_str!("../config.toml");

#[derive(Parser)]
#[command(version)]
#[command(name = "fb")]
#[command(about = "FatherBox .")]
#[command(author = "blackstar-baba <535650957@qq.com>")]
struct Args {
    /// path to config file
    #[arg(short, long)]
    config: Option<String>,
    /// log level (v: info, vv: debug, vvv: trace)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}

fn banner() {
    const B: &str = r"
        FatherBox
    ";
    println!("{B}\n");
}

#[tauri::command]
fn my_custom_command() -> String {
    println!("I was invoked from JS!");
    return String::from("Hello, world!");
}

#[tokio::main]
async fn main() {
    // show banner
    banner();

    // process args
    let args: Args = Args::parse();

    let level = match args.verbose {
        0 => "info",
        1 => "debug",
        2 => "trace",
        _ => "",
    };

    // init tracing
    tracing_subscriber::fmt()
        .pretty()
        .with_line_number(false)
        .with_file(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_env_filter(level)
        .init();

    info!("Tracing level is {}", level);

    // init default path
    if home_dir().is_none() {
        error!("Home directory not found.");
        exit(1)
    }
    let home = home_dir().unwrap();
    info!("Home directory path: {}", home.display());
    let root_path = &home.join(ROOT_PATH);
    if !root_path.exists() {
        info!("Create root path: {}", root_path.display());
        fs::create_dir(root_path.as_path()).unwrap();
    }
    let config_path = &root_path.join(CONFIG_PATH);
    if !config_path.exists() {
        info!("Create {} path: {}", CONFIG_PATH, config_path.display());
        fs::create_dir(config_path).unwrap();
    }
    let data_path = &root_path.join(DATA_PATH);
    if !data_path.exists() {
        info!("Create {} path: {}", DATA_PATH, data_path.display());
        fs::create_dir(data_path).unwrap();
    }
    let file_path = &root_path.join(FILE_PATH);
    if !file_path.exists() {
        info!("Create {} path: {}", FILE_PATH, file_path.display());
        fs::create_dir(file_path).unwrap();
    }
    // process config
    let mut config_builder = config::Config::builder();
    config_builder = match &args.config {
        Some(config) => config_builder.add_source(config::File::with_name(config)),
        None => {
            info!("System use build-in config");
            config_builder.add_source(config::File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
        }
    };
    let config: Config = config_builder.build().unwrap().try_deserialize().unwrap();
    // init user db
    let db_result = init_data_db(data_path).await;
    if db_result.is_err() {
        exit(1);
    }
    let db = db_result.unwrap().unwrap();
    // init default user
    let user_id_result = init_default_user(&db).await;
    if user_id_result.is_err() {
        error!(
            "Init default user failed, err: {}",
            user_id_result.err().unwrap()
        );
        exit(1);
    }
    let user_id = &user_id_result.unwrap();
    // init workspace dir
    // e.g. .fatherbox/files/xxx-xxxxxxxx-xxxx-xxxxxxxx
    let user_file_path = &file_path.join(user_id);
    if !user_file_path.exists() {
        // create workspace
        let create_user_file_dir_result = fs::create_dir_all(user_file_path);
        if create_user_file_dir_result.is_err() {
            error!(
                "Create user file dir failed, err: {}",
                create_user_file_dir_result.err().unwrap()
            );
            exit(1)
        }
        info!(
            "Create user file dir success, path: {}",
            user_file_path.display()
        );
    }
    // init default workspace
    let db_result = init_default_workspace(&db, &user_id, user_file_path).await;
    if db_result.is_err() {
        error!("Init file db failed, err: {}", db_result.err().unwrap());
        exit(1);
    }

    tauri::Builder::default()
        .manage(AppState {
            conn: db,
            root_path: root_path.to_owned(),
            user_path: user_file_path.to_owned(),
        })
        // why sync fn must after sync fc
        .invoke_handler(tauri::generate_handler![route_cmd, my_custom_command, stream_cmd])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn init_default_user(db: &DatabaseConnection) -> Result<String, anyhow::Error> {
    let default_username = "default";
    let default_user_password = "123456";
    let default_nickname = "default user";

    let get_response = get_user_info_by_name(db, default_username, "local").await;
    if !get_response.is_success() {
        error!("get default user error: {}", get_response.message);
        return Err(anyhow!("get default user error: {}", get_response.message));
    }
    let option_user = get_response.result;
    match option_user {
        None => {
            let create_response = create(
                &db,
                &RegisterBody {
                    username: default_username.to_string(),
                    password: default_user_password.to_string(),
                    nickname: default_nickname.to_string(),
                },
            )
            .await;
            if !create_response.is_success() {
                error!("create default user error: {}", create_response.message);
                return Err(anyhow!(
                    "create default user error: {}",
                    create_response.message
                ));
            }
            Ok(create_response.result.unwrap().id)
        }
        Some(user) => return Ok(user.id),
    }
}

async fn init_data_db(data_path: &PathBuf) -> Result<Option<DatabaseConnection>, DbErr> {
    // e.g. ~/.fatherbox/data.db
    let db_file_path = &data_path.join(DATA_DB_NAME);
    info!("begin init data db use file {:?}", db_file_path);
    let db_exist = db_file_path.exists();
    let db = match init_connection(&db_file_path).await {
        Ok(conn) => conn,
        Err(err) => {
            info!("init connection catch err: {:?}", err);
            return Err(err);
        }
    };
    if !db_exist {
        info!("begin init tables in data db");
        let builder = db.get_database_backend();
        let schema = Schema::new(builder);
        match db
            .execute(builder.build(&schema.create_table_from_entity(entity::prelude::User)))
            .await
        {
            Ok(_) => {}
            Err(err) => {
                info!("init data.db catch err: {:?}", err);
                return Err(err);
            }
        }
        match db
            .execute(builder.build(&schema.create_table_from_entity(entity::prelude::Workspace)))
            .await
        {
            Ok(_) => {}
            Err(err) => {
                info!("init data.db catch err: {:?}", err);
                return Err(err);
            }
        }
        match db
            .execute(builder.build(&schema.create_table_from_entity(entity::prelude::File)))
            .await
        {
            Ok(_) => {}
            Err(err) => {
                info!("init data.db catch err: {:?}", err);
                return Err(err);
            }
        }
        match db
            .execute(builder.build(&schema.create_table_from_entity(entity::prelude::Setting)))
            .await
        {
            Ok(_) => {}
            Err(err) => {
                info!("init data.db catch err: {:?}", err);
                return Err(err);
            }
        }
        match db
            .execute(builder.build(&schema.create_table_from_entity(entity::prelude::AiSource)))
            .await
        {
            Ok(_) => {}
            Err(err) => {
                info!("init data.db catch err: {:?}", err);
                return Err(err);
            }
        }
        match db
            .execute(builder.build(&schema.create_table_from_entity(entity::prelude::AiModel)))
            .await
        {
            Ok(_) => {}
            Err(err) => {
                info!("init data.db catch err: {:?}", err);
                return Err(err);
            }
        }
    }
    Ok(Some(db))
}

async fn init_default_workspace(
    db: &DatabaseConnection,
    uid: &str,
    workspace_path: &PathBuf,
) -> Result<Option<Model>, anyhow::Error> {
    let get_response = workspace_service::get_workspace_by_name(db, DEFAULT_WORKSPACE).await;
    if !get_response.is_success() {
        return Err(anyhow!("{}", get_response.message));
    }
    let option_model = get_response.result;
    let mut id = "".to_string();
    if option_model.is_none() {
        // create default workspace
        let create_response = create_workspace(db, uid, DEFAULT_WORKSPACE).await;
        if !create_response.is_success() {
            error!("{}", create_response.message);
            return Err(anyhow!("{}", create_response.message));
        }
        id = create_response.result.id.clone()
    } else {
        id = option_model.unwrap().id
    }
    // create default workspace root dir
    let default_workspace_path = &workspace_path.join(&id);
    if !default_workspace_path.exists() {
        // create workspace
        info!(
            "Create default workspace dir: {}",
            default_workspace_path.display()
        );
        fs::create_dir(default_workspace_path).unwrap();
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use sea_orm::EntityTrait;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct ModelDataNew {
        pub models: String,
    }

    #[test] //由此判断这是一个测试函数
    fn it_works() {
        // let results = get_images();
        // assert_eq!(2, results.len())
        assert_eq!(true, "abc.txt".find("a").is_some());
    }
}
