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
        uint256 escrowed_amount; // amount currently held in escrow for this grant
        // Tax configuration
        uint16 tax_bps; // basis points, parts per 10,000
        address tax_authority; // address receiving withheld tax
        address tax_asset; // if non-zero and different from vested token, tax portion will be liquidated to this asset
        uint256 cumulative_taxes_paid; // accumulated taxes paid (in tax_asset units when liquidated, otherwise token units)
        uint256 tax_rounding_accumulator; // accumulator for fractional tax parts to avoid losing stroops
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
     * @dev Emitted when tax is withheld from a claim and sent to authority.
     */
    event TaxWithheld(address indexed beneficiary, uint256 gross, uint256 taxAmount, uint256 net);
}
