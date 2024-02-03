use std::{result, sync::{Arc, Mutex}};

use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use state::State;

//* domain */
mod domain{
    use serde::Serialize;

    #[derive(Debug, Serialize, Clone)]
    pub struct Song{
        pub name: String,
        pub author: String,
        pub duration_ms:u16
    }
    
    #[derive(Debug, Serialize, Clone)]
    pub struct Playlist{
        pub name: String,
        pub songs: Vec<Song>
    }
}

//*State */
mod state{
    use std::sync::{Mutex, Arc};
    use crate::domain::Playlist;
    
    pub struct State{
        pub playlist: Arc<Mutex<Vec<Playlist>>>
    }
}

//* dtos */
mod dtos{
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Info{
        pub id:usize
    }
    
    #[derive(Deserialize)]
    pub struct CreatePlaylist{
        pub name: String
    }
    
    #[derive(Deserialize)]
    pub struct PartialUpdatePlaylist{
        pub name: Option<String>
    }
}


//* endpoints */
mod endpoints{
    use actix_web::{delete, error::{self}, get, patch, post, put, web::{self}, HttpResponse, Responder};
    use sqlx::{PgPool, Row};
    use crate::{domain::Playlist, dtos::{CreatePlaylist, Info, PartialUpdatePlaylist}};



    //*Función GET ALL*/
    #[get("/playlist")]

    async fn playlist(pool: web::Data<PgPool>) -> impl Responder{
        let query= "SELECT * FROM dbCargo";

        if let Ok(rows) = sqlx::query(query)
        .fetch_all(pool.as_ref())
        .await{
            if !rows.is_empty(){
                //Mapea las filas PgRow a Playlist y Song
                let playlists: Vec<Playlist> = rows.into_iter().map(|row|{
                    Playlist{
                        name: row.try_get::<String, _>("name").unwrap_or_default(),
                        songs: Vec::new(),                    }
                }).collect();

                return Ok(web::Json(playlists));
            } else {
                return Err(error::ErrorNotFound("Don't have a playlists"));
            }
        } else {
            return Err(error::ErrorNotFound("Don't have a playlists"));
        }
    }

    //*Función GET_ID */
    #[get("/playlist/{id}")]
    async fn get_playlist(info: web::Path<Info>, pool: web::Data<PgPool>) -> impl Responder{
        let query = "SELECT * FROM dbCargo WHERE id=$1";
        if let Ok(row) = sqlx::query(query)
        .bind(info.id as i32)
        .fetch_one(pool.as_ref())
        .await
        {
            //Mapea la fila (pgRow) a playlist y song
            let playlists: Playlist = Playlist{
                name: row.try_get::<String, _>("name").unwrap_or_default(),
                songs: Vec::new(),
            };
            Ok(web::Json(playlists))
        } else {
            Err(error::ErrorNotFound("This playlist didn't exist"))
        }
    }

    //*Función POST */
    #[post("/playlist")]
    async fn create_playlist(dto: web::Json<CreatePlaylist>, pool: web::Data<PgPool>) -> impl Responder{

        let query = "INSERT INTO dbCargo (name) VALUES ($1) RETURNING id, name";
    
        if let Ok(row) = sqlx::query(query)
            .bind(&dto.name)
            .fetch_one(pool.as_ref())
            .await
        {
            let new_playlist = Playlist {
                name: row.try_get::<String, _>("name").unwrap_or_default(),
                songs: vec![],
            };
            Ok(web::Json(new_playlist))
        } else {
            Err(error::ErrorInternalServerError("Error creating playlist"))
        }
        
    }

    //*Función Delete */
    #[delete("/playlist/{id}")]
    async fn delete_playlist(info: web::Path<Info>, pool: web::Data<PgPool>) -> impl Responder{
        let query = "DELETE FROM dbCargo WHERE id=$1";

        if let Ok(_) = sqlx::query(query)
        .bind(info.id as i32)
        .fetch_one(pool.as_ref())
        .await{
            Ok(HttpResponse::Ok().json("Playlist deleted successfully"))
        } else {
            Err(error::ErrorNotFound("Playlist not found"))
        }
    }

 
    //*Función PUT */
    #[put("/playlist/{id}")]
    async fn update_playlist(info: web::Path<Info>, pool: web::Data<PgPool>, dto: web::Json<CreatePlaylist>) -> impl Responder{
        let query= "UPDATE dbCargo SET name = $1 WHERE id = $2 RETURNING name";
    
        if let Ok(row) = sqlx::query(query)
        .bind(&dto.name)
        .bind(info.id as i32) 
        .fetch_one(pool.as_ref())
        .await{
            let updated_playlist = Playlist{
                name: row.try_get::<String, _>("name").unwrap_or_default(),
                songs: vec![],
            };
            Ok(web::Json(updated_playlist))
        } else {
            Err(error::ErrorNotFound("playlist not found"))
        }
    }

    //*Funcion de PATCH */
    #[patch("/playlist/{id}")]
    async fn partial_update_playlist(info: web::Path<Info>, dto: web::Json<PartialUpdatePlaylist>, pool: web::Data<PgPool>) -> impl Responder{
        let query = "UPDATE dbCargo SET name = $1 WHERE id = $2 RETURNING id, name";
        if let Ok(row) = sqlx::query(query)
        .bind(&dto.name)
        .bind(info.id as i32)
        .fetch_one(pool.as_ref())
        .await{
            let updated_playlist = Playlist{
                name:row.try_get::<String, _>("name").unwrap_or_default(),
                songs: Vec::new(),
            };
            Ok(web::Json(updated_playlist))
        } else {
            Err(error::ErrorNotFound("Playlist not found"))
        }
             
    }

    pub fn config(cfg: &mut web::ServiceConfig){
        cfg.service(playlist);
        cfg.service(create_playlist);
        cfg.service(get_playlist);
        cfg.service(delete_playlist);
        cfg.service(update_playlist);
        cfg.service(partial_update_playlist);
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Configura la conexión a la base de datos PostgreSQL
    let database_url = "postgresql://username:password@localhost/database_name";
    let pool = PgPool::connect(database_url).await.expect("Failed to connect to the database.");

    // Configura el estado compartido para la aplicación Actix
    let state = web::Data::new(State {
        playlist: Arc::new(Mutex::new(Vec::new())),
    });

    // Inicia el servidor Actix Web
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(web::Data::new(pool.clone()))
            .configure(endpoints::config)  // Usa la función de configuración
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

