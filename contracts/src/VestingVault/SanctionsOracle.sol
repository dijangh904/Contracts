// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../../interfaces/ISanctionsOracle.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title SanctionsOracle
 * @dev On-chain OFAC registry for real-time sanctions screening
 */
contract SanctionsOracle is ISanctionsOracle, Ownable {
    // Mapping of sanctioned addresses
    mapping(address => bool) private _sanctioned;
    
    // Array of all sanctioned addresses for enumeration
    address[] private _sanctionedAddresses;
    
    // Mapping to check if address is in the array
    mapping(address => uint256) private _sanctionedIndex;

    /**
     * @dev Constructor
     * @param initialOwner Initial owner of the contract
     */
    constructor(address initialOwner) Ownable() {
        transferOwnership(initialOwner);
    }

    /**
     * @dev Checks if an address is sanctioned
     * @param target The address to check
     * @return sanctioned True if the address is sanctioned, false otherwise
     */
    function isSanctioned(address target) external view override returns (bool) {
        return _sanctioned[target];
    }

    /**
     * @dev Batch check if addresses are sanctioned
     * @param targets Array of addresses to check
     * @return sanctioned Array of boolean results
     */
    function areSanctioned(address[] calldata targets) external view override returns (bool[] memory) {
        bool[] memory results = new bool[](targets.length);
        for (uint256 i = 0; i < targets.length; i++) {
            results[i] = _sanctioned[targets[i]];
        }
        return results;
    }

    /**
     * @dev Adds an address to the sanctions list (owner only)
     * @param target The address to sanction
     */
    function sanctionAddress(address target) external onlyOwner {
        require(!_sanctioned[target], "Address already sanctioned");
        require(target != address(0), "Cannot sanction zero address");
        
        _sanctioned[target] = true;
        _sanctionedAddresses.push(target);
        _sanctionedIndex[target] = _sanctionedAddresses.length - 1;
        
        emit AddressSanctioned(target, block.timestamp);
    }

    /**
     * @dev Removes an address from the sanctions list (owner only)
     * @param target The address to unsanction
     */
    function unsanctionAddress(address target) external onlyOwner {
        require(_sanctioned[target], "Address not sanctioned");
        
        // Remove from mapping
        _sanctioned[target] = false;
        
        // Remove from array by swapping with last element
        uint256 index = _sanctionedIndex[target];
        uint256 lastIndex = _sanctionedAddresses.length - 1;
        
        if (index != lastIndex) {
            address lastAddress = _sanctionedAddresses[lastIndex];
            _sanctionedAddresses[index] = lastAddress;
            _sanctionedIndex[lastAddress] = index;
        }
        
        _sanctionedAddresses.pop();
        delete _sanctionedIndex[target];
        
        emit AddressUnsanctioned(target, block.timestamp);
    }

    /**
     * @dev Gets the total number of sanctioned addresses
     * @return count The number of sanctioned addresses
     */
    function getSanctionedCount() external view returns (uint256) {
        return _sanctionedAddresses.length;
    }

    /**
     * @dev Gets all sanctioned addresses (paginated)
     * @param offset Starting index
     * @param limit Maximum number of addresses to return
     * @return addresses Array of sanctioned addresses
     */
    function getSanctionedAddresses(uint256 offset, uint256 limit) external view returns (address[] memory) {
        uint256 end = offset + limit;
        if (end > _sanctionedAddresses.length) {
            end = _sanctionedAddresses.length;
        }
        
        address[] memory addresses = new address[](end - offset);
        for (uint256 i = offset; i < end; i++) {
            addresses[i - offset] = _sanctionedAddresses[i];
        }
        
        return addresses;
    }

    /**
     * @dev Batch sanction multiple addresses (owner only)
     * @param targets Array of addresses to sanction
     */
    function batchSanction(address[] calldata targets) external onlyOwner {
        for (uint256 i = 0; i < targets.length; i++) {
            if (!_sanctioned[targets[i]] && targets[i] != address(0)) {
                _sanctioned[targets[i]] = true;
                _sanctionedAddresses.push(targets[i]);
                _sanctionedIndex[targets[i]] = _sanctionedAddresses.length - 1;
                emit AddressSanctioned(targets[i], block.timestamp);
            }
        }
    }

    /**
     * @dev Batch unsanction multiple addresses (owner only)
     * @param targets Array of addresses to unsanction
     */
    function batchUnsanction(address[] calldata targets) external onlyOwner {
        for (uint256 i = 0; i < targets.length; i++) {
            if (_sanctioned[targets[i]]) {
                // Remove from mapping
                _sanctioned[targets[i]] = false;
                
                // Remove from array by swapping with last element
                uint256 index = _sanctionedIndex[targets[i]];
                uint256 lastIndex = _sanctionedAddresses.length - 1;
                
                if (index != lastIndex) {
                    address lastAddress = _sanctionedAddresses[lastIndex];
                    _sanctionedAddresses[index] = lastAddress;
                    _sanctionedIndex[lastAddress] = index;
                }
                
                _sanctionedAddresses.pop();
                delete _sanctionedIndex[targets[i]];
                
                emit AddressUnsanctioned(targets[i], block.timestamp);
            }
        }
    }
}
