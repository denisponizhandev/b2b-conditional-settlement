// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.36;

import { Test } from "forge-std/Test.sol";
import { EscrowFactory } from "../src/EscrowFactory.sol";
import { IEscrowDeal } from "../src/interfaces/IEscrowDeal.sol";
import { IEscrowFactory } from "../src/interfaces/IEscrowFactory.sol";
import { MockERC20 } from "../src/mocks/MockERC20.sol";

contract EscrowFactoryTest is Test {
    MockERC20 public token;
    EscrowFactory public factory;

    address public admin = makeAddr("admin");
    address public dealCreator = makeAddr("dealCreator");
    address public payer = makeAddr("payer");
    address public payee = makeAddr("payee");
    address public approver = makeAddr("approver");

    uint256 internal constant MILESTONE_0 = 500_000;
    uint256 internal constant MILESTONE_1 = 300_000;
    uint256 internal constant TOTAL_TWO_MILESTONES = MILESTONE_0 + MILESTONE_1;

    function setUp() public {
        token = new MockERC20();
        factory = new EscrowFactory(admin, dealCreator);
    }

    function _twoMilestoneAmounts() internal pure returns (uint256[] memory amounts) {
        amounts = new uint256[](2);
        amounts[0] = MILESTONE_0;
        amounts[1] = MILESTONE_1;
    }

    function _singleMilestoneAmounts() internal pure returns (uint256[] memory amounts) {
        amounts = new uint256[](1);
        amounts[0] = MILESTONE_0;
    }

    function _dealIdForNonce(uint256 nonce) internal pure returns (bytes32) {
        return keccak256(abi.encode(nonce));
    }

    function _createDeal(uint256[] memory amounts) internal returns (IEscrowDeal deal) {
        vm.prank(dealCreator);
        deal = factory.createDeal(
            admin,
            approver,
            payer,
            payee,
            token,
            0,
            amounts
        );
    }

    function _fundDeal(IEscrowDeal deal, uint256 amount) internal {
        token.mint(payer, amount);

        vm.prank(payer);
        token.approve(address(deal), amount);

        vm.prank(payer);
        deal.fund(amount);
    }

    function test_createDeal_success() public {
        IEscrowDeal deal = _createDeal(_twoMilestoneAmounts());

        assertTrue(address(deal) != address(0));
        assertEq(factory.dealCount(), 1);

        bytes32 dealId = _dealIdForNonce(1);
        assertEq(address(factory.deals(dealId)), address(deal));

        assertEq(deal.payer(), payer);
        assertEq(deal.payee(), payee);
        assertEq(deal.approver(), approver);
        assertEq(address(deal.token()), address(token));
        assertEq(deal.milestoneCount(), 2);
        assertEq(uint256(deal.state()), uint256(IEscrowDeal.DealState.Draft));
    }

    function test_createDeal_two_deals_have_different_addresses() public {
        IEscrowDeal deal1 = _createDeal(_singleMilestoneAmounts());
        IEscrowDeal deal2 = _createDeal(_twoMilestoneAmounts());

        assertTrue(address(deal1) != address(deal2));
        assertEq(factory.dealCount(), 2);

        assertEq(address(factory.deals(_dealIdForNonce(1))), address(deal1));
        assertEq(address(factory.deals(_dealIdForNonce(2))), address(deal2));
    }

    function test_createDeal_reverts_not_deal_creator() public {
        vm.prank(payer);
        vm.expectRevert();
        factory.createDeal(
            admin,
            approver,
            payer,
            payee,
            token,
            0,
            _singleMilestoneAmounts()
        );
    }

    function test_createDeal_reverts_zero_admin() public {
        vm.prank(dealCreator);
        vm.expectRevert("EscrowDeal: zero address");
        factory.createDeal(
            address(0),
            approver,
            payer,
            payee,
            token,
            0,
            _singleMilestoneAmounts()
        );
    }

    function test_createDeal_emits_DealCreated() public {
        bytes32 expectedDealId = _dealIdForNonce(1);

        vm.expectEmit(true, false, false, true);
        emit IEscrowFactory.DealCreated(
            expectedDealId,
            address(0),
            1,
            admin,
            approver,
            payer,
            payee,
            token,
            0
        );

        _createDeal(_singleMilestoneAmounts());
    }

    function test_full_flow_via_factory() public {
        IEscrowDeal deal = _createDeal(_twoMilestoneAmounts());

        _fundDeal(deal, TOTAL_TWO_MILESTONES);

        assertEq(uint256(deal.state()), uint256(IEscrowDeal.DealState.Funded));
        assertEq(token.balanceOf(address(deal)), TOTAL_TWO_MILESTONES);

        vm.prank(approver);
        deal.releaseMilestone(0);

        assertEq(uint256(deal.state()), uint256(IEscrowDeal.DealState.InProgress));
        assertEq(token.balanceOf(payee), MILESTONE_0);

        vm.prank(approver);
        deal.releaseMilestone(1);

        assertEq(uint256(deal.state()), uint256(IEscrowDeal.DealState.Released));
        assertEq(token.balanceOf(payee), TOTAL_TWO_MILESTONES);
        assertEq(token.balanceOf(address(deal)), 0);
    }
}
