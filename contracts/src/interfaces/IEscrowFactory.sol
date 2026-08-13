// SPDX-License-Identifier: MIT
pragma solidity 0.8.36;

import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { IEscrowDeal } from "./IEscrowDeal.sol";

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