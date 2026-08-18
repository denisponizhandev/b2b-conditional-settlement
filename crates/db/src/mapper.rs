use uuid::Uuid;
use domain::{DealStatus, Milestone};
use crate::error::DbError;
use sqlx::{Row};
use sqlx::postgres::PgRow;

pub fn parse_status(raw: &str) -> Result<DealStatus, DbError> {
    match raw {
        "draft" => Ok(DealStatus::Draft),
        "funded" => Ok(DealStatus::Funded),
        "in_progress" => Ok(DealStatus::InProgress),
        "released" => Ok(DealStatus::Released),
        "disputed" => Ok(DealStatus::Disputed),
        "pending_chain_confirm" => Ok(DealStatus::PendingChainConfirm),
        "closed" => Ok(DealStatus::Closed),
        other => Err(DbError::InvalidStatus(other.to_string()))
    }
}

pub fn status_to_str(status: DealStatus) -> &'static str {
    match status {
        DealStatus::Draft => "draft",
        DealStatus::Funded => "funded",
        DealStatus::InProgress => "in_progress",
        DealStatus::Released => "released",
        DealStatus::Disputed => "disputed",
        DealStatus::PendingChainConfirm => "pending_chain_confirm",
        DealStatus::Closed => "closed"
    }
}

pub fn row_to_milestone(row: &PgRow) -> Result<Milestone, DbError> {  
    let index: i16 = row.try_get("milestone_index")?;
    let amount: i64 = row.try_get("amount")?;
    let released: bool = row.try_get("released")?;

    Ok(Milestone::new(
        index as u16,
        amount.try_into().map_err(|_| DbError::InvalidAmount)?,
        released,
    ))
}

pub fn row_to_deal_fields(row: &PgRow) -> Result<(Uuid, DealStatus, Uuid, Uuid, Option<String>), DbError> {
    let id: Uuid = row.try_get("id")?;
    let status: String = row.try_get("status")?;
    let status = parse_status(&status)?;
    let payer_org_id: Uuid = row.try_get("payer_org_id")?;
    let payee_org_id: Uuid = row.try_get("payee_org_id")?;

    let chain_address: Option<String> = row.try_get("chain_address")?;

    Ok((id, status, payer_org_id, payee_org_id, chain_address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_status_to_str() {
        assert_eq!("draft", status_to_str(DealStatus::Draft));
        assert_eq!("funded", status_to_str(DealStatus::Funded));
        assert_eq!("in_progress", status_to_str(DealStatus::InProgress));
        assert_eq!("released", status_to_str(DealStatus::Released));
        assert_eq!("disputed", status_to_str(DealStatus::Disputed));
        assert_eq!("pending_chain_confirm", status_to_str(DealStatus::PendingChainConfirm));
        assert_eq!("closed", status_to_str(DealStatus::Closed));
    }

    #[test]
    fn it_parses_status() {
        let status_draft = "draft";
        let status_funded = "funded";
        let status_in_progress = "in_progress";
        let status_released = "released";
        let status_disputed = "disputed";
        let status_pending_chain_confirm = "pending_chain_confirm";
        let status_closed = "closed";

        assert_eq!(DealStatus::Draft, parse_status(status_draft).expect("status_draft"));
        assert_eq!(DealStatus::Funded, parse_status(status_funded).expect("status_funded"));
        assert_eq!(DealStatus::InProgress, parse_status(status_in_progress).expect("status_in_progress"));
        assert_eq!(DealStatus::Released, parse_status(status_released).expect("status_released"));
        assert_eq!(DealStatus::Disputed, parse_status(status_disputed).expect("status_disputed"));
        assert_eq!(DealStatus::PendingChainConfirm, parse_status(status_pending_chain_confirm).expect("status_pending_chain_confirm"));
        assert_eq!(DealStatus::Closed, parse_status(status_closed).expect("status_closed"));
    }

    #[test]
    fn it_does_not_parse_wrong_status() {
        let invalid_status = "invalid_status";    
        let res = parse_status(invalid_status);

        assert!(matches!(
            res,
            Err(DbError::InvalidStatus(ref s)) if s == invalid_status
        ));
    }
}