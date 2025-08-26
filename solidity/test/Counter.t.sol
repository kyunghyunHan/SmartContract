// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Counter} from "../src/Counter.sol";

//Test
contract CounterTest is Test {
    Counter public counter;
    // 1 . first setUP
    function setUp() public {
        counter = new Counter();
    }
 
    function test_Increment() public {
        counter.inc();
        assertEq(counter.get(), 1);
    }

   
}
