// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ISanctionsOracle
 * @dev Interface for checking if an address is sanctioned
 */
interface ISanctionsOracle {
    /**
     * @dev Checks if an address is sanctioned
     * @param target The address to check
     * @return sanctioned True if the address is sanctioned, false otherwise
     */
    function isSanctioned(address target) external view returns (bool);

    /**
     * @dev Batch check if addresses are sanctioned
     * @param targets Array of addresses to check
     * @return sanctioned Array of boolean results
     */
    function areSanctioned(address[] calldata targets) external view returns (bool[] memory);

    /**
     * @dev Emitted when an address is added to sanctions list
     */
    event AddressSanctioned(address indexed target, uint256 timestamp);

    /**
     * @dev Emitted when an address is removed from sanctions list
     */
    event AddressUnsanctioned(address indexed target, uint256 timestamp);
}
