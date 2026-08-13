// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.36;

import { Test } from "forge-std/Test.sol";
import { EscrowDeal } from "../src/EscrowDeal.sol";
import { IEscrowDeal } from "../src/interfaces/IEscrowDeal.sol";
import { MockERC20 } from "../src/mocks/MockERC20.sol";

contract EscrowDealTest is Test {
    MockERC20 public token;

    address public admin = makeAddr("admin");
    address public payer = makeAddr("payer");
    address public payee = makeAddr("payee");
    address public approver = makeAddr("approver");

    bytes32 public dealId = keccak256("deal-1");

    uint256 internal constant MILESTONE_0 = 500_000;
    uint256 internal constant MILESTONE_1 = 300_000;
    uint256 internal constant TOTAL_TWO_MILESTONES = MILESTONE_0 + MILESTONE_1;

    function setUp() public {
        token = new MockERC20();
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

    function _deployDeal(uint256[] memory amounts) internal returns (EscrowDeal deal) {
        deal = new EscrowDeal(
            admin,
            approver,
            payer,
            payee,
            token,
            0,
            dealId,
            amounts
        );
    }

    function _fundDeal(EscrowDeal deal, uint256 amount) internal {
        token.mint(payer, amount);

        vm.prank(payer);
        token.approve(address(deal), amount);

        vm.prank(payer);
        deal.fund(amount);
    }

    function test_fund_success() public {
        EscrowDeal deal = _deployDeal(_twoMilestoneAmounts());

        _fundDeal(deal, TOTAL_TWO_MILESTONES);

        assertEq(uint256(deal.state()), uint256(IEscrowDeal.DealState.Funded));
        assertEq(deal.fundedAmount(), TOTAL_TWO_MILESTONES);
        assertEq(token.balanceOf(address(deal)), TOTAL_TWO_MILESTONES);
    }

    function test_fund_reverts_not_payer() public {
        EscrowDeal deal = _deployDeal(_twoMilestoneAmounts());

        token.mint(payer, TOTAL_TWO_MILESTONES);

        vm.prank(payer);
        token.approve(address(deal), TOTAL_TWO_MILESTONES);

        vm.prank(payee);
        vm.expectRevert("EscrowDeal: sender should be payer address");
        deal.fund(TOTAL_TWO_MILESTONES);
    }

    function test_fund_reverts_wrong_amount() public {
        EscrowDeal deal = _deployDeal(_twoMilestoneAmounts());

        token.mint(payer, TOTAL_TWO_MILESTONES);

        vm.prank(payer);
        token.approve(address(deal), TOTAL_TWO_MILESTONES);

        vm.prank(payer);
        vm.expectRevert("EscrowDeal: amount should be equal to the total milestones amount");
        deal.fund(TOTAL_TWO_MILESTONES - 1);
    }

    function test_fund_reverts_twice() public {
        EscrowDeal deal = _deployDeal(_twoMilestoneAmounts());

        _fundDeal(deal, TOTAL_TWO_MILESTONES);

        vm.prank(payer);
        vm.expectRevert("EscrowDeal: funding is only possible in the Draft state");
        deal.fund(TOTAL_TWO_MILESTONES);
    }

    function test_release_success() public {
        EscrowDeal deal = _deployDeal(_singleMilestoneAmounts());

        _fundDeal(deal, MILESTONE_0);

        vm.prank(approver);
        deal.releaseMilestone(0);

        assertEq(token.balanceOf(payee), MILESTONE_0);
        assertEq(uint256(deal.state()), uint256(IEscrowDeal.DealState.Released));

        IEscrowDeal.Milestone memory milestone = deal.getMilestone(0);
        assertTrue(milestone.released);
    }

    function test_release_reverts_not_approver() public {
        EscrowDeal deal = _deployDeal(_singleMilestoneAmounts());

        _fundDeal(deal, MILESTONE_0);

        vm.prank(payer);
        vm.expectRevert();
        deal.releaseMilestone(0);
    }

    function test_release_reverts_before_fund() public {
        EscrowDeal deal = _deployDeal(_singleMilestoneAmounts());

        vm.prank(approver);
        vm.expectRevert("EscrowDeal: release is only possible in the Funded or InProgress state");
        deal.releaseMilestone(0);
    }

    function test_release_reverts_out_of_order() public {
        EscrowDeal deal = _deployDeal(_twoMilestoneAmounts());

        _fundDeal(deal, TOTAL_TWO_MILESTONES);

        vm.prank(approver);
        vm.expectRevert("EscrowDeal: release prev milestone first");
        deal.releaseMilestone(1);
    }

    function test_single_milestone_flow() public {
        EscrowDeal deal = _deployDeal(_singleMilestoneAmounts());

        _fundDeal(deal, MILESTONE_0);

        vm.prank(approver);
        deal.releaseMilestone(0);

        assertEq(uint256(deal.state()), uint256(IEscrowDeal.DealState.Released));
        assertEq(deal.nextMilestoneIndexToRelease(), 1);
        assertEq(token.balanceOf(payee), MILESTONE_0);
    }

    function test_full_flow_two_milestones() public {
        EscrowDeal deal = _deployDeal(_twoMilestoneAmounts());

        _fundDeal(deal, TOTAL_TWO_MILESTONES);

        vm.prank(approver);
        deal.releaseMilestone(0);

        assertEq(uint256(deal.state()), uint256(IEscrowDeal.DealState.InProgress));
        assertEq(token.balanceOf(payee), MILESTONE_0);

        vm.prank(approver);
        deal.releaseMilestone(1);

        assertEq(uint256(deal.state()), uint256(IEscrowDeal.DealState.Released));
        assertEq(token.balanceOf(payee), TOTAL_TWO_MILESTONES);
        assertEq(token.balanceOf(address(deal)), 0);
        assertEq(deal.nextMilestoneIndexToRelease(), 2);
    }
}
