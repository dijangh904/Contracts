// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title IVestingVault
 * @dev Interface for the vesting vault contract
 */
interface IVestingVault {
    struct Grant {
        uint256 amount;
        uint256 start;
        uint256 duration;
        uint256 claimed;
        bool isActive;
        bool isEscrowed; // New field for frozen tokens
    }

    /**
     * @dev Claims vested tokens for a beneficiary
     * @param beneficiary The address claiming tokens
     */
    function claim(address beneficiary) external;

    /**
     * @dev Gets the claimable amount for a beneficiary
     * @param beneficiary The address to check
     * @return The amount of tokens that can be claimed
     */
    function getClaimableAmount(address beneficiary) external view returns (uint256);

    /**
     * @dev Gets the grant details for a beneficiary
     * @param beneficiary The address to check
     * @return The grant details
     */
    function getGrant(address beneficiary) external view returns (Grant memory);

    /**
     * @dev Emitted when tokens are claimed
     */
    event TokensClaimed(address indexed beneficiary, uint256 amount);

    /**
     * @dev Emitted when tokens are frozen due to sanctions
     */
    event TokensFrozen(address indexed beneficiary, uint256 amount);

    /**
     * @dev Emitted when tokens are released from escrow
     */
    event TokensReleased(address indexed beneficiary, uint256 amount);

    /**
     * @dev Emitted when KPI multiplier is updated
     */
    event KPIMultiplierUpdated(
        uint256 oldMultiplier,
        uint256 oracleInput,
        uint256 newMultiplier,
        uint256 timestamp
    );
}
