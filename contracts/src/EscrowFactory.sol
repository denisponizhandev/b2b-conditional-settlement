// SPDX-License-Identifier: MIT
pragma solidity 0.8.36;

import { AccessControl } from "@openzeppelin/contracts/access/AccessControl.sol";
import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import { EscrowDeal } from "./EscrowDeal.sol";
import { IEscrowDeal } from "./interfaces/IEscrowDeal.sol";
import { IEscrowFactory } from "./interfaces/IEscrowFactory.sol";

contract EscrowFactory is IEscrowFactory, AccessControl {

    uint256 public dealCount;

    bytes32 public constant DEAL_CREATOR_ROLE = keccak256("DEAL_CREATOR_ROLE");

    mapping (bytes32 => IEscrowDeal) public deals;

    constructor(
        address _admin, 
        address _dealCreator
    ) {
        dealCount = 0;

        _grantRole(DEFAULT_ADMIN_ROLE, _admin);
        _grantRole(DEAL_CREATOR_ROLE, _dealCreator);
    }

    function createDeal(
        address _admin,
        address _approver, 
        address _payer, 
        address _payee, 
        IERC20 _token, 
        uint16 _retentionBps, 
        uint256[] memory _milestoneAmounts
    ) public onlyRole(DEAL_CREATOR_ROLE) returns (IEscrowDeal deal) {

        require(_admin != address(0), "EscrowDeal: zero address");
        
        dealCount++;
        bytes32 dealId = keccak256(abi.encode(dealCount));

        deal = new EscrowDeal(
            _admin,
            _approver, 
            _payer, 
            _payee, 
            _token, 
            _retentionBps, 
            dealId,
            _milestoneAmounts
        );

        deals[dealId] = deal;

        emit DealCreated(
            dealId, 
            address(deal),
            dealCount,
            _admin,
            _approver, 
            _payer, 
            _payee, 
            _token,
            _retentionBps
        );
    }
}