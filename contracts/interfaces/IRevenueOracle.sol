// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IRevenueOracle
 * @dev Interface for revenue oracle with TWAP protection against flash loan manipulation
 */
interface IRevenueOracle {
    /**
     * @dev Gets the current revenue metric (TVL or monthly revenue)
     * @return revenue The current revenue value
     */
    function getCurrentRevenue() external view returns (uint256);

    /**
     * @dev Gets the 30-day TWAP (Time-Weighted Average Price) of revenue
     * @return twapRevenue The 30-day average revenue
     */
    function get30DayTWAP() external view returns (uint256);

    /**
     * @dev Gets the target revenue for KPI comparison
     * @return targetRevenue The target revenue value
     */
    function getTargetRevenue() external view returns (uint256);

    /**
     * @dev Checks if the oracle is functioning properly
     * @return isHealthy True if oracle is responding correctly
     */
    function isOracleHealthy() external view returns (bool);

    /**
     * @dev Updates the revenue data (called by oracle keeper)
     * @param newRevenue The new revenue value
     */
    function updateRevenue(uint256 newRevenue) external;

    /**
     * @dev Emitted when revenue is updated
     */
    event RevenueUpdated(uint256 oldRevenue, uint256 newRevenue, uint256 timestamp);
}
