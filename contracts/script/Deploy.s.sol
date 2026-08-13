// SPDX-License-Identifier: MIT
pragma solidity 0.8.36;

import { Script } from "forge-std/Script.sol";
import { console2 } from "forge-std/console2.sol";

import { MockERC20 } from "../src/mocks/MockERC20.sol";
import { EscrowFactory } from "../src/EscrowFactory.sol";

contract DeployScript is Script {

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        address factoryAdmin = vm.envOr("FACTORY_ADMIN", deployer);
        address dealCreator = vm.envOr("DEAL_CREATOR", deployer);

        vm.startBroadcast(deployerPrivateKey);

        MockERC20 token = new MockERC20();
        EscrowFactory factory = new EscrowFactory(factoryAdmin, dealCreator);

        vm.stopBroadcast();

        console2.log("Deployer:", deployer);
        console2.log("MockERC20:", address(token));
        console2.log("EscrowFactory:", address(factory));
        console2.log("Factory admin:", factoryAdmin);
        console2.log("Deal creator:", dealCreator);

        string memory objectKey = "deployment";
        string memory json = vm.serializeAddress(objectKey, "mockToken", address(token));
        json = vm.serializeAddress(objectKey, "escrowFactory", address(factory));
        json = vm.serializeAddress(objectKey, "factoryAdmin", factoryAdmin);
        json = vm.serializeAddress(objectKey, "dealCreator", dealCreator);
        json = vm.serializeUint(objectKey, "chainId", block.chainid);
        
        vm.writeJson(json, "./deployments/sepolia.json");
    }
}
