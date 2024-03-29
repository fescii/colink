use actix_web::{web, HttpResponse, Responder, HttpRequest, HttpMessage};
use diesel::{ prelude::*, result::Error};
use crate::db::{
  connection::establish_connection,
  platform::platform::sections,
};
use crate::models::platform::{Section, NewSection, SectionIdentity};
use crate::configs::state::AppState;
use serde_json::json;
use crate::middlewares::auth::{auth::{JwtMiddleware, Claims}, section::* };


// Handler for creating new section/org section
pub async fn create_section(req: HttpRequest, _: JwtMiddleware, app_data: web::Data<AppState>, section_data: web::Json<NewSection>) -> impl Responder {
  //  Get extensions
  let ext = req.extensions();
  let mut conn = establish_connection(&app_data.config.database_url).await;


  // Use the 'get' method to retrieve the 'Claims' value from extensions
	if let Some(claims) = ext.get::<Claims>() {
		// Access 'user' from 'Claims'
		let _user = &claims.user;

    // Collect Registration data from the body
    match section_data.validate() {
      Ok(section) => {
        // Check if the section already exists
        if section_exists(&section.name, &mut conn) {
          return HttpResponse::Conflict().json(
            json!({
              "success": false,
              "message": "Similar section already exists"
            })
          );
        }

        match diesel::insert_into(sections::table)
        .values(&section)
        .get_result::<Section>(&mut conn)
        {
          Ok(section) => return HttpResponse::Ok().json(
            json!({
              "success": true,
              "section": section,
              "message": "Section added successfully"
            })
          ),
          Err(err) => {
            // Handle the database error and return an error response
            return	HttpResponse::InternalServerError().json(
              json!({
                "success": false,
                "message": format!("Failed to add section: {}", err.to_string())
              })
            )
          }
        }
      }
      Err(err) => {
        // Directly return the HttpResponse
        return HttpResponse::ExpectationFailed().json(
          json!({
            "success": false,
            "message": err.to_string()
          })
        )
      }
    }
	}
	else {
		return HttpResponse::BadRequest().json(
      json!({
        "success": false,
        "message": "Authorization failure!"
      })
    )
	}
}


// Handler for deleting existing section
pub async fn delete_section(req: HttpRequest, _: JwtMiddleware, app_data: web::Data<AppState>, section_data: web::Json<SectionIdentity>) -> impl Responder {
  //  Get extensions
  let ext = req.extensions();
  let mut conn = establish_connection(&app_data.config.database_url).await;


  // Use the 'get' method to retrieve the 'Claims' value from extensions
	if let Some(claims) = ext.get::<Claims>() {
		// Access 'user' from 'Claims'
		let _user = &claims.user;

    match section_data.validate() {
      Ok(validated_data) => {
        // Check if the section already exists
        match section_deleted(&validated_data.id, &validated_data.identity, &mut conn) {
          Ok(true) => {
            return HttpResponse::Ok().json(
              json!({
                "success": true,
                "message": format!("Section: {} is deleted successfully!", &validated_data.identity)
              })
            )
          }

          Ok(false) => {
            return HttpResponse::NotFound().json(
              json!({
                "success": false,
                "message": format!("Section: {} does not exists!", &validated_data.identity)
              })
            )
          }

          Err(_) => {
            return HttpResponse::InternalServerError().json(
              json!({
                "success": false,
                "message": "An internal error occurred while deleting the section!"
              })
            )
          }
        }

      },
      Err(err) => {
        // Directly return the HttpResponse
        return HttpResponse::ExpectationFailed().json(
          json!({
            "success": false,
            "message": err.to_string()
          })
        )
      }
    }
	}
	else {
		return HttpResponse::BadRequest().json(
      json!({
        "success": false,
        "message": "Authorization failure!"
      })
    )
	}
}


// Handler for updating existing section
pub async fn update_section(req: HttpRequest, _: JwtMiddleware, app_data: web::Data<AppState>, section_data: web::Json<Section>) -> impl Responder {
  //  Get extensions
  let ext = req.extensions();
  let mut conn = establish_connection(&app_data.config.database_url).await;

  // Use the 'get' method to retrieve the 'Claims' value from extensions
	if let Some(claims) = ext.get::<Claims>() {
		// Access 'user' from 'Claims'
		let _user = &claims.user;

    let section = section_data.into_inner();

    if section.id <= 0 {
      return HttpResponse::ExpectationFailed().json(
        json!({
          "success": false,
          "message": "Section validation error: zero(0) was encountered for value(id)"
        })
      )
    }

    // Check if the section already exists
    match section_updated(&section.id, &section, &mut conn) {
      Ok(updated_section) => {
        return HttpResponse::Ok().json(
          json!({
            "success": true,
            "section": updated_section,
            "message": format!("Section - ({}) - is updated successfully!", &section.name)
          })
        )
      },

      Err(Error::NotFound) => {
        HttpResponse::NotFound().json(
          json!({
            "success": false,
            "message": format!("Section - ({}) - does not exists!", &section.name)
          })
        )
      }

      Err(_) => {
        return HttpResponse::InternalServerError().json(
          json!({
            "success": false,
            "message": "An internal error occurred while updating the section!"
          })
        )
      }
    }

	}
	else {
		return HttpResponse::BadRequest().json(
      json!({
        "success": false,
        "message": "Authorization failure!"
      })
    )
	}
}