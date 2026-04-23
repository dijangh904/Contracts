// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title MockERC20
 * @dev Mock ERC20 token for testing purposes
 */
contract MockERC20 is ERC20, Ownable {
    uint8 private _decimals;
    
    /**
     * @dev Constructor
     * @param name Token name
     * @param symbol Token symbol
     */
    constructor(string memory name, string memory symbol) ERC20(name, symbol) {
        _decimals = 18;
        transferOwnership(msg.sender);
    }
    
    /**
     * @dev Mint tokens to an address (owner only)
     * @param to Address to mint to
     * @param amount Amount to mint
     */
    function mint(address to, uint256 amount) external onlyOwner {
        _mint(to, amount);
    }
    
    /**
     * @dev Returns the number of decimals used
     * @return The number of decimals
     */
    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }
}
