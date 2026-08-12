// SPDX-License-Identifier: MIT
pragma solidity 0.8.36;

interface IEscrowDeal {
    enum DealState {
        Draft,
        Funded,
        InProgress,
        Released
        // TODO: Disputed, Refunded
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
    function token() external view returns (address);
    function retentionBps() external view returns (uint16);
    function fundedAmount() external view returns (uint256);
    function milestoneCount() external view returns (uint8);
    function getMilestone(uint8 index) external view returns (Milestone memory);
    function dealId() external view returns (bytes32);

    function fund(uint256 amount) external;
    function releaseMilestone(uint8 index) external;
}