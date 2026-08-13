// SPDX-License-Identifier: MIT
pragma solidity 0.8.36;

import { IEscrowDeal } from "./interfaces/IEscrowDeal.sol";
import { AccessControl } from "@openzeppelin/contracts/access/AccessControl.sol";
import { ReentrancyGuard } from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract EscrowDeal is IEscrowDeal, AccessControl, ReentrancyGuard {
    using SafeERC20 for IERC20;

    bytes32 public constant APPROVER_ROLE = keccak256("APPROVER_ROLE");

    uint16 public constant BP = 10000;

    address public immutable payer;
    address public immutable payee;
    address public immutable approver;

    IERC20 public immutable token;

    uint16 public immutable retentionBps;
    uint16 public milestoneCount;
    uint16 public nextMilestoneIndexToRelease;

    bytes32 public immutable dealId;

    mapping(uint16 => IEscrowDeal.Milestone) public milestones;

    uint256 public fundedAmount;

    IEscrowDeal.DealState public state;

    constructor(
        address _approver, 
        address _payer, 
        address _payee, 
        IERC20 _token, 
        uint16 _retentionBps, 
        bytes32 _dealId,
        uint256[] memory _milestoneAmounts
    ) {
        require(_approver != address(0), "EscrowDeal: zero address");
        require(_payer != address(0), "EscrowDeal: zero address");
        require(_payee != address(0), "EscrowDeal: zero address");
        require(address(_token) != address(0), "EscrowDeal: zero address");

        require(_milestoneAmounts.length > 0, "EscrowDeal: milestone amounts cannot be empty");
        require(_milestoneAmounts.length <= 255, "EscrowDeal: milestone amounts cannot be more than 255");
        require(_retentionBps <= 10000, "EscrowDeal: retention bps cannot be more than 10000");

        approver = _approver;
        payer = _payer;
        payee = _payee;
        token = _token;
        retentionBps = _retentionBps;
        dealId = _dealId;
        fundedAmount = 0;
        nextMilestoneIndexToRelease = 0;

        for (uint16 i = 0; i < _milestoneAmounts.length; i++) {
            require(_milestoneAmounts[i] > 0,  "EscrowDeal: milestone amount cannot be 0");

            milestones[i] = IEscrowDeal.Milestone({
                amount: _milestoneAmounts[i],
                released: false
            });
        }

        milestoneCount = uint16(_milestoneAmounts.length);
        state = IEscrowDeal.DealState.Draft;

        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(APPROVER_ROLE, approver);
        
    }

    function getMilestone(uint16 _index) public view returns (IEscrowDeal.Milestone memory) {
        require(_index < milestoneCount, "EscrowDeal: milestone index out of bound");
        return milestones[_index];
    }

    function fund(uint256 _amount) public nonReentrant {
        require(msg.sender == payer, "EscrowDeal: sender should be payer address");
        require(state == IEscrowDeal.DealState.Draft, "EscrowDeal: funding is only possible in the Draft state");
    
        uint256 totalRequired = 0;

        for (uint16 i = 0; i < milestoneCount; i++) {
            totalRequired += milestones[i].amount;
        }

        require(_amount == totalRequired, "EscrowDeal: amount should be equal to the total milestones amount");

        fundedAmount = _amount;
        state = IEscrowDeal.DealState.Funded;

        token.safeTransferFrom(payer, address(this), _amount);

        emit Funded(dealId, _amount);
    }

    function releaseMilestone(uint16 _index) public nonReentrant onlyRole(APPROVER_ROLE) {
        require(
            state == IEscrowDeal.DealState.Funded || 
            state == IEscrowDeal.DealState.InProgress, 
            "EscrowDeal: release is only possible in the Funded or InProgress state"
        );

        require(_index < milestoneCount, "EscrowDeal: milestone index out of bound");
        require(!milestones[_index].released, "EscrowDeal: milestone already released");
        require(_index == nextMilestoneIndexToRelease, "EscrowDeal: release prev milestone first");

        // TODO: work with retentionHeld after MVP (currently stays on contract)
        uint256 amount = milestones[_index].amount;
        uint256 retentionHeld = amount * retentionBps / BP;
        uint256 payout = amount - retentionHeld;

        require(token.balanceOf(address(this)) >= amount, "EscrowDeal: not enough token balance");

        nextMilestoneIndexToRelease++;
        milestones[_index].released = true;

        if (nextMilestoneIndexToRelease < milestoneCount) {
            state = IEscrowDeal.DealState.InProgress;
        } else if (nextMilestoneIndexToRelease == milestoneCount) {
            state = IEscrowDeal.DealState.Released;
        }

        token.safeTransfer(payee, payout);

        emit MilestoneReleased(dealId, _index, amount, retentionHeld);
    }
}