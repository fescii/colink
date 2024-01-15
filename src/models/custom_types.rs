use diesel_derive_enum::DbEnum;
use diesel::Selectable;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone)]
#[derive(Debug, DbEnum)]
#[ExistingTypePath = "crate::db::schema::sql_types::InstitutionType"]
pub enum InstitutionType {
  Vocational,
  High,
  College,
  University,
  Technical,
  Other,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone)]
#[derive(Debug, DbEnum)]
#[ExistingTypePath = "crate::db::schema::sql_types::ProposalType"]
pub enum ProposalType {
  Proposal,
  Revised,
  Supplemental,
  Continuation,
  Notice,
  Solicited,
  Other
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone)]
#[derive(Debug, DbEnum)]
#[ExistingTypePath = "crate::db::schema::sql_types::RoleType"]
pub enum RoleType {
  Owner,
  Admin,
  Staff,
  Period
}