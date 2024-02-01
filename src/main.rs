

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
    use actix_web::{delete, error::{self, ErrorNotFound}, get, patch, post, put, web::{self, Data}, HttpResponse, Responder};
    use sqlx::{PgPool, Row};
    use crate::{domain::{Playlist, Song}, dtos::{self, CreatePlaylist, Info, PartialUpdatePlaylist}, state::State};

    //*Función GET All*/
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

    //*Función get_id */
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
    
        // Intenta ejecutar la consulta INSERT con RETURNING 
        if let Ok(row) = sqlx::query(query)
            .bind(&dto.name)
            .fetch_one(pool.as_ref())
            .await
        {
            let new_playlist = Playlist {
                name: row.try_get::<String, _>("name").unwrap_or_default(),
                songs: vec![],
            };
            // Si la inserción fue exitosa, simplemente retorna un mensaje de éxito
            Ok(web::Json(new_playlist))
        } else {
            // Si hubo un error en la inserción, retorna un mensaje de error
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


}







fn main() {
}
