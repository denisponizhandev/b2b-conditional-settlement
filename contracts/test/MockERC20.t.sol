// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.36;

import {Test} from "forge-std/Test.sol";
import {MockERC20} from "../src/mocks/MockERC20.sol";

contract MockERC20Test is Test {
    MockERC20 public mockERC20;

    function setUp() public {
        mockERC20 = new MockERC20();
    }

    function test_mint() public {
        mockERC20.mint(address(this), 1_000_000);

        assertEq(mockERC20.totalSupply(), 1_000_000);
        assertEq(mockERC20.balanceOf(address(this)), 1_000_000);
    }

    function test_decimals() public view {
        assertEq(mockERC20.decimals(), 6);
    }

    function test_transfer() public {
        mockERC20.mint(address(this), 1_000_000);
        assertEq(mockERC20.balanceOf(address(this)), 1_000_000);
        
        bool success = mockERC20.transfer(address(1), 1_000_000);
        assertTrue(success);
        
        assertEq(mockERC20.balanceOf(address(this)), 0);
        assertEq(mockERC20.balanceOf(address(1)), 1_000_000);
    }
}