// SPDX-License-Identifier: MIT
pragma solidity 0.8.36;

import { Script } from "forge-std/Script.sol";
import { console2 } from "forge-std/console2.sol";

import { MockERC20 } from "../src/mocks/MockERC20.sol";
import { EscrowFactory } from "../src/EscrowFactory.sol";
import { IEscrowDeal } from "../src/interfaces/IEscrowDeal.sol";
import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract CreateDealScript is Script {
    struct DealConfig {
        address dealAdmin;
        address approver;
        address payer;
        address payee;
        uint16 retentionBps;
        uint256 mintAmount;
        uint256 fundTotal;
        uint256[] milestoneAmounts;
        bytes32 intentId;
    }

    function run() external {
        uint256 pk = vm.envUint("PRIVATE_KEY");
        address broadcaster = vm.addr(pk);

        DealConfig memory cfg = _loadConfig(broadcaster);

        vm.startBroadcast(pk);
        IEscrowDeal deal = _mintAndCreateDeal(cfg);
        vm.stopBroadcast();

        _logResult(broadcaster, cfg, deal);
    }

    function _loadConfig(address broadcaster) internal view returns (DealConfig memory cfg) {
        cfg.dealAdmin = vm.envOr("DEAL_ADMIN", broadcaster);
        cfg.approver = vm.envAddress("DEAL_APPROVER");
        cfg.payer = vm.envAddress("DEAL_PAYER");
        cfg.payee = vm.envAddress("DEAL_PAYEE");
        cfg.retentionBps = uint16(vm.envOr("RETENTION_BPS", uint256(0)));

        uint256 milestone0 = vm.envOr("MILESTONE_0", uint256(500_000));
        uint256 milestone1 = vm.envOr("MILESTONE_1", uint256(300_000));

        cfg.milestoneAmounts = new uint256[](2);
        cfg.milestoneAmounts[0] = milestone0;
        cfg.milestoneAmounts[1] = milestone1;
        cfg.fundTotal = milestone0 + milestone1;
        cfg.mintAmount = vm.envOr("MINT_AMOUNT", cfg.fundTotal);
        cfg.intentId = vm.envBytes32("DEAL_INTENT_ID");
    }

    function _mintAndCreateDeal(DealConfig memory cfg) internal returns (IEscrowDeal deal) {
        MockERC20 token = MockERC20(vm.envAddress("MOCK_TOKEN_ADDRESS"));
        EscrowFactory factory = EscrowFactory(vm.envAddress("ESCROW_FACTORY_ADDRESS"));

        token.mint(cfg.payer, cfg.mintAmount);

        deal = factory.createDeal(
            cfg.dealAdmin,
            cfg.approver,
            cfg.payer,
            cfg.payee,
            IERC20(address(token)),
            cfg.retentionBps,
            cfg.milestoneAmounts,
            cfg.intentId
        );
    }

    function _logResult(address broadcaster, DealConfig memory cfg, IEscrowDeal deal) internal view {
        console2.log("Broadcaster:", broadcaster);
        console2.log("Minted to payer:", cfg.mintAmount);
        console2.log("Payer:", cfg.payer);
        console2.log("Payee:", cfg.payee);
        console2.log("Approver:", cfg.approver);
        console2.log("EscrowDeal:", address(deal));
        console2.log("Deal state (Draft=0):", uint256(deal.state()));
        console2.log("Next: payer approves deal and calls fund(", cfg.fundTotal, ")");
    }
}
