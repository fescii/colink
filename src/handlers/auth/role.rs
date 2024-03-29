use actix_web::{web, HttpResponse, Responder, HttpRequest, HttpMessage};
use tokio::spawn;
use diesel::prelude::*;
use diesel::result::{Error, DatabaseErrorKind};
use chrono::{Utc, Days};
use crate::db::{
	connection::establish_connection,
	platform::platform::roles::dsl::*
};
use crate::models::{
	platform::{
		Role, NewRole, InsertableRole,
		RoleData, RolePrivileges, RoleExpiry
	},
	platform::InsertableLog,
	custom::{ ActionType, LogType }
};
use crate::configs::state::AppState;
use serde_json::json;
use crate::middlewares::{
	auth::{
		auth::{JwtMiddleware, Claims},
		role::*
	},
	log::log::*
};
use crate::utilities::time_utility::future_date;

// Handler for creating new Role
pub async fn create_role(
	req: HttpRequest,
	_: JwtMiddleware,
	app_data: web::Data<AppState>,
	role_data: web::Json<NewRole>
) -> impl Responder {
	//  Get extensions
	let ext = req.extensions();
	let mut conn = establish_connection(&app_data.config.database_url).await;

	// Use the 'get' method to retrieve the 'Claims' value from extensions
	if let Some(claims) = ext.get::<Claims>() {
		// Access 'user' from 'Claims'
		let user = &claims.user;

		// Collect Role data from the body
		match role_data.validate() {
			Ok(role) => {
				//Check if user is authorized to create the role
				match check_authority(&user.id, &role.section, &role.base, &mut conn).await {
					Ok(true) => {
						// Check if the Role already exists
						match role_exists(&role.author, &role.section, &mut conn) {
							Ok(true) => {
									return HttpResponse::Conflict().json(
										json!({
											"success": false,
											"message": "Same role already exists!"
										})
									);
								}
							Ok(false) => {

								let expiry_date = future_date(role.expiry).await;

								// No existing role - creating one
								let new_role = InsertableRole {
									section: role.section,
									base: role.base,
									name: role.name,
									author: role.author,
									privileges: role.privileges,
									expiry: expiry_date,
								};

								match role_created(&new_role, &mut conn).await {
									Ok(role) => {
										//Create section log
										let new_log = new_section_log(
											user.id,
											role.section,
											ActionType::Create,
											format!("{} created a new role", &user.username)
										).await;

										// Spawn an independent task(Record log)
										spawn(async move {
											create_log(&new_log, &mut conn).await;
										});

										return HttpResponse::Ok().json(
											json!({
												"success": true,
												"role": role,
												"message": format!("Role - ({}) - was created successfully", &role.name)
											})
										)
									},
									Err(Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _)) => {
										return HttpResponse::NotFound().json(
											json!({
												"success": false,
												"message": "Section or User does not exists"
											})
										)
									}
									Err(_) => {
										// Handle the database error and return an error response
										return	HttpResponse::InternalServerError().json(
											json!({
												"success": false,
												"message": "Failed to create the role: Internal Error Occurred!"
											})
										)
									}
								}
							},
							Err(_) => {
								return HttpResponse::InternalServerError().json(
									json!({
										"success": false,
										"message": "Failed to create the role: Internal Error Occurred!"
									})
								)
							}
						}
					},

					Ok(false) => {
						//Create section log
						let new_log = new_section_log(
							user.id,
							role.section,
							ActionType::Create,
							format!("Unauthorized User({}) tried to create a new role", &user.username)
						).await;

						// Spawn an independent task(Record log)
						spawn(async move {
							create_log(&new_log, &mut conn).await;
						});

						return HttpResponse::Forbidden().json(
							json!({
								"success": false,
								"message": "You're not authorized to create the role!"
							})
						)
					}

					Err(_) => {
						return HttpResponse::InternalServerError().json(
							json!({
								"success": false,
								"message": "Failed to create the role: Internal Error Occurred!"
							})
						)
					}
				}
			}

			Err(err) => {
				// Directly return the HttpResponse
				return HttpResponse::BadRequest().json(
					json!({
						"success": false,
						"message": err
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


// Handler for deleting existing role
pub async fn delete_role(
	req: HttpRequest,
	_: JwtMiddleware,
	app_data: web::Data<AppState>,
	role_data: web::Json<RoleData>
) -> impl Responder {
	//  Get extensions
	let ext = req.extensions();
	let mut conn = establish_connection(&app_data.config.database_url).await;

	// Use the 'get' method to retrieve the 'Claims' value from extensions
	if let Some(claims) = ext.get::<Claims>() {
		// Access 'user' from 'Claims'
		let user = &claims.user;

		match role_data.validate() {
			Ok(role) => {
				match check_authority(&user.id, &role.section, &role.base, &mut conn).await {
					Ok(true) => {
						// Attempt to delete the role
						match role_deleted(&role.id, &mut conn) {
							Ok(true) => {

								//Create section log
								let new_log = new_section_log(
									user.id,
									role.section,
									ActionType::Delete,
									format!("{} deleted a role", &user.username)
								).await;

								// Spawn an independent task(Record log)
								spawn(async move {
									create_log(&new_log, &mut conn).await;
								});

								return HttpResponse::Ok().json(
									json!({
										"success": true,
										"message": "Role is deleted successfully!"
									})
								)
							}
							Ok(false) => {
								return HttpResponse::NotFound().json(
									json!({
										"success": false,
										"message": "Role does not exists!"
									})
								)
							}
							Err(_) => {
								return HttpResponse::InternalServerError().json(
									json!({
										"success": false,
										"message": "Internal server error has occurred!"
									})
								)
							}
						}
					}

					Ok(false) => {
						// Create new log & save it to db
						let new_log = InsertableLog {
							audit: 	LogType::Action,
							author: user.id,
							target: role.section,
							name: "sections".to_owned(),
							action: ActionType::Create,
							verb: format!("Unauthorized user -({})- tried to delete role", &user.username),
						};
						// Spawn an independent task(Record log)
						spawn(async move {
							create_log(&new_log, &mut conn).await;
						});

						return HttpResponse::Forbidden().json(
							json!({
								"success": false,
								"message": "You're not authorized to create the role!"
							})
						)
					}
					Err(_) => {
						return HttpResponse::InternalServerError().json(
							json!({
								"success": false,
								"message": "Failed to create the role: Internal Error Occurred!"
							})
						)
					}
				}
			},
			Err(err) => {
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


// Handler for updating privileges of existing role
pub async fn update_privileges(
	req: HttpRequest,
	_: JwtMiddleware,
	app_data: web::Data<AppState>,
	role_privileges: web::Json<RolePrivileges>
) -> impl Responder {
	//  Get extensions
	let ext = req.extensions();
	let mut conn = establish_connection(&app_data.config.database_url).await;

	// Use the 'get' method to retrieve the 'Claims' value from extensions
	if let Some(claims) = ext.get::<Claims>() {
		// Access 'user' from 'Claims'
		let user = &claims.user;

		match role_privileges.validate() {
			Ok(role) => {
				match check_authority(&user.id, &role.section, &role.base, &mut conn).await {
					Ok(true) => {
						// Check if the section already exists
						match privileges_updated(&role, &mut conn) {
							Ok(updated_role) => {
								// Create new log & save it to db
								let new_log = InsertableLog {
									audit: 	LogType::Action,
									author: user.id,
									target: updated_role.section,
									name: "sections".to_owned(),
									action: ActionType::Create,
									verb: format!("{} updated role privileges", &user.username),
								};
								// Spawn an independent task(Record log)
								spawn(async move {
									create_log(&new_log, &mut conn).await;
								});
								return HttpResponse::Ok().json(
									json!({
										"success": true,
										"role": updated_role,
										"message": format!("Privileges for Role - ({}) - is updated successfully!", &updated_role.name)
									})
								)
							}
							Err(Error::NotFound) => {
								return HttpResponse::NotFound().json(
									json!({
										"success": false,
										"message": "No such role was  found!"
									})
								)
							}
							Err(err) => {
								return HttpResponse::InternalServerError().json(
									json!({
										"success": false,
										"message": format!("Internal server error has occurred: {}", err)
									})
								)
							}
						}
					}
					Ok(false) => {
						// Create new log & save it to db
						let new_log = InsertableLog {
							audit: 	LogType::Action,
							author: user.id,
							target: role.section,
							name: "sections".to_owned(),
							action: ActionType::Update,
							verb: format!("Unauthorized user -({})- tried to update role privileges", &user.username),
						};
						// Spawn an independent task(Record log)
						spawn(async move {
							create_log(&new_log, &mut conn).await;
						});
						return HttpResponse::Forbidden().json(
							json!({
								"success": false,
								"message": "You're not authorized to create the role!"
							})
						)
					}
					Err(_) => {
						return HttpResponse::InternalServerError().json(
							json!({
								"success": false,
								"message": "Failed to create the role: Internal Error Occurred!"
							})
						)
					}
				}

			},
			Err(err) => {
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


// Handler for updating expiry date
pub async fn update_expiry(
	req: HttpRequest,
	_: JwtMiddleware,
	app_data: web::Data<AppState>,
	role_data: web::Json<RoleExpiry>
) -> impl Responder {
	//  Get extensions
	let ext = req.extensions();
	let mut conn = establish_connection(&app_data.config.database_url).await;

	// Use the 'get' method to retrieve the 'Claims' value from extensions
	if let Some(claims) = ext.get::<Claims>() {
		// Access 'user' from 'Claims'
		let user = &claims.user;

		// let role_expiry = role_data.into_inner();

		match role_data.validate() {
			Ok(role_expiry) => {
				match check_authority(&user.id, &role_expiry.section, &role_expiry.base, &mut conn).await {
					Ok(true) => {
						match roles.filter(id.eq(role_expiry.id)).first::<Role>(&mut conn) {
							Ok(mut role) => {
								// If expiry days exists add the supplied number/ else supplied convert to future date from today
								let duration = Days::new(role_expiry.expiry.try_into().unwrap());
								if role.expiry.is_some() {
									let today_date = Utc::now().naive_utc();
									let date_time = role.expiry.unwrap() + duration;
									let diff_days = (date_time - today_date).num_days();
									if diff_days <= 0 || diff_days > 180 {
										return HttpResponse::ExpectationFailed().json(
											json!({
												"success": false,
												"message": "Roles permissions cannot be less than 1 or exceed 180 days!"
											})
										)
									}
									//Update expiry
        					role.expiry = Some(date_time);
								}
								else {
									let initial_date = Utc::now();
									let future_date = initial_date + duration;
									role.expiry = Some(future_date.naive_utc())
								};

								// Check if the section expiry date was updated
								match expiry_updated(&role, &mut conn) {
									Ok(updated_role) => {

										// Create new log & save it to db
										let new_log = InsertableLog {
											audit: 	LogType::Action,
											author: user.id,
											target: updated_role.section,
											name: "sections".to_owned(),
											action: ActionType::Update,
											verb: format!("{} updated a role", &user.username),
										};
										// Spawn an independent task(Record log)
										spawn(async move {
											create_log(&new_log, &mut conn).await;
										});

										return HttpResponse::Ok().json(
											json!({
												"success": true,
												"role": updated_role,
												"message": format!("Expiry for Role - ({}) - is updated successfully!", &updated_role.name)
											})
										)
									}

									Err(_) => {
										return HttpResponse::InternalServerError().json(
											json!({
												"success": false,
												"message": "Internal server error has occurred while updating role!"
											})
										)
									}
								}
							},
							Err(Error::NotFound) => {
								return HttpResponse::NotFound().json(
									json!({
										"success": false,
										"message": "No such role was  found!"
									})
								)
							}
							Err(_) => {
								return HttpResponse::InternalServerError().json(
									json!({
										"success": false,
										"message": "Internal server error has occurred while updating role expiry!"
									})
								)
							}
						}
					}

					Ok(false) => {
						// Create new log & save it to db
						let new_log = InsertableLog {
							audit: 	LogType::Action,
							author: user.id,
							target: role_expiry.section,
							name: "sections".to_owned(),
							action: ActionType::Update,
							verb: format!("Unauthorized user -({})- tried to update a role", &user.username),
						};
						// Spawn an independent task(Record log)
						spawn(async move {
							create_log(&new_log, &mut conn).await;
						});

						return HttpResponse::Forbidden().json(
							json!({
								"success": false,
								"message": "You're not authorized to create the role!"
							})
						)
					}

					Err(_) => {
						return HttpResponse::InternalServerError().json(
							json!({
								"success": false,
								"message": "Failed to create the role: Internal Error Occurred!"
							})
						)
					}
				}
			}
			Err(err) => {
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