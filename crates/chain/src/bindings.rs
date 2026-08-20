use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint256);
    }

    #[sol(rpc)]
    interface IEscrowFactory {
        event DealCreated(
            bytes32 indexed dealId,
            address indexed dealAddress,
            uint256 dealNonce,
            address admin,
            address approver,
            address payer,
            address payee,
            IERC20 token,
            uint256 retentionBps
        );

        function createDeal(
            address _admin,
            address _approver,
            address _payer,
            address _payee,
            IERC20 _token,
            uint16 _retentionBps,
            uint256[] memory _milestoneAmounts
        ) external returns (IEscrowDeal deal);
    }

    #[sol(rpc)]
    interface IEscrowDeal {
        enum DealState {
            Draft,
            Funded,
            InProgress,
            Released
        }

        struct Milestone {
            uint256 amount;
            bool released;
        }

        event Funded(
            bytes32 indexed dealId,
            uint256 amount
        );

        event MilestoneReleased(
            bytes32 indexed dealId,
            uint256 indexed milestoneIndex,
            uint256 amount,
            uint256 retentionHeld
        );

        function state() external view returns (DealState);
        function payer() external view returns (address);
        function payee() external view returns (address);
        function approver() external view returns (address);
        function token() external view returns (IERC20);
        function retentionBps() external view returns (uint16);
        function fundedAmount() external view returns (uint256);
        function milestoneCount() external view returns (uint16);
        function getMilestone(uint16 _index) external view returns (Milestone memory);
        function dealId() external view returns (bytes32);
        function fund(uint256 _amount) external;
        function releaseMilestone(uint16 _index) external;
    }
}

pub type DealCreated = IEscrowFactory::DealCreated;
pub type Funded = IEscrowDeal::Funded;
pub type MilestoneReleased = IEscrowDeal::MilestoneReleased;

pub type DealState = IEscrowDeal::DealState;

#[cfg(test)]
mod tests {
    use super::*;
    
    use alloy::sol_types::SolEvent;
    
    #[test]
    fn deal_created_signature_matches_abi() {
        assert_eq!(
            DealCreated::SIGNATURE,
            "DealCreated(bytes32,address,uint256,address,address,address,address,address,uint256)"
        );
    }
    
    #[test]
    fn funded_signature_matches_abi() {
        assert_eq!(
            Funded::SIGNATURE,
            "Funded(bytes32,uint256)"
        );
    }
    
    #[test]
    fn milestone_released_signature_matches_abi() {
        assert_eq!(
            MilestoneReleased::SIGNATURE,
            "MilestoneReleased(bytes32,uint256,uint256,uint256)"
        );
    }
    
    #[test]
    fn deal_state_enum_has_four_variants() {
        let _ = DealState::Draft;
        let _ = DealState::Funded;
        let _ = DealState::InProgress;
        let _ = DealState::Released;
    }
}